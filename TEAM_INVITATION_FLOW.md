# Team Invitation Flow - Email System

## Tổng Quan

Hệ thống gửi email mời tự động khi admin tạo team member mới. User nhận email với link để set password và kích hoạt tài khoản.

## Chi Tiết Flow

### 1. Tạo Team Member (Backend)

**Endpoint:** `POST /api/team/members`

**Request:**
```json
{
  "name": "John Doe",
  "email": "john@example.com"
}
```

**Process:**
```rust
// src/routes/team.rs - create_team_member()

1. Kiểm tra user hiện tại có account_id
2. Tạo activation_token (UUID)
3. Hash password tạm (sẽ thay đổi qua email)
4. Set is_active = false (user phải set password trước)
5. Lưu user vào database
6. Gửi email invitation (async, không block UI)
```

**Database:**
- User được tạo với `is_active = false`
- `activation_token` được lưu để verify sau này
- `account_id` được set để liên kết với team

### 2. Gửi Email Invitation (Async)

**Email Service:** `EmailService::send_team_invitation_email()`

**Thời điểm gửi:**
- ✅ Sau khi user được tạo thành công
- ✅ Gửi bất đồng bộ (`tokio::spawn`) để không block UI
- ✅ Nếu email service fail, không ảnh hưởng đến API response

**Điều kiện gửi:**
- ✅ Chỉ gửi nếu tạo user thành công
- ✅ Nếu email đã tồn tại → StatusCode::CONFLICT → không gửi email
- ✅ Nếu có lỗi validation → không tạo user → không gửi email

**Nội dung Email:**

**Subject:** `"{invited_by_name} invited you to join their team on DocuSeal Pro"`

**HTML Template:**
- Header: "You've Been Invited!"
- Thông tin:
  - Account name
  - Invited by
  - Quyền lợi team member
- CTA Button: "Accept Invitation"
- Link: `{BASE_URL}/set-password?token={activation_token}`
- Footer: Note về expiration (có thể thêm sau)

**Text Body:**
```
Hello {name},

{invited_by} has invited you to join their team '{account_name}' on DocuSeal Pro.

Accept invitation: {invitation_link}

DocuSeal Pro
```

### 3. User Nhận Email & Set Password

**Link trong email:**
```
http://localhost:8081/set-password?token={activation_token}
```

**Frontend Page:** `/set-password`
- Component: `SetPasswordPage.tsx`
- Form fields:
  - Password (min 8 chars, với toggle show/hide)
  - Confirm Password
- Validation:
  - Passwords match
  - Min length 8
  - Token exists in URL

**API Call:**
```typescript
POST /api/auth/set-password
{
  "token": "uuid-from-url",
  "password": "new-secure-password"
}
```

### 4. Activate Account (Backend)

**Endpoint:** `POST /api/auth/set-password`

**Process:**
```rust
// src/routes/web.rs - set_password_handler()

1. Tìm user bằng activation_token
2. Kiểm tra is_active = false (chưa activate)
3. Hash password mới
4. Update user:
   - password_hash = new hash
   - is_active = true
   - activation_token = NULL
5. Return success message
```

**Response:**
```json
{
  "message": "Password set successfully. You can now login.",
  "email": "john@example.com"
}
```

### 5. Redirect to Login

Sau khi set password thành công:
- Toast notification: "Password set successfully!"
- Auto redirect sau 2 giây → `/login`
- User login với email + password mới

## Sơ Đồ Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Admin tạo team member                                       │
│ POST /api/team/members                                      │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ Backend tạo user                                            │
│ - Generate activation_token (UUID)                          │
│ - Set is_active = false                                     │
│ - Set account_id                                            │
│ - Save to database                                          │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ Gửi email invitation (async)                                │
│ - tokio::spawn (không block)                                │
│ - EmailService::send_team_invitation_email()                │
│ - Link: /set-password?token={activation_token}             │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ User nhận email                                             │
│ - Click "Accept Invitation" button                          │
│ - Hoặc copy/paste link                                      │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ SetPasswordPage                                             │
│ - Input: Password + Confirm Password                        │
│ - Validation: length, match                                 │
│ - Submit: POST /api/auth/set-password                       │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ Backend activate account                                    │
│ - Verify activation_token                                   │
│ - Update: password, is_active=true, token=NULL              │
│ - Return success                                            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ Redirect to Login                                           │
│ - Toast: "Password set successfully!"                       │
│ - Auto redirect after 2s                                    │
│ - User login với credentials mới                            │
└─────────────────────────────────────────────────────────────┘
```

## Error Handling

### Backend Errors

**Email đã tồn tại:**
```rust
StatusCode::CONFLICT
// Không tạo user, không gửi email
```

**Email service fail:**
```rust
eprintln!("Failed to send team invitation email: {}", e);
// Log error nhưng không fail API response
// User đã được tạo thành công
```

**Invalid activation_token:**
```json
{
  "error": "Invalid token or user already activated"
}
```

### Frontend Errors

**Missing token:**
```typescript
if (!token) {
    setError('Invalid or missing activation token');
}
```

**Passwords don't match:**
```typescript
if (password !== confirmPassword) {
    setError('Passwords do not match');
}
```

**Password too short:**
```typescript
if (password.length < 8) {
    setError('Password must be at least 8 characters long');
}
```

## Environment Variables

```env
# Email Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
FROM_EMAIL=noreply@docuseal.com
FROM_NAME=DocuSeal Pro
SMTP_USE_TLS=true
EMAIL_TEST_MODE=false

# Application
BASE_URL=http://localhost:8081
```

## Test Mode

Để test mà không gửi email thật:

```env
EMAIL_TEST_MODE=true
```

Khi enabled, email chỉ được log ra console:
```
TEST MODE: Would send team invitation email to john@example.com (John Doe) with link: http://localhost:8081/set-password?token=...
```

## Security Considerations

### Token Security
- ✅ activation_token là UUID (random, không đoán được)
- ✅ Token chỉ dùng 1 lần (set NULL sau khi activate)
- ✅ Check is_active=false để tránh activate lại

### Password Security
- ✅ Minimum 8 characters
- ✅ Bcrypt hash với DEFAULT_COST
- ✅ Password không bao giờ gửi qua email

### Email Verification
- ✅ Link chỉ work với token hợp lệ
- ✅ User phải có access vào email để nhận link
- ✅ Không thể activate mà không có token

## Future Enhancements

### Có thể thêm sau:

1. **Token Expiration:**
   - Thêm `activation_token_expires_at` column
   - Check expiration khi set password
   - Resend invitation nếu expired

2. **Email Templates:**
   - Support multi-language
   - Customizable branding
   - Account-specific templates

3. **Invitation Tracking:**
   - Track khi email được mở
   - Track khi link được click
   - Resend invitation functionality

4. **Better Account Name:**
   - Hiện tại hardcode "DocuSeal Pro"
   - Có thể lấy từ accounts table
   - Or từ inviting user's company

## API Endpoints Summary

### Create Team Member
```
POST /api/team/members
Authorization: Bearer {jwt_token}

Request:
{
  "name": "string",
  "email": "string"
}

Response 201:
{
  "user": {
    "id": 123,
    "name": "John Doe",
    "email": "john@example.com",
    "is_active": false,
    ...
  }
}

Response 409: Email already exists
Response 401: Unauthorized
```

### Set Password
```
POST /api/auth/set-password

Request:
{
  "token": "uuid-string",
  "password": "string"
}

Response 200:
{
  "message": "Password set successfully. You can now login.",
  "email": "john@example.com"
}

Response 400:
{
  "error": "Invalid token or user already activated"
}
```

## Files Changed

### Backend
- ✅ `src/routes/team.rs` - Added email sending to create_team_member
- ✅ `src/routes/web.rs` - Added set_password_handler endpoint
- ✅ `src/services/email.rs` - Already has send_team_invitation_email

### Frontend
- ✅ `app/docuseal/pages/Auth/SetPasswordPage.tsx` - New page
- ✅ `app/docuseal/App.tsx` - Added /set-password route

### Database
- ✅ Users table already has:
  - `activation_token` column
  - `is_active` column
  - `account_id` column
