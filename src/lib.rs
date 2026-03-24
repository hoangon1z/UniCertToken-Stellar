#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Address, Env, String, symbol_short,
};

const DAY: u32 = 17280;

// ===== Data Keys =====
#[contracttype]
pub enum DataKey {
    Admin,            // Địa chỉ admin (trường đại học)
    CertCount,        // Tổng số chứng chỉ đã phát hành
    Cert(u64),        // Chi tiết chứng chỉ theo ID
    StudentCert(Address), // Mapping sinh viên -> cert ID
    Revoked(u64),     // Chứng chỉ bị thu hồi
    UniName,          // Tên trường đại học
}

// ===== Cấu trúc Chứng Chỉ =====
#[contracttype]
#[derive(Clone, Debug)]
pub struct Certificate {
    pub id: u64,              // ID chứng chỉ
    pub student: Address,     // Địa chỉ sinh viên
    pub student_name: String, // Tên sinh viên
    pub degree: String,       // Bằng cấp (VD: "Cử nhân Công nghệ Thông tin")
    pub major: String,        // Ngành học
    pub graduation_year: u32, // Năm tốt nghiệp
    pub issued_at: u64,       // Timestamp phát hành
    pub gpa: u32,             // GPA x100 (VD: 350 = 3.50)
}

// ===== Mã lỗi =====
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotAdmin = 1,             // Không phải admin
    AlreadyIssued = 2,        // Sinh viên đã có chứng chỉ
    CertNotFound = 3,         // Không tìm thấy chứng chỉ
    CertRevoked = 4,          // Chứng chỉ đã bị thu hồi
    InvalidGpa = 5,           // GPA không hợp lệ
    AlreadyInitialized = 6,   // Contract đã được khởi tạo
}

#[contract]
pub struct UniCertToken;

#[contractimpl]
impl UniCertToken {
    /// Khởi tạo contract - gọi một lần khi deploy
    /// admin: địa chỉ của trường đại học (người có quyền phát hành)
    /// uni_name: tên trường đại học
    pub fn initialize(env: Env, admin: Address, uni_name: String) -> Result<(), Error> {
        // Kiểm tra chưa khởi tạo
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UniName, &uni_name);
        env.storage().instance().set(&DataKey::CertCount, &0_u64);
        env.storage().instance().extend_ttl(6 * DAY, 7 * DAY);
        Ok(())
    }

    /// Phát hành chứng chỉ cho sinh viên (chỉ admin/trường mới gọi được)
    pub fn issue_certificate(
        env: Env,
        student: Address,
        student_name: String,
        degree: String,
        major: String,
        graduation_year: u32,
        gpa: u32,
    ) -> Result<u64, Error> {
        // Xác thực admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        // Kiểm tra GPA hợp lệ (0-400, tức 0.00-4.00)
        if gpa > 400 {
            return Err(Error::InvalidGpa);
        }

        // Kiểm tra sinh viên chưa có chứng chỉ
        if env
            .storage()
            .persistent()
            .has(&DataKey::StudentCert(student.clone()))
        {
            return Err(Error::AlreadyIssued);
        }

        // Tạo ID mới
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CertCount)
            .unwrap_or(0)
            + 1;

        // Tạo chứng chỉ
        let cert = Certificate {
            id,
            student: student.clone(),
            student_name,
            degree,
            major,
            graduation_year,
            issued_at: env.ledger().timestamp(),
            gpa,
        };

        // Lưu chứng chỉ
        env.storage()
            .persistent()
            .set(&DataKey::Cert(id), &cert);
        env.storage()
            .persistent()
            .set(&DataKey::StudentCert(student.clone()), &id);

        // Cập nhật counter
        env.storage()
            .instance()
            .set(&DataKey::CertCount, &id);

        // Gia hạn TTL
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Cert(id), 89 * DAY, 90 * DAY);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::StudentCert(student.clone()), 89 * DAY, 90 * DAY);
        env.storage()
            .instance()
            .extend_ttl(6 * DAY, 7 * DAY);

        // Phát sự kiện
        env.events()
            .publish((symbol_short!("issued"), student), id);

        Ok(id)
    }

    /// Thu hồi chứng chỉ (chỉ admin - trường hợp phát hiện gian lận)
    pub fn revoke_certificate(env: Env, cert_id: u64) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        // Kiểm tra chứng chỉ tồn tại
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Cert(cert_id))
        {
            return Err(Error::CertNotFound);
        }

        // Đánh dấu thu hồi
        env.storage()
            .persistent()
            .set(&DataKey::Revoked(cert_id), &true);

        // Gia hạn TTL để flag thu hồi không bị xóa
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Revoked(cert_id), 89 * DAY, 90 * DAY);

        env.events()
            .publish((symbol_short!("revoked"),), cert_id);

        Ok(())
    }

    /// Xác minh chứng chỉ - ai cũng có thể gọi để kiểm tra
    /// Trả về chứng chỉ nếu hợp lệ, lỗi nếu không tìm thấy hoặc đã thu hồi
    pub fn verify_certificate(env: Env, cert_id: u64) -> Result<Certificate, Error> {
        // Kiểm tra chứng chỉ bị thu hồi
        if env
            .storage()
            .persistent()
            .has(&DataKey::Revoked(cert_id))
        {
            return Err(Error::CertRevoked);
        }

        // Lấy chứng chỉ
        env.storage()
            .persistent()
            .get(&DataKey::Cert(cert_id))
            .ok_or(Error::CertNotFound)
    }

    /// Tra cứu chứng chỉ theo địa chỉ sinh viên
    pub fn get_cert_by_student(env: Env, student: Address) -> Result<Certificate, Error> {
        let cert_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::StudentCert(student))
            .ok_or(Error::CertNotFound)?;

        Self::verify_certificate(env, cert_id)
    }

    /// Kiểm tra chứng chỉ đã bị thu hồi chưa
    pub fn is_revoked(env: Env, cert_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Revoked(cert_id))
    }

    /// Tổng số chứng chỉ đã phát hành
    pub fn total_certificates(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::CertCount)
            .unwrap_or(0)
    }

    /// Lấy tên trường đại học
    pub fn university_name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::UniName)
            .unwrap()
    }
}

// ===== TESTS =====
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        assert_eq!(
            client.university_name(),
            String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM")
        );
        assert_eq!(client.total_certificates(), 0);
    }

    #[test]
    fn test_issue_certificate() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        // Phát hành chứng chỉ
        let cert_id = client.issue_certificate(
            &student,
            &String::from_str(&env, "Nguyen Van A"),
            &String::from_str(&env, "Cu nhan Cong nghe Thong tin"),
            &String::from_str(&env, "Khoa hoc May tinh"),
            &2026,
            &350, // GPA 3.50
        );

        assert_eq!(cert_id, 1);
        assert_eq!(client.total_certificates(), 1);

        // Xác minh chứng chỉ
        let cert = client.verify_certificate(&cert_id);
        assert_eq!(cert.student, student);
        assert_eq!(cert.graduation_year, 2026);
        assert_eq!(cert.gpa, 350);
    }

    #[test]
    fn test_get_cert_by_student() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        client.issue_certificate(
            &student,
            &String::from_str(&env, "Tran Thi B"),
            &String::from_str(&env, "Cu nhan Ky thuat Dien"),
            &String::from_str(&env, "Ky thuat Dien - Dien tu"),
            &2026,
            &380, // GPA 3.80
        );

        // Tra cứu theo địa chỉ sinh viên
        let cert = client.get_cert_by_student(&student);
        assert_eq!(cert.gpa, 380);
    }

    #[test]
    fn test_revoke_certificate() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        let cert_id = client.issue_certificate(
            &student,
            &String::from_str(&env, "Le Van C"),
            &String::from_str(&env, "Cu nhan Quan tri Kinh doanh"),
            &String::from_str(&env, "Quan tri Kinh doanh"),
            &2025,
            &300, // GPA 3.00
        );

        // Chưa thu hồi
        assert_eq!(client.is_revoked(&cert_id), false);

        // Thu hồi chứng chỉ
        client.revoke_certificate(&cert_id);

        // Đã thu hồi
        assert_eq!(client.is_revoked(&cert_id), true);
    }

    #[test]
    fn test_multiple_certificates() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Quoc Gia Ha Noi"),
        );

        let id1 = client.issue_certificate(
            &student1,
            &String::from_str(&env, "Pham Van D"),
            &String::from_str(&env, "Cu nhan Luat"),
            &String::from_str(&env, "Luat hoc"),
            &2026,
            &320,
        );

        let id2 = client.issue_certificate(
            &student2,
            &String::from_str(&env, "Hoang Thi E"),
            &String::from_str(&env, "Cu nhan Y khoa"),
            &String::from_str(&env, "Y khoa"),
            &2026,
            &390,
        );

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(client.total_certificates(), 2);

        let cert1 = client.verify_certificate(&id1);
        let cert2 = client.verify_certificate(&id2);
        assert_eq!(cert1.student, student1);
        assert_eq!(cert2.student, student2);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_duplicate_certificate_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        // Phát hành lần 1 - OK
        client.issue_certificate(
            &student,
            &String::from_str(&env, "Test Student"),
            &String::from_str(&env, "Cu nhan CNTT"),
            &String::from_str(&env, "CNTT"),
            &2026,
            &350,
        );

        // Phát hành lần 2 cho cùng sinh viên - PHẢI FAIL
        client.issue_certificate(
            &student,
            &String::from_str(&env, "Test Student"),
            &String::from_str(&env, "Cu nhan CNTT"),
            &String::from_str(&env, "CNTT"),
            &2026,
            &350,
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn test_invalid_gpa_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UniCertToken, ());
        let client = UniCertTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        client.initialize(
            &admin,
            &String::from_str(&env, "Dai Hoc Bach Khoa TP.HCM"),
        );

        // GPA > 4.00 (401) - PHẢI FAIL
        client.issue_certificate(
            &student,
            &String::from_str(&env, "Test Student"),
            &String::from_str(&env, "Cu nhan CNTT"),
            &String::from_str(&env, "CNTT"),
            &2026,
            &401,
        );
    }
}
