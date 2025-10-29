# Deployment Guide - Frontend & Backend on Port 8080

## Cấu hình đã thực hiện

### 1. Frontend (app/docuseal)
- **vite.config.ts**: Cấu hình build output vào thư mục `dist` và inject environment variables
- **axiosClient.ts**: API_BASE_URL đọc trực tiếp từ `import.meta.env.VITE_API_BASE_URL`
- **vite-env.d.ts**: TypeScript definitions cho Vite environment variables
- **.env**: File cấu hình environment variables (không commit vào git)
- **.env.example**: File mẫu để tham khảo

### 2. Backend (src/main.rs)
- Thêm `tower_http::services::ServeDir` để serve static files
- Serve frontend build từ `app/docuseal/dist`
- Fallback service để hỗ trợ client-side routing (React Router)
- Tất cả routes đều chạy trên cổng 8080:
  - `/` - Frontend
  - `/api/*` - Backend API
  - `/swagger-ui` - API Documentation

## Cấu hình Environment Variables

### Backend (.env ở root)
File `.env` chính của backend chứa tất cả config:
```env
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/db

# Server
PORT=8080
Domain_URL=http://localhost:8080

# JWT & Auth
JWT_SECRET=your-secret-key

# Storage (S3/MinIO)
STORAGE_TYPE=s3
STORAGE_ENDPOINT=http://localhost:9000
# ... các config khác
```

### Frontend (.env trong app/docuseal/)

**1. Copy file mẫu:**
```bash
cd app/docuseal
cp .env.example .env
```

**2. Cấu hình theo môi trường:**

**Production (FE + BE cùng port 8080):**
```env
# Để trống hoặc comment out
# VITE_API_BASE_URL=
```

**Development (FE port 3000, BE port 8080):**
```env
# Sử dụng Domain_URL từ backend .env
VITE_API_BASE_URL=http://localhost:8080
```

**Production với custom domain:**
```env
VITE_API_BASE_URL=https://yourdomain.com
```

## Cách chạy

### Production Mode (FE + BE cùng port 8080)

**Option 1: Sử dụng script tự động**
```bash
./build_and_run.sh
```
Script này sẽ:
- Kiểm tra và tạo frontend `.env` nếu chưa có
- Install dependencies nếu cần
- Build frontend
- Chạy backend với frontend static files

**Option 2: Chạy thủ công từng bước**
```bash
# 1. Setup frontend .env (nếu chưa có)
cd app/docuseal
cp .env.example .env
# Edit .env - để trống VITE_API_BASE_URL cho production

# 2. Build frontend
npm install
npm run build
cd ../..

# 3. Run backend (sẽ serve frontend static files)
cargo run --release
```

**Truy cập ứng dụng:**
- 🌐 Frontend: http://localhost:8080
- 🔌 API: http://localhost:8080/api
- 📚 Swagger UI: http://localhost:8080/swagger-ui

### Development Mode (FE riêng với hot reload)

**Option 1: Sử dụng script setup**
```bash
./dev.sh
```
Script này sẽ:
- Tạo frontend `.env` với `VITE_API_BASE_URL=http://localhost:8080`
- Hiển thị hướng dẫn chạy FE và BE riêng
- Có thể chạy backend ngay

**Option 2: Chạy thủ công**

Terminal 1 - Backend:
```bash
cargo run
```

Terminal 2 - Frontend (với hot reload):
```bash
cd app/docuseal

# Setup .env cho dev mode (chỉ cần 1 lần)
cat > .env << 'EOF'
VITE_API_BASE_URL=http://localhost:8080
EOF

npm install
npm run dev
```

**Truy cập:**
- 🌐 Frontend: http://localhost:3000 (hot reload)
- 🔌 Backend API: http://localhost:8080/api
- 📚 Swagger UI: http://localhost:8080/swagger-ui

## Lưu ý

- Mỗi khi thay đổi code frontend, cần rebuild bằng `npm run build` trong thư mục `app/docuseal`
- Backend tự động serve file static từ `app/docuseal/dist`
- Fallback service đảm bảo client-side routing hoạt động (tất cả routes không phải API sẽ trả về index.html)
- CORS đã được cấu hình permissive cho development

## Docker Deployment (Optional)

Tạo file `Dockerfile` để deploy:
```dockerfile
# Build frontend
FROM node:18 AS frontend-builder
WORKDIR /app/frontend
COPY app/docuseal/package*.json ./
RUN npm install
COPY app/docuseal ./
RUN npm run build

# Build backend
FROM rust:1.75 AS backend-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY build.rs ./
COPY include ./include
COPY lib ./lib
RUN cargo build --release

# Final image
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=backend-builder /app/target/release/docuseal_pro ./
COPY --from=frontend-builder /app/frontend/dist ./app/docuseal/dist
COPY lib ./lib
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
EXPOSE 8080
CMD ["./docuseal_pro"]
```
