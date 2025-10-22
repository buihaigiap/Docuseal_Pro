#!/bin/bash

echo "=========================================="
echo "🧪 DOCUSEAL PRO - TEAM SHARING TEST"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Base URL
BASE_URL="http://localhost:8080"

# Function to make API calls
call_api() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local token="$4"

    if [ "$method" = "GET" ]; then
        curl -s -X GET "$BASE_URL$endpoint" ${token:+-H "Authorization: Bearer $token"}
    else
        curl -s -X $method "$BASE_URL$endpoint" -H "Content-Type: application/json" ${token:+-H "Authorization: Bearer $token"} -d "$data"
    fi
}

echo "1. Admin Login..."
ADMIN_LOGIN=$(call_api "POST" "/api/auth/login" '{"email": "newuser@example.com", "password": "test123"}' "")
ADMIN_TOKEN=$(echo "$ADMIN_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -n "$ADMIN_TOKEN" ]; then
    echo -e "${GREEN}✅ Admin logged in${NC}"
else
    echo -e "${RED}❌ Admin login failed${NC}"
    exit 1
fi

echo ""
echo "2. Creating Invited User..."
TIMESTAMP=$(date +%s)
INVITED_EMAIL="invited${TIMESTAMP}@example.com"

# Invite user
INVITE_RESULT=$(call_api "POST" "/api/users" "{\"email\": \"$INVITED_EMAIL\", \"name\": \"Invited User\", \"role\": \"editor\"}" "$ADMIN_TOKEN")
INVITE_CODE=$(echo "$INVITE_RESULT" | grep -o '"activation_code":"[^"]*' | cut -d'"' -f4)

if [ -n "$INVITE_CODE" ]; then
    echo -e "${GREEN}✅ User invited, code: $INVITE_CODE${NC}"

    # Activate user
    ACTIVATE_RESULT=$(call_api "POST" "/api/auth/activate" "{\"email\": \"$INVITED_EMAIL\", \"activation_code\": \"$INVITE_CODE\", \"password\": \"test123\"}" "")

    if echo "$ACTIVATE_RESULT" | grep -q '"message":"Account activated successfully'; then
        echo -e "${GREEN}✅ User activated${NC}"

        # Login as invited user
        USER_LOGIN=$(call_api "POST" "/api/auth/login" "{\"email\": \"$INVITED_EMAIL\", \"password\": \"test123\"}" "")
        USER_TOKEN=$(echo "$USER_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

        if [ -n "$USER_TOKEN" ]; then
            echo -e "${GREEN}✅ Invited user logged in${NC}"
        else
            echo -e "${RED}❌ Invited user login failed${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ User activation failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ User invitation failed${NC}"
    exit 1
fi

echo ""
echo "3. Creating Template as Admin..."
DOCUMENT_B64=$(echo '<html><body><h1>Team Template</h1><p>This should be visible to team members.</p></body></html>' | base64 -w 0)
CREATE_TEMPLATE=$(call_api "POST" "/api/templates" "{\"name\": \"Team Template\", \"document\": \"$DOCUMENT_B64\"}" "$ADMIN_TOKEN")
TEMPLATE_ID=$(echo "$CREATE_TEMPLATE" | grep -o '"id":[0-9]*' | cut -d':' -f2)

if [ -n "$TEMPLATE_ID" ]; then
    echo -e "${GREEN}✅ Template created, ID: $TEMPLATE_ID${NC}"
else
    echo -e "${RED}❌ Template creation failed${NC}"
    exit 1
fi

echo ""
echo "4. Testing Team Template Sharing..."

# Admin sees template
ADMIN_TEMPLATES=$(call_api "GET" "/api/templates" "" "$ADMIN_TOKEN")
ADMIN_SEES=$(echo "$ADMIN_TEMPLATES" | jq -r ".data[] | select(.id == $TEMPLATE_ID) | .name" 2>/dev/null)

if [ "$ADMIN_SEES" = "Team Template" ]; then
    echo -e "${GREEN}✅ Admin sees template${NC}"
else
    echo -e "${RED}❌ Admin cannot see template${NC}"
fi

# Invited user sees template
USER_TEMPLATES=$(call_api "GET" "/api/templates" "" "$USER_TOKEN")
USER_SEES=$(echo "$USER_TEMPLATES" | jq -r ".data[] | select(.id == $TEMPLATE_ID) | .name" 2>/dev/null)

if [ "$USER_SEES" = "Team Template" ]; then
    echo -e "${GREEN}✅ Invited user sees team template - TEAM SHARING WORKS!${NC}"
else
    echo -e "${RED}❌ Invited user cannot see team template${NC}"
    echo "User templates response: $USER_TEMPLATES"
fi

echo ""
echo "=========================================="
echo "Test completed!"