#!/bin/bash

# Test tạo template trong folder lồng nhau (folder B trong folder A)
BASE_URL="http://localhost:8080/api"
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjIsImVtYWlsIjoidGVzdF8xNzYwOTIzMDM5QGV4YW1wbGUuY29tIiwicm9sZSI6IlRlYW1NZW1iZXIiLCJleHAiOjE3NjEwMTc2NTV9.lc23YrlO_7RlAtp6wXmu43h8ZisAifml840AM2JAJ48"

echo "======================================"
echo "Test tạo template trong folder lồng nhau"
echo "======================================"

# 1. Tạo folder A
echo ""
echo "1. Tạo folder A..."
FOLDER_A=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Folder A"
  }')
echo "✅ Response: $FOLDER_A"

FOLDER_A_ID=$(echo $FOLDER_A | jq -r '.data.id')
echo "📁 Folder A ID: $FOLDER_A_ID"

# 2. Tạo folder B trong folder A
echo ""
echo "2. Tạo folder B trong folder A..."
FOLDER_B=$(curl -s -X POST "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Folder B\",
    \"parent_folder_id\": $FOLDER_A_ID
  }")
echo "✅ Response: $FOLDER_B"

FOLDER_B_ID=$(echo $FOLDER_B | jq -r '.data.id')
echo "📁 Folder B ID: $FOLDER_B_ID"

# 3. Tạo template trong folder B
echo ""
echo "3. Tạo template trong folder B..."
TEMPLATE=$(curl -s -X POST "$BASE_URL/templates" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Template trong Folder B\",
    \"folder_id\": $FOLDER_B_ID,
    \"document\": \"$(echo 'Nội dung template trong folder lồng nhau' | base64)\"
  }")
echo "✅ Template created: $(echo $TEMPLATE | jq -r '.data.name')"

TEMPLATE_ID=$(echo $TEMPLATE | jq -r '.data.id')
echo "📄 Template ID: $TEMPLATE_ID"

# 4. Kiểm tra cấu trúc folders
echo ""
echo "4. Kiểm tra cấu trúc folders..."
curl -s -X GET "$BASE_URL/folders" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

# 5. Kiểm tra template
echo ""
echo "5. Kiểm tra template..."
curl -s -X GET "$BASE_URL/templates/$TEMPLATE_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '.'

echo ""
echo "Test hoàn thành! Template đã được tạo trong folder B (trong folder A)."