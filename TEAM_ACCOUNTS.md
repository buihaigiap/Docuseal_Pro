# Team Accounts - Tài liệu Hướng dẫn

## Tổng quan
Team Accounts là tính năng quản lý thành viên theo tổ chức/công ty (account-based multi-tenant). Cho phép nhiều người dùng cùng làm việc trong một account và chia sẻ tài nguyên.

## Cấu trúc Database

### Bảng `accounts`
```sql
- id: BIGSERIAL PRIMARY KEY
- name: VARCHAR(255) - Tên công ty/tổ chức
- slug: VARCHAR(255) UNIQUE - URL-friendly identifier
- created_at, updated_at: TIMESTAMP
```

### Bảng `users` (đã cập nhật)
```sql
- account_id: BIGINT - FK tới accounts(id)
- archived_at: TIMESTAMP - Thời điểm archive member
```

### Bảng `account_linked_accounts`
```sql
- Liên kết giữa các accounts (cho tương lai mở rộng)
```

## API Endpoints

### 1. GET `/api/team/members`
Lấy danh sách members đang hoạt động trong account

**Response:**
```json
{
  "users": [
    {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com",
      "role": "admin",
      "account_id": 1,
      "archived_at": null,
      "created_at": "2025-11-25T..."
    }
  ]
}
```

### 2. GET `/api/team/members/archived`
Lấy danh sách members đã bị archive

### 3. POST `/api/team/members`
Tạo member mới

**Request:**
```json
{
  "name": "Jane Smith",
  "email": "jane@example.com",
  "password": "optional", // Auto-generate nếu không cung cấp
  "role": "editor" // admin, editor, member, agent, viewer
}
```

### 4. PUT `/api/team/members/:id`
Cập nhật thông tin member

**Request:**
```json
{
  "name": "Jane Smith Updated",
  "email": "jane.new@example.com",
  "role": "admin"
}
```

### 5. POST `/api/team/members/:id/archive`
Archive một member (soft delete)

### 6. POST `/api/team/members/:id/unarchive`
Khôi phục member đã archive

### 7. DELETE `/api/team/members/:id`
Xóa vĩnh viễn member (hard delete)

**Lưu ý:** Không thể xóa member cuối cùng trong account

## Frontend Components

### TeamSettings Component
- **Location:** `app/docuseal/pages/Settings/Activate/TeamSettings.tsx`
- **Route:** `/settings/team`
- **Features:**
  - Tab Active/Archived members
  - Add/Edit/Archive/Unarchive/Delete members
  - Role management
  - Auto-generate password
  - Beautiful UI with Material-UI

### UsersSettings Component (Độc lập)
- **Location:** `app/docuseal/pages/Settings/Activate/UsersSettings.tsx`
- **Route:** `/settings/users`
- **Purpose:** Quản lý User Invitations (không liên quan đến accounts)

## Roles & Permissions

1. **Admin** - Quyền cao nhất, quản lý toàn bộ account
2. **Editor** - Chỉnh sửa templates và submissions
3. **Member** - Xem và tạo submissions
4. **Agent** - Chỉ xem được phần được assign
5. **Viewer** - Chỉ xem, không chỉnh sửa

## Quy trình sử dụng

### 1. Tạo Account (Manual hoặc tự động)
```sql
INSERT INTO accounts (name, slug) VALUES ('My Company', 'my-company');
```

### 2. Gán Account cho User
```sql
UPDATE users SET account_id = 1 WHERE id = 1;
```

### 3. Thêm Members qua UI
1. Vào `/settings/team`
2. Click "Add Member"
3. Nhập thông tin: Name, Email, Role
4. Password tự động generate nếu không nhập
5. Member nhận email invitation

### 4. Quản lý Members
- **Edit:** Cập nhật name, email, role
- **Archive:** Tạm ẩn member (có thể khôi phục)
- **Unarchive:** Khôi phục member đã archive
- **Delete:** Xóa vĩnh viễn (không thể khôi phục)

## Security

### Account Isolation
- Mỗi member chỉ thấy members trong cùng account
- Queries tự động filter theo `account_id`
- Không thể truy cập data của account khác

### Validation
- Email unique
- Không xóa member cuối cùng
- Check account_id trong mọi operation

## Workflow Diagram

```
User Login
    ↓
Check account_id
    ↓
    ├─ NULL → Individual User (không có team)
    ├─ NOT NULL → Team Member
    ↓
GET /api/team/members
    ↓
Filter by account_id
    ↓
Return members trong cùng account
```

## Migration

File: `migrations/20251201000001_create_accounts_system.sql`

Chạy migration:
```bash
psql $DATABASE_URL -f migrations/20251201000001_create_accounts_system.sql
```

Hoặc:
```bash
sqlx migrate run
```

## Testing

### 1. Tạo account và test
```bash
# Login
curl -X POST http://localhost:8081/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Get members
curl -X GET http://localhost:8081/api/team/members \
  -H "Authorization: Bearer YOUR_TOKEN"

# Add member
curl -X POST http://localhost:8081/api/team/members \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"New Member","email":"new@example.com","role":"editor"}'
```

## Troubleshooting

### Lỗi: "Database error" hoặc 500
- Kiểm tra user có `account_id` chưa
- Check migration đã chạy chưa

### Lỗi: 401 Unauthorized
- Token hết hạn, đăng nhập lại
- Kiểm tra localStorage có token không

### Lỗi: "Cannot delete last user"
- Account phải có ít nhất 1 member active
- Archive hoặc thêm member khác trước khi xóa

## Tương lai mở rộng

1. **Account Linking** - Liên kết nhiều accounts
2. **Email Invitations** - Gửi email tự động
3. **Permission System** - Chi tiết hơn về quyền hạn
4. **Audit Log** - Theo dõi hoạt động của members
5. **Account Settings** - Cấu hình riêng cho mỗi account
6. **Billing** - Tính phí theo account
7. **SSO Integration** - Đăng nhập qua Google/Microsoft

## Liên hệ
Mọi thắc mắc về Team Accounts, vui lòng liên hệ team development.
