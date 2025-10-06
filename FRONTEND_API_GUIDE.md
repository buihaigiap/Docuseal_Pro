# DocuSeal Pro API Documentation - Frontend Implementation Guide

## Tổng quan

Tài liệu này phân tích các API được sử dụng trong `run_full_test.ps1` và `sign_simple.ps1` để hướng dẫn implement frontend.

## 1. Authentication APIs

### 1.1 POST /api/auth/register
**Mục đích:** Đăng ký tài khoản mới

**Request:**
```javascript
POST /api/auth/register
Content-Type: application/json

{
  "name": "string",
  "email": "string",
  "password": "string"
}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 201,
  "message": "User registered successfully",
  "data": {
    "id": 1,
    "name": "string",
    "email": "string"
  }
}
```

**Frontend Usage:**
```javascript
const registerUser = async (name, email, password) => {
  const response = await fetch('/api/auth/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, email, password })
  });
  return response.json();
};
```

### 1.2 POST /api/auth/login
**Mục đích:** Đăng nhập và nhận JWT token

**Request:**
```javascript
POST /api/auth/login
Content-Type: application/json

{
  "email": "string",
  "password": "string"
}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 200,
  "message": "Login successful",
  "data": {
    "token": "jwt_token_here",
    "user": {
      "id": 1,
      "name": "string",
      "email": "string"
    }
  }
}
```

**Frontend Usage:**
```javascript
const loginUser = async (email, password) => {
  const response = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  const data = await response.json();
  if (data.success) {
    localStorage.setItem('token', data.data.token);
  }
  return data;
};
```

## 2. Template Management APIs

### 2.1 POST /api/templates/pdf
**Mục đích:** Upload PDF và tạo template

**Request:**
```javascript
POST /api/templates/pdf
Content-Type: multipart/form-data
Authorization: Bearer {jwt_token}

FormData:
- pdf: File (PDF file)
- name: string (Template name)
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 201,
  "message": "Template created successfully",
  "data": {
    "id": 1,
    "name": "Contract Template",
    "file_url": "string",
    "created_at": "2025-10-06T10:00:00Z"
  }
}
```

**Frontend Usage:**
```javascript
const createTemplateFromPDF = async (pdfFile, templateName) => {
  const formData = new FormData();
  formData.append('pdf', pdfFile);
  formData.append('name', templateName);

  const response = await fetch('/api/templates/pdf', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    },
    body: formData
  });
  return response.json();
};
```

### 2.2 POST /api/templates/{templateId}/fields
**Mục đích:** Tạo signature field với vị trí

**Request:**
```javascript
POST /api/templates/{templateId}/fields
Content-Type: application/json
Authorization: Bearer {jwt_token}

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
}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 201,
  "message": "Template field created successfully",
  "data": {
    "id": 1,
    "name": "buyer_signature",
    "field_type": "signature",
    "required": true,
    "position": {
      "x": 50.0,
      "y": 100.0,
      "width": 200.0,
      "height": 60.0,
      "page": 0
    }
  }
}
```

### 2.3 GET /api/templates/{id}/full-info
**Mục đích:** Lấy thông tin đầy đủ của template bao gồm tất cả submitters

**Request:**
```javascript
GET /api/templates/{templateId}/full-info
Authorization: Bearer {jwt_token}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 200,
  "message": "Template full information retrieved successfully",
  "data": {
    "submitters": [
      {
        "created_at": "2025-10-06T09:11:37.359230Z",
        "email": "buihaigiap0101@gmail.com",
        "id": 8,
        "name": "Bui Hai Giap",
        "signed_at": null,
        "status": "pending",
        "template_id": 5,
        "token": "ottsJCaNwIutbeGg/SU19vFD4YUTtfKUM2oB3fxm1TY=",
        "updated_at": "2025-10-06T09:11:37.359230Z",
        "user_id": 5
      },
      {
        "bulk_signatures": [
          {
            "field_id": 13,
            "field_name": "buyer_signature",
            "signature_value": "data:text/plain;base64,U2lnbmVkIGJ5IFRlc3QgVXNlciAyIC0gRmllbGQgMTMgLSAyMDI1LTEwLTA2IDE2OjExOjU4"
          },
          {
            "field_id": 14,
            "field_name": "seller_signature",
            "signature_value": "data:text/plain;base64,U2lnbmVkIGJ5IFRlc3QgVXNlciAyIC0gRmllbGQgMTQgLSAyMDI1LTEwLTA2IDE2OjExOjU4"
          },
        ],
        "created_at": "2025-10-06T09:11:42.556036Z",
        "email": "buihaigiap0102@gmail.com",
        "id": 9,
        "name": "Test User 2",
        "signed_at": "2025-10-06T09:11:58.846110Z",
        "status": "signed",
        "template_id": 5,
        "token": "/5y37adrf6hzVzzlxoIi76d0g/aeQnphzlVyEz3roaw=",
        "updated_at": "2025-10-06T09:11:58.846110Z",
        "user_id": 5
      }
    ],
    "template": {
      "created_at": "2025-10-06T09:11:37.275931Z",
      "documents": [
        {
          "content_type": "application/pdf",
          "filename": "test.pdf",
          "size": 0,
          "url": "templates/1759741897_test.pdf"
        }
      ],
      "id": 5,
      "name": "Contract Template 1759741893",
      "slug": "pdf-contract-template-1759741893-1759741897",
      "updated_at": "2025-10-06T09:11:37.275931Z",
      "user_id": 5
    },
    "total_submitters": 2
  },
}
```

**Frontend Usage:**
```javascript
const getTemplateFullInfo = async (templateId) => {
  const response = await fetch(`/api/templates/${templateId}/full-info`, {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
  });
  return response.json();
};
```

**Use Case:** Hiển thị dashboard với thông tin template và trạng thái ký của tất cả submitters.

**Khi nào dùng:** Khi cần xem tổng quan submission cho một template (ví dụ: trang dashboard, trang chi tiết template).

**Khác với GET /api/templates/{id}:**
- `/api/templates/{id}`: Chỉ trả về thông tin template
- `/api/templates/{id}/full-info`: Trả về template + tất cả submitters + thống kê

## 3. Submission Management APIs

### 3.1 POST /api/submissions
**Mục đích:** Tạo submission và gửi email mời ký

**Request:**
```javascript
POST /api/submissions
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "template_id": 1,
  "submitters": [
    {
      "name": "John Doe",
      "email": "john@example.com"
    },
    {
      "name": "Jane Smith",
      "email": "jane@example.com"
    }
  ]
}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 201,
  "message": "Submission created successfully",
  "data": {
    "id": 1,
    "template_id": 1,
    "user_id": 1,
    "status": "pending",
    "submitters": [
      {
        "id": 1,
        "name": "John Doe",
        "email": "john@example.com",
        "token": "unique_token_1",
        "status": "pending"
      },
      {
        "id": 2,
        "name": "Jane Smith",
        "email": "jane@example.com",
        "token": "unique_token_2",
        "status": "pending"
      }
    ]
  }
}
```

**Frontend Usage:**
```javascript
const createSubmission = async (templateId, submitters) => {
  const response = await fetch('/api/submissions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    },
    body: JSON.stringify({ template_id: templateId, submitters })
  });
  return response.json();
};
```

## 4. Public APIs (For Signing)

### 4.1 GET /public/submitters/{token}
**Mục đích:** Lấy thông tin submitter để hiển thị form ký

**Request:**
```javascript
GET /public/submitters/{token}
Authorization: Bearer {jwt_token}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 200,
  "message": "Submitter retrieved successfully",
  "data": {
    "id": 1,
    "template_id": 1,
    "user_id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "status": "pending",
    "token": "unique_token",
     "bulk_signatures": [
      {
        "field_id": 13,
        "field_name": "buyer_signature",
        "signature_value": "data:text/plain;base64,U2lnbmVkIGJ5IFRlc3QgVXNlciAyIC0gRmllbGQgMTMgLSAyMDI1LTEwLTA2IDE2OjExOjU4"
      },
     ]
    "created_at": "2025-10-06T10:00:00Z"
  }
}
```

**Frontend Usage:**
```javascript
const getSubmitterInfo = async (token) => {
  const response = await fetch(`/public/submitters/${token}`, {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
  });
  return response.json();
};
```

### 4.2 POST /public/signatures/bulk/{token}
**Mục đích:** Ký nhiều field cùng lúc

**Request:**
```javascript
POST /public/signatures/bulk/{token}
Content-Type: application/json
Authorization: Bearer {jwt_token}

{
  "signatures": [
    {
      "field_id": 1,
      "signature_value": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
    },
    {
      "field_id": 2,
      "signature_value": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
    }
  ],
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Response:**
```javascript
{
  "success": true,
  "status_code": 200,
  "message": "Bulk signatures submitted successfully",
  "data": {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "status": "completed",
    "signed_at": "2025-10-06T10:30:00Z",
    "bulk_signatures": [
      {
        "field_id": 1,
        "field_name": "buyer_signature",
        "signature_value": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
      }
    ]
  }
}
```

**Frontend Usage:**
```javascript
const submitSignatures = async (token, signatures) => {
  const response = await fetch(`/public/signatures/bulk/${token}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    },
    body: JSON.stringify({
      signatures,
      ip_address: window.location.hostname,
      user_agent: navigator.userAgent
    })
  });
  return response.json();
};
```

## 4. File Access APIs

### 4.1 GET /api/files/{key}
**Mục đích:** Download/xem file từ S3 storage (PDF, DOCX, HTML)

**Request:**
```javascript
GET /api/files/{key}
Authorization: Bearer {jwt_token}
```

**Parameters:**
- `key`: File key từ template response (ví dụ: "templates/1759741897_test.pdf")

**Response:**
- Trả về file binary data với content-type phù hợp
- PDF files: `application/pdf` với `Content-Disposition: inline`
- Có thể mở trực tiếp trong browser hoặc download

**Frontend Usage - Mở PDF trong iframe:**
```javascript
const viewPDFFile = async (fileKey) => {
  const token = localStorage.getItem('token');
  if (!token) {
    throw new Error('No authentication token found');
  }

  // Tạo URL với authorization header
  const pdfUrl = `/api/files/${fileKey}`;
  
  // Mở trong iframe hoặc new window
  const iframe = document.createElement('iframe');
  iframe.src = pdfUrl;
  iframe.style.width = '100%';
  iframe.style.height = '600px';
  iframe.style.border = '1px solid #ccc';
  
  // Thêm authorization header cho iframe
  // Note: iframe không thể set custom headers, cần proxy hoặc presigned URL
  
  document.getElementById('pdf-container').appendChild(iframe);
  return iframe;
};
```

**Frontend Usage - Download file:**
```javascript
const downloadFile = async (fileKey, filename) => {
  const token = localStorage.getItem('token');
  if (!token) {
    throw new Error('No authentication token found');
  }

  const response = await fetch(`/api/files/${fileKey}`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });

  if (!response.ok) {
    throw new Error('Failed to download file');
  }

  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  
  const a = document.createElement('a');
  a.href = url;
  a.download = filename || 'download';
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(url);
  document.body.removeChild(a);
};
```

**Frontend Usage - Hiển thị PDF trong React/Vue component:**
```javascript
// React component
const PDFViewer = ({ fileKey }) => {
  const [pdfUrl, setPdfUrl] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const loadPDF = async () => {
      try {
        const token = localStorage.getItem('token');
        const response = await fetch(`/api/files/${fileKey}`, {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        });

        if (!response.ok) {
          throw new Error('Failed to load PDF');
        }

        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        setPdfUrl(url);
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    };

    loadPDF();
  }, [fileKey]);

  if (loading) return <div>Loading PDF...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <iframe
      src={pdfUrl}
      style={{ width: '100%', height: '600px', border: '1px solid #ccc' }}
      title="PDF Viewer"
    />
  );
};
```

**Notes:**
- File key được trả về trong template response (`url` field)
- Cần JWT token để authenticate
- PDF sẽ được hiển thị inline trong browser
- Có thể sử dụng PDF.js library để render PDF với controls tùy chỉnh

## 5. Workflow Implementation

### 5.1 Complete User Journey

```javascript
// 1. Register/Login
const token = await loginUser(email, password);

// 2. Create Template
const template = await createTemplateFromPDF(pdfFile, "My Contract");

// 3. Get template full info (to see submitters status)
const templateInfo = await getTemplateFullInfo(template.data.id);
console.log(`Template has ${templateInfo.data.total_submitters} submitters`);

// 4. Add Signature Fields
const fields = [];
for (const field of signatureFields) {
  const newField = await createTemplateField(template.data.id, field);
  fields.push(newField.data);
}

// 4. Create Submission
const submission = await createSubmission(template.data.id, [
  { name: "Signer 1", email: "signer1@example.com" },
  { name: "Signer 2", email: "signer2@example.com" }
]);

// 5. Share signing links (tokens are in submission.data.submitters)
const signingUrls = submission.data.submitters.map(submitter =>
  `${window.location.origin}/sign/${submitter.token}`
);
```

### 5.2 Signing Journey (For Signers)

```javascript
// 1. Get submitter info using token from URL
const submitterInfo = await getSubmitterInfo(tokenFromUrl);

// 2. Display signing form with template fields
// (Template info would be included in submitterInfo response)

// 3. Collect signatures and submit
const signatures = fields.map(field => ({
  field_id: field.id,
  signature_value: canvasSignature.toDataURL() // Base64 image
}));

const result = await submitSignatures(tokenFromUrl, signatures);

// 4. Show success message
if (result.success) {
  showSuccess("Document signed successfully!");
}
```

## 6. Error Handling

### Common Error Responses:
```javascript
// 401 Unauthorized - JWT token missing/invalid
{
  "success": false,
  "status_code": 401,
  "message": "Unauthorized",
  "error": "Invalid or missing JWT token"
}

// 403 Forbidden - No permission
{
  "success": false,
  "status_code": 403,
  "message": "Forbidden",
  "error": "You don't have permission to access this resource"
}

// 404 Not Found - Resource not found
{
  "success": false,
  "status_code": 404,
  "message": "Not Found",
  "error": "Resource not found"
}
```

### Frontend Error Handling:
```javascript
const handleApiResponse = async (response) => {
  const data = await response.json();

  if (!response.ok) {
    switch (response.status) {
      case 401:
        // Redirect to login
        window.location.href = '/login';
        break;
      case 403:
        showError('You do not have permission to perform this action');
        break;
      case 404:
        showError('Resource not found');
        break;
      default:
        showError(data.error || 'An error occurred');
    }
    throw new Error(data.error);
  }

  return data;
};
```

## 7. Security Notes

1. **JWT Token Storage:** Store tokens securely (localStorage/sessionStorage with httpOnly cookies for production)
2. **Token Refresh:** Implement token refresh logic for long sessions
3. **Authorization:** All public APIs now require JWT authentication + ownership validation
4. **Input Validation:** Validate all user inputs on both frontend and backend
5. **HTTPS:** Always use HTTPS in production
