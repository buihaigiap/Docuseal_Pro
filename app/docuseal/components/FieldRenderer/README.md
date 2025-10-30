# FieldRenderer Component

## Mục đích
Component dùng chung để render nội dung của các field trong PDF, đảm bảo logic hiển thị **nhất quán** giữa:
- **PdfFieldEditor** (chế độ chỉnh sửa template)
- **PdfFullView** (chế độ xem/preview submission)

## Vấn đề đã giải quyết

### Trước đây:
- Code render field bị **trùng lặp** giữa `PdfFieldEditor.tsx` và `PdfFullView.tsx`
- Logic hiển thị **không đồng nhất**, dẫn đến:
  - Vị trí và kích thước field có thể khác nhau giữa editor và view
  - Khó bảo trì khi cần thay đổi cách hiển thị field type mới
  - Dễ xảy ra bug khi update logic ở một nơi mà quên update nơi khác

### Sau khi refactor:
- **Một component duy nhất** xử lý toàn bộ logic render
- **Đảm bảo nhất quán** 100% về hiển thị field
- **Dễ maintain**: Chỉ cần sửa 1 nơi khi có thay đổi
- **Clean code**: Giảm ~150 dòng code trùng lặp

## Cách sử dụng

### Props Interface
```typescript
interface FieldRendererProps {
  field: Field;           // Field object chứa thông tin field
  value?: string;         // Giá trị đã điền (nếu có)
  className?: string;     // Custom CSS classes (applied to wrapper)
  style?: React.CSSProperties;  // Custom inline styles (applied to wrapper)
  onClick?: () => void;   // Handler khi click
  title?: string;         // Tooltip text
  children?: React.ReactNode;  // Custom content (override default render)
}
```

### ⚠️ Important: Position Handling

**FieldRenderer** chỉ render **nội dung** của field, KHÔNG xử lý positioning.
Position phải được handle bởi **parent component** (wrapper div).

### Ví dụ 1: Render field trong PdfFullView (có value)
```tsx
<div
  className={getFieldClass(field.partner, true, partnerColorClasses)}
  style={{
    position: 'absolute',
    left: `${field.position.x * 100}%`,
    top: `${field.position.y * 100}%`,
    width: `${field.position.width * 100}%`,
    height: `${field.position.height * 100}%`,
    cursor: 'pointer',
    fontSize: '16px',
    color: 'black',
    fontWeight: 'bold'
  }}
  onClick={() => onFieldClick(field)}
  title={field.name}
>
  <FieldRenderer
    field={field}
    value={texts[field.id]}
  />
</div>
```

### Ví dụ 2: Render field trong PdfFieldEditor (với Rnd)
```tsx
<Rnd
  size={{ width: ..., height: ... }}
  position={{ x: ..., y: ... }}
  className={getFieldClass(...)}
>
  <div className="w-full h-full relative">
    <FieldRenderer field={field} />
  </div>
</Rnd>
```

### Ví dụ 3: Render với custom content
```tsx
<div style={{ position: 'absolute', ... }}>
  <FieldRenderer field={field}>
    <div className="custom-editing-ui">
      {/* Your custom content */}
    </div>
  </FieldRenderer>
</div>
```

## Field Types được hỗ trợ

| Field Type | Hiển thị khi có value | Hiển thị khi placeholder |
|------------|----------------------|-------------------------|
| `signature` / `initials` | SignatureRenderer component | Icon PenTool |
| `image` | `<img>` tag | Icon ImageIcon |
| `file` | Tên file | Icon File |
| `checkbox` | Checkmark nếu true | Icon CheckSquare |
| `radio` / `select` | Text value | Icon Circle/ChevronDown |
| `multiple` | Text value (truncated) | Icon List |
| `cells` | Grid với characters | Grid với số thứ tự |
| `text` / `number` / `date` | Text (truncated nếu >10 chars) | Icon tương ứng |

## Positioning

**QUAN TRỌNG**: FieldRenderer **KHÔNG** tự xử lý position!

### Kiến trúc phân tách trách nhiệm:

1. **Parent Component** (PdfFullView, PdfFieldEditor):
   - Quản lý **position** (absolute, left, top, width, height)
   - Quản lý **layout** (wrapper div hoặc Rnd component)
   - Quản lý **interactions** (click handlers, drag/resize)

2. **FieldRenderer Component**:
   - Chỉ render **content** bên trong field
   - Fill 100% width/height của parent
   - Không biết gì về position trong canvas

### Hai cách tính toán position:

1. **Percentage-based** (PdfFullView - wrapper div):
   ```tsx
   <div style={{
     position: 'absolute',
     left: `${field.position.x * 100}%`,
     top: `${field.position.y * 100}%`,
     width: `${field.position.width * 100}%`,
     height: `${field.position.height * 100}%`
   }}>
     <FieldRenderer field={field} />
   </div>
   ```

2. **Pixel-based** (PdfFieldEditor with Rnd):
   ```tsx
   <Rnd
     size={{ 
       width: field.position.width * canvasClientWidth,
       height: field.position.height * canvasClientHeight
     }}
     position={{ 
       x: field.position.x * canvasClientWidth,
       y: field.position.y * canvasClientHeight
     }}
   >
     <div className="w-full h-full">
       <FieldRenderer field={field} />
     </div>
   </Rnd>
   ```

Cả hai đều dựa trên `field.position` với giá trị **decimal (0-1)** lưu trong database.

## Notes

- Component **chỉ** render content, không xử lý positioning
- Parent component chịu trách nhiệm về position, size, và layout
- Component fill 100% width/height của container
- Với `cells` field type, parent có thể thêm resize handles riêng
- `children` prop cho phép override hoàn toàn rendering logic khi cần
- **Icon styling**: Luôn centered và có text color `text-black` trong edit mode

## Migration Guide

Nếu bạn đang có code cũ render field thủ công:

**Before:**
```tsx
{field.field_type === 'signature' ? (
  <SignatureRenderer ... />
) : field.field_type === 'image' ? (
  <img ... />
) : ...}
```

**After:**
```tsx
<div style={{ position: 'absolute', ... }}>
  <FieldRenderer field={field} value={texts[field.id]} />
</div>
```

⚠️ **Lưu ý**: Đừng quên wrap FieldRenderer trong div có position styling!
