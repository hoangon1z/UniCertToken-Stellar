# University Certificate Token (UniCertToken)

## Vấn Đề
Bằng đại học / chứng chỉ truyền thống dễ bị làm giả, khó xác minh nhanh chóng bởi nhà tuyển dụng và có nguy cơ bị thất lạc bản vật lý.

## Giải Pháp
UniCertToken là một dApp trên Stellar/Soroban cho phép các trường đại học phát hành chứng chỉ kỹ thuật số dưới dạng record on-chain. Mỗi chứng chỉ là **duy nhất, không thể làm giả** và có thể được **xác minh tức thì** bởi bất kỳ ai thông qua Smart Contract — không cần liên hệ văn phòng trường.

## Tại Sao Stellar
Stellar cung cấp tốc độ giao dịch cực nhanh (~5s) và phí cực thấp (~$0.000003), phù hợp cho việc phát hành và xác minh hàng triệu chứng chỉ sinh viên một cách hiệu quả và minh bạch. Không thể giả mạo vì dữ liệu được ghi trên blockchain công khai.

## Người Dùng Mục Tiêu
- **Các trường đại học:** Phát hành bằng tốt nghiệp minh bạch, không làm giả được.
- **Sinh viên:** Sở hữu chứng chỉ số vĩnh viễn trong ví blockchain của mình.
- **Nhà tuyển dụng:** Xác minh hồ sơ ứng viên ngay lập tức mà không cần liên hệ văn phòng trường.

## Tính Năng
| Tính năng | Mô tả |
|---|---|
| `issue_certificate` | Admin (trường) phát hành chứng chỉ cho sinh viên |
| `verify_certificate` | Bất kỳ ai xác minh chứng chỉ theo ID |
| `get_cert_by_student` | Tra cứu chứng chỉ theo địa chỉ ví sinh viên |
| `revoke_certificate` | Admin thu hồi chứng chỉ (nếu phát hiện gian lận) |
| `is_revoked` | Kiểm tra trạng thái thu hồi |
| `total_certificates` | Tổng số chứng chỉ đã phát |
| `university_name` | Tên trường đại học đã đăng ký |

## Demo Trực Tiếp
- **Mạng**: Stellar Testnet
- **Contract ID**: `CAGPJRUOY73BORPTPGUV6AQJQCMMTSQ3BCWW37OPLPEKCNSZNI7I7TMI`
- **Giao dịch phát hành mẫu**: [0d455502...](https://stellar.expert/explorer/testnet/tx/0d455502441fd21f9b5ec4c236298fb1078f275009edd379abac7dd61031438e)
- **Contract trên Explorer**: [stellar.expert →](https://stellar.expert/explorer/testnet/contract/CAGPJRUOY73BORPTPGUV6AQJQCMMTSQ3BCWW37OPLPEKCNSZNI7I7TMI)

## Chạy Giao Diện Frontend

```bash
cd contracts/uni-cert-token/frontend
npx serve .
# Mở http://localhost:3000
```

## Build & Test Contract

```bash
cd contracts/uni-cert-token

# Build WASM
stellar contract build

# Chạy 7 test cases
cargo test
```

## Deploy Lại (nếu cần)

```bash
# Từ thư mục contracts/uni-cert-token
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/uni_cert_token.wasm \
  --source-account student \
  --network testnet

# Khởi tạo contract sau khi deploy
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account student \
  --network testnet \
  -- initialize \
  --admin $(stellar keys address student) \
  --uni_name "Hoc Vien Cong Nghe Buu Chinh Vien Thong"
```

## Tech Stack
- **Smart Contract**: Rust / Soroban SDK v22
- **Frontend**: HTML / Vanilla JS / Stellar SDK v12 / Freighter API v2
- **Deployment Tools**: Stellar CLI
- **Mạng**: Stellar Testnet

## Nhóm
- **Tran Hoang** | PTIT | Rise In x Stellar University Tour 2026
