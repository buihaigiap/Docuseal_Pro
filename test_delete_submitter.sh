#!/bin/bash

# Script to test DELETE /api/submitters/{id} endpoint

BASE_URL="http://localhost:8080"
API_BASE="$BASE_URL/api"

echo "=== Testing DELETE Submitter API ==="
echo ""

# Step 1: Register/Login to get token
echo "Step 1: Login to get authentication token..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | sed 's/"token":"//')

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get token. Trying to register..."
    
    # Try to register first
    REGISTER_RESPONSE=$(curl -s -X POST "$API_BASE/auth/register" \
      -H "Content-Type: application/json" \
      -d '{
        "email": "test@example.com",
        "password": "password123",
        "name": "Test User"
      }')
    
    echo "Register response: $REGISTER_RESPONSE"
    
    # Try login again
    LOGIN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
      -H "Content-Type: application/json" \
      -d '{
        "email": "test@example.com",
        "password": "password123"
      }')
    
    TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | sed 's/"token":"//')
fi

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to authenticate"
    exit 1
fi

echo "✅ Token obtained: ${TOKEN:0:20}..."
echo ""

# Step 2: Get list of submitters
echo "Step 2: Get list of submitters..."
SUBMITTERS_RESPONSE=$(curl -s -X GET "$API_BASE/submitters" \
  -H "Authorization: Bearer $TOKEN")

echo "Submitters: $SUBMITTERS_RESPONSE"
echo ""

# Extract first submitter ID (if exists)
SUBMITTER_ID=$(echo $SUBMITTERS_RESPONSE | grep -o '"id":[0-9]*' | head -1 | sed 's/"id"://')

if [ -z "$SUBMITTER_ID" ]; then
    echo "⚠️  No submitters found. Need to create a template and submission first."
    echo ""
    echo "To test DELETE submitter:"
    echo "1. Create a template"
    echo "2. Create a submission"
    echo "3. Get the submitter ID"
    echo "4. Run: curl -X DELETE \"$API_BASE/submitters/{id}\" -H \"Authorization: Bearer \$TOKEN\""
    exit 0
fi

echo "Found submitter ID: $SUBMITTER_ID"
echo ""

# Step 3: Get submitter details
echo "Step 3: Get submitter details..."
SUBMITTER_DETAIL=$(curl -s -X GET "$API_BASE/submitters/$SUBMITTER_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Submitter details: $SUBMITTER_DETAIL"
echo ""

# Step 4: Delete the submitter
echo "Step 4: Deleting submitter $SUBMITTER_ID..."
DELETE_RESPONSE=$(curl -s -X DELETE "$API_BASE/submitters/$SUBMITTER_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Delete response: $DELETE_RESPONSE"
echo ""

# Step 5: Verify deletion
echo "Step 5: Verify deletion (should return 404)..."
VERIFY_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X GET "$API_BASE/submitters/$SUBMITTER_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Verify response: $VERIFY_RESPONSE"
echo ""

if echo "$VERIFY_RESPONSE" | grep -q "HTTP_CODE:404"; then
    echo "✅ DELETE Submitter API test PASSED - Submitter was successfully deleted"
else
    echo "⚠️  DELETE Submitter API test WARNING - Submitter may still exist"
fi

echo ""
echo "=== Test Complete ==="
