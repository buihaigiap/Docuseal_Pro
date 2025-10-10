# DocuSeal Pro API Documentation

## Tổng quan
DocuSeal Pro là hệ thống ký tài liệu điện tử với các tính năng:
- Tạo template từ PDF/DOCX
- Thêm fields (signature, text, image)
- Gửi email mời ký
- Ký tài liệu với chữ ký và upload ảnh
- Lưu trữ file trên S3/MinIO

## Authentication
API sử dụng JWT Bearer token cho authentication.

### 1. Đăng ký user
```bash
POST /api/auth/register
Content-Type: application/json

{
  "name": "Test User",
  "email": "test@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "User registered successfully",
  "data": {
    "id": 1,
    "name": "Test User",
    "email": "test@example.com",
    "role": "TeamMember",
    "created_at": "2025-10-10T09:00:00Z"
  }
}
```

### 2. Đăng nhập
```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "test@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "message": "Login successful",
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "user": {
      "id": 1,
      "name": "Test User",
      "email": "test@example.com",
      "role": "TeamMember"
    }
  }
}
```

## Templates API

### 1. Upload file (PDF/DOCX/Image)
```bash
POST /api/files/upload
Authorization: Bearer {token}
Content-Type: multipart/form-data

# File upload
curl -X POST "http://localhost:8080/api/files/upload" \
  -H "Authorization: Bearer {token}" \
  -F "file=@document.pdf"
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "File uploaded successfully",
  "data": {
    "id": "templates/1760088149_document.pdf",
    "filename": "document.pdf",
    "file_type": "pdf",
    "file_size": 163287,
    "url": "http://localhost:9000/docuseal/templates/1760088149_document.pdf",
    "content_type": "application/pdf",
    "uploaded_at": "2025-10-10T09:22:29Z"
  }
}
```

### 2. Tạo template từ file đã upload
```bash
POST /api/templates/from-file
Authorization: Bearer {token}
Content-Type: application/json

{
  "file_id": "templates/1760088149_document.pdf",
  "name": "Contract Template"
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "Template created from file successfully",
  "data": {
    "id": 4,
    "name": "Contract Template",
    "slug": "contract-template-1760088149",
    "user_id": 1,
    "template_fields": [],
    "documents": [
      {
        "filename": "document.pdf",
        "content_type": "application/pdf",
        "size": 163287,
        "url": "http://localhost:9000/docuseal/templates/1760088149_document.pdf"
      }
    ],
    "created_at": "2025-10-10T09:22:29Z",
    "updated_at": "2025-10-10T09:22:29Z"
  }
}
```

### 3. Thêm fields vào template
```bash
POST /api/templates/{template_id}/fields
Authorization: Bearer {token}
Content-Type: application/json

{
  "fields": [
    {
      "name": "buyer_signature",
      "field_type": "signature",
      "required": true,
      "display_order": 1,
      "position": {
        "x": 50.0,
        "y": 100.0,
        "width": 200.0,
        "height": 60.0,
        "page": 0
      }
    },
    {
      "name": "buyer_photo",
      "field_type": "image",
      "required": false,
      "display_order": 2,
      "position": {
        "x": 300.0,
        "y": 100.0,
        "width": 150.0,
        "height": 100.0,
        "page": 0
      }
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "Template fields created successfully",
  "data": [
    {
      "id": 18,
      "template_id": 4,
      "name": "buyer_signature",
      "field_type": "signature",
      "required": true,
      "display_order": 1,
      "position": {
        "x": 50.0,
        "y": 100.0,
        "width": 200.0,
        "height": 60.0,
        "page": 0
      },
      "options": null,
      "created_at": "2025-10-10T09:22:29Z",
      "updated_at": "2025-10-10T09:22:29Z"
    },
    {
      "id": 19,
      "template_id": 4,
      "name": "buyer_photo",
      "field_type": "image",
      "required": false,
      "display_order": 2,
      "position": {
        "x": 300.0,
        "y": 100.0,
        "width": 150.0,
        "height": 100.0,
        "page": 0
      },
      "options": null,
      "created_at": "2025-10-10T09:22:29Z",
      "updated_at": "2025-10-10T09:22:29Z"
    }
  ]
}
```

### 4. Lấy danh sách templates
```bash
GET /api/templates
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "message": "Templates retrieved successfully",
  "data": [
    {
      "id": 4,
      "name": "Contract Template",
      "slug": "contract-template-1760088149",
      "user_id": 1,
      "template_fields": null,
      "documents": [...],
      "created_at": "2025-10-10T09:22:29Z",
      "updated_at": "2025-10-10T09:22:29Z"
    }
  ]
}
```

### 5. Lấy chi tiết template với fields
```bash
GET /api/templates/{id}
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "message": "Template retrieved successfully",
  "data": {
    "id": 4,
    "name": "Contract Template",
    "slug": "contract-template-1760088149",
    "user_id": 1,
    "template_fields": [
      {
        "id": 18,
        "template_id": 4,
        "name": "buyer_signature",
        "field_type": "signature",
        "required": true,
        "display_order": 1,
        "position": {
          "x": 50.0,
          "y": 100.0,
          "width": 200.0,
          "height": 60.0,
          "page": 0
        },
        "options": null,
        "created_at": "2025-10-10T09:22:29Z",
        "updated_at": "2025-10-10T09:22:29Z"
      }
    ],
    "documents": [...],
    "created_at": "2025-10-10T09:22:29Z",
    "updated_at": "2025-10-10T09:22:29Z"
  }
}
```

## Submissions API

### 1. Tạo submission (gửi email mời ký)
```bash
POST /api/submissions
Authorization: Bearer {token}
Content-Type: application/json

{
  "template_id": 4,
  "submitters": [
    {
      "name": "Nguyễn Văn A",
      "email": "nguyenvana@gmail.com"
    },
    {
      "name": "Trần Thị B",
      "email": "tranthib@gmail.com"
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "Submission created successfully",
  "data": {
    "id": 4,
    "template_id": 4,
    "user_id": 1,
    "submitters": [
      {
        "id": 7,
        "template_id": 4,
        "user_id": 1,
        "name": "Nguyễn Văn A",
        "email": "nguyenvana@gmail.com",
        "status": "pending",
        "token": "abc123def456",
        "created_at": "2025-10-10T09:22:29Z",
        "updated_at": "2025-10-10T09:22:29Z"
      },
      {
        "id": 8,
        "template_id": 4,
        "user_id": 1,
        "name": "Trần Thị B",
        "email": "tranthib@gmail.com",
        "status": "pending",
        "token": "xyz789uvw123",
        "created_at": "2025-10-10T09:22:29Z",
        "updated_at": "2025-10-10T09:22:29Z"
      }
    ],
    "created_at": "2025-10-10T09:22:29Z",
    "updated_at": "2025-10-10T09:22:29Z"
  }
}
```

## Public Signing API (không cần authentication)

### 1. Lấy thông tin submitter và template
```bash
GET /public/submissions/{token}
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "message": "Submitter retrieved successfully",
  "data": {
    "id": 7,
    "template_id": 4,
    "user_id": 1,
    "name": "Nguyễn Văn A",
    "email": "nguyenvana@gmail.com",
    "status": "pending",
    "signed_at": null,
    "token": "abc123def456",
    "bulk_signatures": null,
    "created_at": "2025-10-10T09:22:29Z",
    "updated_at": "2025-10-10T09:22:29Z"
  }
}
```

### 2. Upload file cho signing (không cần auth)
```bash
POST /api/files/upload/public
Content-Type: multipart/form-data

# Upload image for signing
curl -X POST "http://localhost:8080/api/files/upload/public" \
  -F "file=@photo.png"
```

**Response:**
```json
{
  "success": true,
  "status_code": 201,
  "message": "File uploaded successfully",
  "data": {
    "id": "templates/1760088149_photo.png",
    "filename": "photo.png",
    "file_type": "image",
    "file_size": 1024,
    "url": "http://localhost:9000/docuseal/templates/1760088149_photo.png",
    "content_type": "image/png",
    "uploaded_at": "2025-10-10T09:22:29Z"
  }
}
```

### 3. Submit signatures
```bash
POST /public/signatures/bulk/{token}
Content-Type: application/json

{
  "signatures": [
    {
      "field_id": 18,
      "signature_value": "data:text/plain;base64,U2lnbmVkIGJ5IE5ndXnhurVuIFbEg24gQQ=="
    },
    {
      "field_id": 19,
      "signature_value": "http://localhost:9000/docuseal/templates/1760088149_photo.png"
    }
  ],
  "ip_address": "127.0.0.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Response:**
```json
{
  "success": true,
  "status_code": 200,
  "message": "Signatures submitted successfully",
  "data": {
    "signed_at": "2025-10-10T09:22:29Z",
    "bulk_signatures": [
      {
        "field_id": 18,
        "field_name": "buyer_signature",
        "signature_value": "data:text/plain;base64,U2lnbmVkIGJ5IE5ndXnhurVuIFbEg24gQQ=="
      },
      {
        "field_id": 19,
        "field_name": "buyer_photo",
        "signature_value": "http://localhost:9000/docuseal/templates/1760088149_photo.png"
      }
    ]
  }
}
```

## Response Format
Tất cả API responses đều có format thống nhất:

```json
{
  "success": true|false,
  "status_code": 200,
  "message": "Human readable message",
  "data": { ... } | [...],
  "error": null | "error message"
}
```

## Error Codes
- `400` - Bad Request (dữ liệu đầu vào không hợp lệ)
- `401` - Unauthorized (chưa đăng nhập)
- `403` - Forbidden (không có quyền)
- `404` - Not Found (không tìm thấy resource)
- `500` - Internal Server Error (lỗi server)

## Field Types
- `signature` - Chữ ký điện tử (text base64)
- `text` - Text input
- `image` - Upload ảnh (S3 URL)

## File Storage
- Files được lưu trên MinIO S3 compatible storage
- Public URLs có format: `http://localhost:9000/{bucket}/{key}`
- Bucket mặc định: `docuseal`
- Anonymous read access được enable

## Testing Scripts
- `run_full_test.ps1` - Workflow hoàn chỉnh từ tạo template đến ký
- `sign_simple.ps1` - Script ký tài liệu với token

## Swagger Documentation
Truy cập: `http://localhost:8080/swagger-ui/` để xem API documentation tương tác.</content>
<parameter name="filePath">/home/giap/giap/Docuseal_Pro/API_DOCUMENTATION.md