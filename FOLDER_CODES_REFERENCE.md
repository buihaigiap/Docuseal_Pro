# MÃ FOLDER SYSTEM - QUICK REFERENCE

## 📁 CÁC MÃ FOLDER ĐÃ TẠO

Từ demo vừa chạy:

### 🏢 FOLDER CHÍNH
- **CÔNG TY**: MÃ `11`
  - 📁 **NHÂN SỰ**: MÃ `12` (folder cha: `11`)
  - 📁 **BÁN HÀNG**: MÃ `13` (folder cha: `11`) 
  - 📁 **PHÁP LÝ**: MÃ `14` (folder cha: `11`)

### 📄 TEMPLATES VÀ MÃ FOLDER
- **Hợp đồng lao động**: Template ID `18` → hiện tại trong folder `14` (PHÁP LÝ)
- **Hợp đồng mua bán**: Template ID `19` → trong folder `13` (BÁN HÀNG)

## 🔧 CÁCH SỬ DỤNG MÃ FOLDER

### 1. Tạo Folder Mới
```bash
POST /api/folders
{
  "name": "Tên Folder",
  "parent_folder_id": 11,    # MÃ FOLDER CHA (optional)
  "color": "#3b82f6"
}
```

### 2. Lấy Folder theo Mã
```bash
GET /api/folders/11         # Lấy folder có mã 11
GET /api/folders/12         # Lấy folder có mã 12
```

### 3. Tạo Template trong Folder
```bash
POST /api/templates  
{
  "name": "Tên Template",
  "folder_id": 12,          # MÃ FOLDER để chứa template
  "document": "base64_content"
}
```

### 4. Di chuyển Template giữa Folders
```bash
PUT /api/templates/18/move/14    # Move template ID 18 vào folder ID 14
PUT /api/templates/19/move/0     # Move template ID 19 ra ngoài (no folder)
```

### 5. Cập nhật Folder
```bash
PUT /api/folders/11         # Cập nhật folder có mã 11
{
  "name": "Tên Mới",
  "color": "#ef4444"
}
```

### 6. Xóa Folder
```bash
DELETE /api/folders/12      # Xóa folder có mã 12
```

## 📊 HIERARCHY STRUCTURE

```
CÔNG TY (ID: 11)
├── NHÂN SỰ (ID: 12)
│   └── [templates: none]
├── BÁN HÀNG (ID: 13) 
│   └── [templates: Hợp đồng mua bán (ID: 19)]
└── PHÁP LÝ (ID: 14)
    └── [templates: Hợp đồng lao động (ID: 18)]
```

## 💡 LƯU Ý QUAN TRỌNG

1. **MỖI FOLDER CÓ MÃ DUY NHẤT**: Không trùng lặp trong hệ thống
2. **TEMPLATES THUỘC VỀ USER**: Chỉ user tạo folder mới thấy được
3. **HIERARCHY KHÔNG GIỚI HạN**: Có thể tạo nhiều cấp folder con
4. **TEMPLATES CÓ THỂ DI CHUYỂN**: Giữa các folders hoặc ra ngoài
5. **XÓA FOLDER CHA**: Sẽ move tất cả folder con lên cấp cha

## 🎯 USE CASES

### Tổ chức theo Phòng ban
```
CÔNG TY (11)
├── NHÂN SỰ (12)
├── KẾ TOÁN (13)  
├── MARKETING (14)
└── SALES (15)
```

### Tổ chức theo Loại hợp đồng  
```
HỢP ĐỒNG (20)
├── LAO ĐỘNG (21)
├── MUA BÁN (22)
├── THUÊ MẶT BẰNG (23)
└── ĐỐI TÁC (24)
```

### Tổ chức theo Dự án
```
DỰ ÁN A (30)
├── THIẾT KẾ (31)
├── PHÁP LÝ (32)
└── TÀI CHÍNH (33)
```

## 🚀 TỰ ĐỘNG HOÁ

Có thể sử dụng mã folder để:
- **Auto-assign templates** vào folders dựa trên tên/type
- **Batch operations** trên tất cả templates trong folder  
- **Permission management** theo folder
- **Reporting** theo từng folder/phòng ban
- **API integration** với external systems