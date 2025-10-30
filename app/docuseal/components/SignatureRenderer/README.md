# SignatureRenderer Component

## Mục đích
Component để render chữ ký (signature) hoặc initials từ nhiều nguồn dữ liệu khác nhau:
- Vector data (vẽ tay từ canvas)
- Typed text (text thường)
- Image URL (uploaded images)
- Blob URLs (local images)

## Props

```typescript
interface SignatureRendererProps {
  data: string;           // Required: JSON string, text, hoặc image URL
  width?: number;         // Optional: Canvas width (default: 200)
  height?: number;        // Optional: Canvas height (default: 100)
  fieldType?: string;     // Optional: 'signature' | 'initials' (affects text rendering style)
  color?: string;         // Optional: Color for signature/text (default: '#000000')
}
```

## Cách sử dụng

### 1. Render vector signature (vẽ tay)
```tsx
<SignatureRenderer
  data={JSON.stringify([[{x: 10, y: 20}, {x: 30, y: 40}]])}
  width={300}
  height={150}
  color="#0066cc"
/>
```

### 2. Render typed text
```tsx
<SignatureRenderer
  data="John Doe"
  width={300}
  height={150}
  fieldType="signature"
  color="#ff0000"
/>
```

### 3. Render initials (italic style)
```tsx
<SignatureRenderer
  data="JD"
  width={150}
  height={100}
  fieldType="initials"
  color="#00cc00"
/>
```

### 4. Render image URL
```tsx
<SignatureRenderer
  data="https://example.com/signature.png"
  width={300}
  height={150}
  // color không apply cho images
/>
```

### 5. Render blob URL (uploaded file)
```tsx
<SignatureRenderer
  data="blob:http://localhost:3000/abc123"
  width={300}
  height={150}
/>
```

## Color Support

Prop `color` cho phép thay đổi màu của:
- ✅ Vector signatures (stroke color)
- ✅ Typed text (fill color)
- ✅ Initials (fill color)
- ❌ Images (không thay đổi được)

### Ví dụ màu sắc:
```tsx
// Đen (default)
<SignatureRenderer data="Sign here" color="#000000" />

// Xanh dương
<SignatureRenderer data="Sign here" color="#0066cc" />

// Đỏ
<SignatureRenderer data="Sign here" color="#ff0000" />

// Trắng (cho background tối)
<SignatureRenderer data="Sign here" color="#ffffff" />

// RGB
<SignatureRenderer data="Sign here" color="rgb(0, 102, 204)" />

// RGBA (với transparency)
<SignatureRenderer data="Sign here" color="rgba(0, 102, 204, 0.8)" />
```

## Data Formats

### Vector Data (JSON)
Mảng các nhóm điểm, mỗi nhóm là một stroke:
```json
[
  [
    {"x": 10, "y": 20},
    {"x": 15, "y": 25},
    {"x": 20, "y": 30}
  ],
  [
    {"x": 30, "y": 40},
    {"x": 35, "y": 45}
  ]
]
```

### Text Data
String thường:
```
"John Doe"
"JD"
"Signature"
```

### Image URLs
```
"https://example.com/image.png"
"/uploads/signature.jpg"
"blob:http://localhost:3000/abc123"
```

## Rendering Behavior

### Auto-scaling
Component tự động scale nội dung để fit vào canvas:
- Vector signatures: Scale với padding 10px
- Text: Auto-size font để fill canvas
- Images: Maintain aspect ratio, center in canvas

### Text Rendering Modes

**Signature mode** (default):
- Sans-serif font
- Normal weight
- Centered
- Auto-sized

**Initials mode** (`fieldType="initials"`):
- Arial font
- Italic style
- Higher resolution (3x scale)
- Optimized for short text

## Examples trong Project

### 1. Trong Signature Settings
```tsx
<SignatureRenderer
  data={savedSignature}
  width={300}
  height={150}
  color="white"  // Cho dark background
/>
```

### 2. Trong PDF Template Editor
```tsx
<SignatureRenderer
  fieldType={field.field_type}
  data={signatureData}
  width={field.position.width * 600}
  height={field.position.height * 800}
  color="#000000"
/>
```

### 3. Preview trong Form
```tsx
{value && (
  <SignatureRenderer
    data={value}
    width={200}
    height={100}
    fieldType="signature"
    color={theme.palette.primary.main}
  />
)}
```

## Error Handling

Component xử lý các trường hợp lỗi:

1. **Image load failed**: Hiển thị text "Image failed to load"
2. **Invalid JSON**: Fallback sang text rendering
3. **Empty data**: Không render gì (canvas trống)
4. **Canvas not available**: Log error ra console

## Performance Notes

- Canvas được re-render khi bất kỳ prop nào thay đổi
- Images được cache bởi browser
- Vector data được vẽ mỗi lần useEffect chạy
- High-DPI support cho initials (3x scaling)

## Styling

Canvas được style với:
```css
width: 100%;
height: 100%;
max-width: 100%;
max-height: 100%;
```

Parent container nên có fixed dimensions để canvas render đúng size.

## Browser Compatibility

- ✅ Modern browsers (Chrome, Firefox, Safari, Edge)
- ✅ Canvas 2D API support required
- ✅ CORS support cho external images
- ⚠️ Blob URLs require same-origin hoặc CORS headers
