# Tài Liệu Dự Án DocuSeal

## Giới Thiệu

DocuSeal là một nền tảng mã nguồn mở cung cấp khả năng ký và xử lý tài liệu số một cách an toàn và hiệu quả. Dự án cho phép tạo biểu mẫu PDF để điền và ký trực tuyến trên mọi thiết bị với công cụ web dễ sử dụng, tối ưu hóa cho di động.
DocuSeal cung cấp các tính năng cơ bản sau:

- **Trình Tạo Trường Biểu Mẫu PDF (WYSIWYG)**: Tạo biểu mẫu PDF với giao diện kéo-thả.
- **12 Loại Trường Khác Nhau**: Bao gồm Chữ Ký, Ngày Tháng, Tệp, Hộp Kiểm, v.v.
- **Nhiều Người Gửi Mỗi Tài Liệu**: Hỗ trợ nhiều người ký trên cùng một tài liệu.
- **Email Tự Động Qua SMTP**: Gửi thông báo qua email.
- **Lưu Trữ Tệp**: Trên đĩa hoặc đám mây (AWS S3, Google Storage, Azure Cloud).
- **Ký Điện Tử PDF Tự Động**: Tích hợp ký điện tử.
- **Xác Minh Chữ Ký PDF**: Kiểm tra tính hợp lệ của chữ ký.
- **Quản Lý Người Dùng**: Hệ thống quản lý tài khoản.
- **Tối Ưu Cho Di Động**: Giao diện thân thiện trên thiết bị di động.
- **Đa Ngôn Ngữ**: Giao diện hỗ trợ 6 ngôn ngữ, ký hỗ trợ 14 ngôn ngữ.
- **API và Webhooks**: Tích hợp với các hệ thống khác.
- **Dễ Triển Khai**: Triển khai nhanh chóng trong vài phút.

## Tính Năng Pro

Phiên bản Pro cung cấp các tính năng nâng cao:

- **Logo Công Ty và White-Label**: Tùy chỉnh thương hiệu.
- **Vai Trò Người Dùng**: Phân quyền chi tiết.
- **Nhắc Nhở Tự Động**: Gửi nhắc nhở ký tài liệu.
- **Xác Minh Danh Tính Qua SMS**: Xác thực qua tin nhắn.
- **Trường Điều Kiện và Công Thức**: Logic động trong biểu mẫu.
- **Gửi Hàng Loạt Với CSV/XLSX**: Nhập dữ liệu từ bảng tính.
- **SSO / SAML**: Đăng nhập đơn.
- **Tạo Mẫu Từ HTML API**: [Hướng Dẫn](https://www.docuseal.com/guides/create-pdf-document-fillable-form-with-html-api)
- **Tạo Mẫu Từ PDF hoặc DOCX Với Thẻ Trường API**: [Hướng Dẫn](https://www.docuseal.com/guides/use-embedded-text-field-tags-in-the-pdf-to-create-a-fillable-form)
- **Biểu Mẫu Ký Nhúng**: Hỗ trợ React, Vue, Angular, JavaScript ([Tài Liệu](https://www.docuseal.com/docs/embedded))
- **Trình Tạo Biểu Mẫu Tài Liệu Nhúng**: Tương tự cho việc tạo biểu mẫu.

## Triển Khai

DocuSeal có thể được triển khai dễ dàng qua Docker hoặc các nền tảng đám mây.

### Docker

Chạy lệnh sau để triển khai:

```bash
docker run --name docuseal -p 3000:3000 -v .:/data docuseal/docuseal
```

Theo mặc định, sử dụng cơ sở dữ liệu SQLite. Có thể sử dụng PostgreSQL hoặc MySQL bằng biến môi trường `DATABASE_URL`.

### Docker Compose

Tải file docker-compose.yml:

```bash
curl https://raw.githubusercontent.com/docusealco/docuseal/master/docker-compose.yml > docker-compose.yml
```

Chạy với tên miền tùy chỉnh qua HTTPS:

```bash
sudo HOST=your-domain-name.com docker compose up
```

### Các Nền Tảng Khác

- **Heroku**: [Triển Khai Trên Heroku](https://heroku.com/deploy?template=https://github.com/docusealco/docuseal)
- **Railway**: [Triển Khai Trên Railway](https://railway.app/template/docuseal)
- **DigitalOcean**: [Triển Khai Trên DigitalOcean](https://marketplace.digitalocean.com/apps/docuseal)
- **Render**: [Triển Khai Trên Render](https://render.com/deploy?repo=https://render.com/deploy?repo=https://github.com/docusealco/docuseal)

## Triển Khai Bằng Rust

Dự án DocuSeal gốc được xây dựng bằng Ruby on Rails và Vue.js. Để triển khai và phát triển phiên bản bằng Rust, chúng ta có thể sử dụng các framework và thư viện Rust phù hợp để tái tạo chức năng tương tự. Phần này cung cấp hướng dẫn triển khai và phát triển DocuSeal bằng Rust.

### Kiến Trúc Tổng Quan

- **Backend**: Sử dụng Actix-web (hoặc Rocket) cho web server async.
- **Database**: Diesel (cho PostgreSQL/SQLite) hoặc SeaORM (async ORM).
- **PDF Processing**: Thư viện như `lopdf` cho xử lý PDF, `wkhtmltopdf` cho chuyển HTML sang PDF.
- **Ký Điện Tử**: Sử dụng `ring` cho cryptography, hoặc crate như `pdf-sign` cho ký PDF.
- **Email**: Crate `lettre` cho gửi email qua SMTP.
- **Storage**: AWS SDK cho Rust để lưu trữ đám mây.
- **Frontend**: Giữ Vue.js, hoặc chuyển sang Yew (Rust web framework) cho frontend hoàn toàn Rust.
- **Authentication**: JWT hoặc session-based với thư viện như `jsonwebtoken`.

### Thiết Lập Dự Án Rust

1. **Cài Đặt Rust**: Đảm bảo có Rust toolchain.

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Tạo Dự Án Mới**:

   ```bash
   cargo new docuseal-rust
   cd docuseal-rust
   ```

3. **Thêm Dependencies vào Cargo.toml**:

   ```toml
   [dependencies]
   actix-web = "4"
   actix-files = "0.6"
   diesel = { version = "2.0", features = ["postgres", "r2d2"] }
   dotenvy = "0.15"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   tokio = { version = "1", features = ["full"] }
   lettre = "0.10"
   lopdf = "0.29"
   ring = "0.16"
   aws-sdk-s3 = "0.25"
   jsonwebtoken = "8"
   ```

4. **Cấu Hình Database**:

   - Sử dụng Diesel CLI để thiết lập migrations.

     ```bash
     cargo install diesel_cli --no-default-features --features postgres
     diesel setup
     ```

   - Tạo migrations cho các bảng như users, templates, submissions, v.v.

5. **Cấu Trúc Dự Án**:

   ```
   src/
   ├── main.rs
   ├── models/
   │   ├── mod.rs
   │   ├── user.rs
   │   ├── template.rs
   │   └── submission.rs
   ├── handlers/
   │   ├── mod.rs
   │   ├── templates.rs
   │   ├── submissions.rs
   │   └── submitters.rs
   ├── schema.rs (tạo bởi Diesel)
   └── lib.rs
   ```

### Lưu Trữ Files

Để liệt kê và quản lý các file mẫu (templates) của user, DocuSeal lưu trữ:

- **Metadata của Templates**: Lưu trong database (PostgreSQL/SQLite) thông qua Diesel models. Bao gồm thông tin như tên, slug, fields, submitters, v.v.
- **Files Tài Liệu (PDF/DOCX/HTML)**: Lưu trên đĩa cục bộ hoặc đám mây.
  - **Đĩa cục bộ**: Sử dụng thư mục như `./uploads/templates/` hoặc `./data/files/`.
  - **Đám mây**: AWS S3, Google Cloud Storage, hoặc Azure Blob Storage sử dụng AWS SDK hoặc tương tự.

**Cấu hình trong .env**:

```env
DATABASE_URL=postgres://user:password@localhost/docuseal
STORAGE_TYPE=disk  # hoặc s3, gcs, azure
STORAGE_PATH=./data/files  # cho disk
AWS_ACCESS_KEY_ID=your_key
AWS_SECRET_ACCESS_KEY=your_secret
AWS_REGION=us-east-1
AWS_S3_BUCKET=docuseal-bucket
```

**Ví dụ code lưu file** (trong handler):

```rust
use std::fs;
use aws_sdk_s3::Client;

async fn save_file(file: &actix_multipart::Multipart, storage_type: &str) -> Result<String, Box<dyn std::error::Error>> {
    match storage_type {
        "disk" => {
            let path = format!("./data/files/{}", file.filename);
            fs::copy(file.path, &path)?;
            Ok(path)
        }
        "s3" => {
            let client = Client::new(&aws_config::load_from_env().await);
            let key = format!("templates/{}", file.filename);
            client.put_object()
                .bucket("docuseal-bucket")
                .key(&key)
                .body(file.path.into())
                .send()
                .await?;
            Ok(format!("s3://docuseal-bucket/{}", key))
        }
        _ => Err("Unsupported storage type".into())
    }
}
```

Khi liệt kê templates qua API GET /templates, truy vấn database để lấy metadata, và URL của files từ storage.

### Ví Dụ Code Cơ Bản

**main.rs**:

```rust
use actix_web::{web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

mod handlers;
mod models;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(handlers::templates::get_templates)
            .service(handlers::submissions::create_submission)
            // Thêm các route khác
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**handlers/templates.rs** (Ví dụ):

```rust
use actix_web::{get, web, HttpResponse, Result};
use diesel::prelude::*;
use crate::models::Template;
use crate::schema::templates;
use crate::DbPool;

#[get("/templates")]
async fn get_templates(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let results = templates::table
        .load::<Template>(&mut conn)
        .expect("Error loading templates");

    Ok(HttpResponse::Ok().json(results))
}
```

### Triển Khai

- **Build và Chạy**:

  ```bash
  cargo build --release
  ./target/release/docuseal-rust
  ```

- **Docker cho Rust**:

  Tạo Dockerfile:

  ```dockerfile
  FROM rust:1.70 as builder
  WORKDIR /usr/src/docuseal
  COPY . .
  RUN cargo build --release

  FROM debian:bullseye-slim
  COPY --from=builder /usr/src/docuseal/target/release/docuseal-rust /usr/local/bin/
  CMD ["docuseal-rust"]
  ```

- **Triển Khai Đám Mây**: Sử dụng Heroku, Railway, v.v. với buildpack Rust hoặc Docker.

### Lưu Ý Phát Triển

- **Bảo Mật**: Sử dụng HTTPS, validate input, tránh SQL injection với Diesel.
- **Testing**: Sử dụng `cargo test` và thư viện như `actix-test`.
- **Performance**: Rust rất nhanh, phù hợp cho xử lý PDF và ký điện tử.
- **Tích Hợp Frontend**: Sử dụng API từ Rust backend với Vue.js frontend.

Phiên bản Rust này sẽ cung cấp hiệu suất cao hơn và an toàn bộ nhớ so với phiên bản gốc. Để triển khai đầy đủ, cần implement tất cả API endpoints và logic nghiệp vụ.

### Lộ Trình Phát Triển Theo Các Phần

Để phát triển DocuSeal bằng Rust một cách có hệ thống, chia nhỏ dự án thành các phần (issues) sau, làm từng phần một:

1. **Thiết Lập Cấu Trúc Dự Án Rust**: Tạo project với Cargo.toml, thêm dependencies cơ bản (Actix-web, Diesel, v.v.), thiết lập cấu trúc thư mục.
2. **Cấu Hình Database và Migrations**: Thiết lập kết nối database, tạo migrations cho bảng users, templates, submissions, submitters.
3. **Implement Models Database**: Định nghĩa models Rust cho User, Template, Submission, Submitter sử dụng Diesel derive.
4. **Implement Authentication**: Thêm xác thực API với token (X-Auth-Token) như trong OpenAPI spec. Dự án này không yêu cầu đăng ký/đăng nhập user; sử dụng token tĩnh hoặc external auth để bảo mật endpoints.
5. **Implement Handlers Templates**: Tạo handlers cho API Templates (GET, POST, PUT, DELETE /templates và các endpoint liên quan).
6. **Implement Handlers Submissions**: Tạo handlers cho API Submissions, bao gồm xử lý PDF và tạo submissions.
7. **Implement Handlers Submitters**: Tạo handlers cho API Submitters, cập nhật fields và status.
8. **Implement Hệ Thống Lưu Trữ Files**: Thiết lập lưu trữ files trên disk hoặc cloud (S3), upload/download documents.
9. **Tích Hợp Gửi Email**: Sử dụng lettre để gửi thông báo qua SMTP.
10. **Implement Ký PDF**: Thêm logic ký điện tử và xác minh chữ ký PDF.
11. **Thiết Lập Frontend**: Tích hợp Vue.js hoặc chuyển sang Yew, kết nối với API Rust.
12. **Viết Tests**: Tạo unit tests và integration tests cho models và handlers.
13. **Triển Khai và Dockerization**: Tạo Dockerfile, scripts triển khai production.

Mỗi phần nên được hoàn thành và test riêng trước khi chuyển sang phần tiếp theo để đảm bảo tiến độ ổn định.

## API Chính

DocuSeal cung cấp API RESTful để tích hợp. Dưới đây là các endpoint chính từ OpenAPI spec:

### Bảo Mật
- Sử dụng `X-Auth-Token` trong header cho xác thực.

### Các Nhóm API

#### Templates (Mẫu)
- **GET /templates**: Liệt kê tất cả mẫu.
- **GET /templates/{id}**: Lấy thông tin mẫu.
- **DELETE /templates/{id}**: Lưu trữ mẫu.
- **PUT /templates/{id}**: Cập nhật mẫu.
- **PUT /templates/{id}/documents**: Cập nhật tài liệu mẫu.
- **POST /templates/{id}/clone**: Nhân bản mẫu.
- **POST /templates/html**: Tạo mẫu từ HTML.
- **POST /templates/docx**: Tạo mẫu từ DOCX.
- **POST /templates/pdf**: Tạo mẫu từ PDF.
- **POST /templates/merge**: Hợp nhất mẫu.

#### Submissions (Gửi Tài Liệu)
- **GET /submissions**: Liệt kê tất cả gửi.
- **POST /submissions**: Tạo gửi mới.
- **GET /submissions/{id}**: Lấy thông tin gửi.
- **DELETE /submissions/{id}**: Lưu trữ gửi.
- **GET /submissions/{id}/documents**: Lấy tài liệu gửi.
- **POST /submissions/emails**: Tạo gửi từ email.
- **POST /submissions/pdf**: Tạo gửi từ PDF.
- **POST /submissions/docx**: Tạo gửi từ DOCX.
- **POST /submissions/html**: Tạo gửi từ HTML.

#### Submitters (Người Gửi)
- **GET /submitters**: Liệt kê tất cả người gửi.
- **GET /submitters/{id}**: Lấy thông tin người gửi.
- **PUT /submitters/{id}**: Cập nhật người gửi.

### Ví Dụ Sử Dụng API

Để tạo một submission mới:

```json
POST /submissions
{
  "template_id": 1000001,
  "submitters": [
    {
      "email": "signer@example.com",
      "name": "John Doe"
    }
  ]
}
