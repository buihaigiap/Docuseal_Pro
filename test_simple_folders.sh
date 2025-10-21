#!/bin/bash

# Test Simplified Template Folders API (without color and description)
BASE_URL="http://localhost:8080/api"
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjIsImVtYWlsIjoidGVzdF8xNzYwOTIzMDM5QGV4YW1wbGUuY29tIiwicm9sZSI6IlRlYW1NZW1iZXIiLCJleHAiOjE3NjEwMTc2NTV9.lc23YrlO_7RlAtp6wXmu43h8ZisAifml840AM2JAJ48"

echo "======================================"
echo "Testing SIMPLIFIED Template Folders"  
echo "======================================"

# 1. Tạo folder đơn giản (chỉ có tên)
echo ""
echo "1. Creating simple folder (name only)..."
FOLDER1=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Documents"
  }')
echo "✅ Response: $FOLDER1"

FOLDER1_ID=$(echo $FOLDER1 | jq -r '.data.id')
echo "📁 Created folder ID: $FOLDER1_ID"

# 2. Tạo sub-folder
echo ""
echo "2. Creating sub-folder..."
SUBFOLDER=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Contracts\",
    \"parent_folder_id\": $FOLDER1_ID
  }")
echo "✅ Response: $SUBFOLDER"

SUBFOLDER_ID=$(echo $SUBFOLDER | jq -r '.data.id')
echo "📁 Created subfolder ID: $SUBFOLDER_ID"

# 3. Lấy tất cả folders
echo ""
echo "3. Getting all folders..."
curl -s -X GET "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 4. Tạo template trong folder
echo ""
echo "4. Creating template in folder..."
TEMPLATE=$(curl -s -X POST "$BASE_URL/templates" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Simple Contract\",
    \"folder_id\": $SUBFOLDER_ID,
    \"document\": \"$(echo 'Simple contract content' | base64)\"
  }")
echo "✅ Template created: $(echo $TEMPLATE | jq -r '.data.name')"

TEMPLATE_ID=$(echo $TEMPLATE | jq -r '.data.id')

# 5. Update folder (chỉ tên)
echo ""
echo "5. Updating folder name..."
curl -s -X PUT "$BASE_URL/folders/$FOLDER1_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Important Documents"
  }' | jq '.'

# 6. Lấy folder với templates
echo ""
echo "6. Getting folder with templates..."
curl -s -X GET "$BASE_URL/folders/$SUBFOLDER_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 7. Move template ra ngoài
echo ""
echo "7. Moving template out of folder..."
curl -s -X PUT "$BASE_URL/templates/$TEMPLATE_ID/move/0" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 8. Xóa subfolder
echo ""
echo "8. Deleting subfolder..."
curl -s -X DELETE "$BASE_URL/folders/$SUBFOLDER_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 9. Kiểm tra kết quả cuối
echo ""
echo "9. Final check - all folders:"
curl -s -X GET "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

echo ""
echo "10. Final check - all templates (should show folder_id as null):"
curl -s -X GET "$BASE_URL/templates" \
  -H "Authorization: Bearer $TOKEN" | jq '.data[] | {id: .id, name: .name, folder_id: .folder_id}'

echo ""
echo "======================================"
echo "✅ SIMPLIFIED FOLDERS TEST COMPLETE!"
echo "💡 Key changes: NO color, NO description"
echo "📁 Only name and parent_folder_id needed"
echo "======================================"