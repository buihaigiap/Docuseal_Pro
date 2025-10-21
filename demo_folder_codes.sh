#!/bin/bash

# Demo: Sử dụng MÃ FOLDER trong Template System
# Script này demonstrate cách sử dụng ID/mã của folders

BASE_URL="http://localhost:8080/api"
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjIsImVtYWlsIjoidGVzdF8xNzYwOTIzMDM5QGV4YW1wbGUuY29tIiwicm9sZSI6IlRlYW1NZW1iZXIiLCJleHAiOjE3NjEwMTc2NTV9.lc23YrlO_7RlAtp6wXmu43h8ZisAifml840AM2JAJ48"

echo "===========================================" 
echo "DEMO: Sử dụng MÃ FOLDER cho Templates"
echo "==========================================="

# 1. Tạo folder chính - lấy MÃ FOLDER
echo ""
echo "🗂️  Bước 1: Tạo folder chính 'CÔNG TY'"
MAIN_FOLDER=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CÔNG TY",
    "color": "#1e40af",
    "description": "Tất cả tài liệu công ty"
  }')

MAIN_ID=$(echo $MAIN_FOLDER | jq -r '.data.id')
echo "✅ Tạo thành công folder chính với MÃ: $MAIN_ID"
echo "📋 Chi tiết: $(echo $MAIN_FOLDER | jq -r '.data.name') (ID: $MAIN_ID)"

# 2. Tạo sub-folders với mã cha
echo ""
echo "📁 Bước 2: Tạo các sub-folders với MÃ CHA = $MAIN_ID"

# Sub-folder 1: HR
HR_FOLDER=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"NHÂN SỰ\",
    \"parent_folder_id\": $MAIN_ID,
  }")
HR_ID=$(echo $HR_FOLDER | jq -r '.data.id')
echo "✅ Tạo folder NHÂN SỰ với MÃ: $HR_ID (cha: $MAIN_ID)"

# Sub-folder 2: Sales
SALES_FOLDER=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"BÁN HÀNG\",
    \"parent_folder_id\": $MAIN_ID,
    \"color\": \"#16a34a\",
    \"description\": \"Hợp đồng bán hàng\"
  }")
SALES_ID=$(echo $SALES_FOLDER | jq -r '.data.id')
echo "✅ Tạo folder BÁN HÀNG với MÃ: $SALES_ID (cha: $MAIN_ID)"

# Sub-folder 3: Legal
LEGAL_FOLDER=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"PHÁP LÝ\",
    \"parent_folder_id\": $MAIN_ID,
    \"color\": \"#7c2d12\",
    \"description\": \"Tài liệu pháp lý\"
  }")
LEGAL_ID=$(echo $LEGAL_FOLDER | jq -r '.data.id')
echo "✅ Tạo folder PHÁP LÝ với MÃ: $LEGAL_ID (cha: $MAIN_ID)"

# 3. Tạo templates trong các folders sử dụng MÃ FOLDER
echo ""
echo "📄 Bước 3: Tạo templates trong folders (sử dụng MÃ FOLDER)"

# Template trong HR folder
HR_TEMPLATE=$(curl -s -X POST "$BASE_URL/templates" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Hợp đồng lao động\",
    \"folder_id\": $HR_ID,
    \"document\": \"$(echo 'Mẫu hợp đồng lao động chuẩn' | base64)\"
  }")
HR_TEMPLATE_ID=$(echo $HR_TEMPLATE | jq -r '.data.id')
echo "✅ Tạo template 'Hợp đồng lao động' với ID: $HR_TEMPLATE_ID trong folder: $HR_ID"

# Template trong Sales folder  
SALES_TEMPLATE=$(curl -s -X POST "$BASE_URL/templates" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Hợp đồng mua bán\",
    \"folder_id\": $SALES_ID,
    \"document\": \"$(echo 'Mẫu hợp đồng mua bán sản phẩm' | base64)\"
  }")
SALES_TEMPLATE_ID=$(echo $SALES_TEMPLATE | jq -r '.data.id')
echo "✅ Tạo template 'Hợp đồng mua bán' với ID: $SALES_TEMPLATE_ID trong folder: $SALES_ID"

# 4. Hiển thị cấu trúc hierarchy với MÃ FOLDER
echo ""
echo "🌳 Bước 4: Hiển thị cấu trúc hierarchy với MÃ FOLDER"
echo "================================================"
curl -s -X GET "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" | jq -r '
  .data[] | 
  "📁 \(.name) (MÃ: \(.id)) - Màu: \(.color)" + 
  (if .children and (.children | length > 0) then 
    "\n" + (.children[] | "  └─ 📁 \(.name) (MÃ: \(.id)) - Cha: \(.parent_folder_id)")
  else "" end)'

# 5. Lấy templates theo MÃ FOLDER cụ thể
echo ""
echo "📋 Bước 5: Lấy templates trong folder NHÂN SỰ (MÃ: $HR_ID)"
echo "=============================================="
curl -s -X GET "$BASE_URL/folders/$HR_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '
  "📁 Folder: " + .data.name + " (MÃ: " + (.data.id | tostring) + ")" +
  (if .data.templates and (.data.templates | length > 0) then
    "\n📄 Templates:" +
    (.data.templates[] | "\n  - " + .name + " (ID: " + (.id | tostring) + ")")
  else "\n📄 Không có templates" end)'

# 6. Demo di chuyển template giữa folders sử dụng MÃ
echo ""
echo "🔄 Bước 6: Di chuyển template từ NHÂN SỰ sang PHÁP LÝ"
echo "=================================================="
echo "Di chuyển template ID $HR_TEMPLATE_ID từ folder $HR_ID sang folder $LEGAL_ID"

MOVE_RESULT=$(curl -s -X PUT "$BASE_URL/templates/$HR_TEMPLATE_ID/move/$LEGAL_ID" \
  -H "Authorization: Bearer $TOKEN")
echo "✅ Kết quả: $(echo $MOVE_RESULT | jq -r '.message')"

# 7. Kiểm tra lại templates sau khi move
echo ""
echo "🔍 Bước 7: Kiểm tra templates sau khi di chuyển"
echo "============================================="

echo "📁 Folder NHÂN SỰ (MÃ: $HR_ID):"
curl -s -X GET "$BASE_URL/folders/$HR_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '
  if .data.templates and (.data.templates | length > 0) then
    (.data.templates[] | "  📄 " + .name + " (ID: " + (.id | tostring) + ")")
  else "  📄 Không có templates" end'

echo ""
echo "📁 Folder PHÁP LÝ (MÃ: $LEGAL_ID):"
curl -s -X GET "$BASE_URL/folders/$LEGAL_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '
  if .data.templates and (.data.templates | length > 0) then
    (.data.templates[] | "  📄 " + .name + " (ID: " + (.id | tostring) + ")")
  else "  📄 Không có templates" end'

# 8. Tóm tắt tất cả MÃ đã sử dụng
echo ""
echo "📊 Bước 8: TÓNG TẤT CÁC MÃ FOLDER & TEMPLATE"
echo "==========================================="
echo "🗂️  FOLDER CHÍNH - CÔNG TY: MÃ $MAIN_ID"
echo "📁 Sub-folder NHÂN SỰ: MÃ $HR_ID (cha: $MAIN_ID)"
echo "📁 Sub-folder BÁN HÀNG: MÃ $SALES_ID (cha: $MAIN_ID)"  
echo "📁 Sub-folder PHÁP LÝ: MÃ $LEGAL_ID (cha: $MAIN_ID)"
echo "📄 Template Hợp đồng lao động: ID $HR_TEMPLATE_ID (hiện tại trong folder: $LEGAL_ID)"
echo "📄 Template Hợp đồng mua bán: ID $SALES_TEMPLATE_ID (trong folder: $SALES_ID)"

echo ""
echo "✅ DEMO HOÀN THÀNH!"
echo "👆 Tất cả operations đều sử dụng MÃ FOLDER để organize và manage templates"