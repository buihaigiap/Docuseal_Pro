#!/bin/bash

echo "=========================================="
echo "🧪 DOCUSEAL PRO # Test 2: Invited User Login (create new invited user for testing)
echo "2. Testing Invited User Creation and Login..."
TIMESTAMP=$(date +%s)
INVITED_EMAIL="invited${TIMESTAMP}@example.com"
INVITED_NAME="Invited User ${TIMESTAMP}"

# Invite user
INVITE_RESULT=$(call_api "POST" "/api/users" "{\"email\": \"$INVITED_EMAIL\", \"name\": \"$INVITED_NAME\", \"role\": \"editor\"}" "$ADMIN_TOKEN")
INVITE_CODE=$(echo "$INVITE_RESULT" | grep -o '"activation_code":"[^"]*' | cut -d'"' -f4)

if [ -n "$INVITE_CODE" ]; then
    # Activate user
    ACTIVATE_RESULT=$(call_api "POST" "/api/auth/activate" "{\"email\": \"$INVITED_EMAIL\", \"activation_code\": \"$INVITE_CODE\", \"password\": \"test123\"}" "")
    ACTIVATE_SUCCESS=$(echo "$ACTIVATE_RESULT" | grep -o '"message":"Account activated successfully')

    if [ -n "$ACTIVATE_SUCCESS" ]; then
        # Login
        USER_LOGIN=$(call_api "POST" "/api/auth/login" "{\"email\": \"$INVITED_EMAIL\", \"password\": \"test123\"}" "")
        USER_TOKEN=$(echo "$USER_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

        if [ -n "$USER_TOKEN" ]; then
            print_test_result "Invited User Creation & Login" "PASS" "Created and logged in: $INVITED_EMAIL"
        else
            print_test_result "Invited User Creation & Login" "FAIL" "Login failed: $USER_LOGIN"
            USER_TOKEN=""
        fi
    else
        print_test_result "Invited User Creation & Login" "FAIL" "Activation failed: $ACTIVATE_RESULT"
        USER_TOKEN=""
    fi
else
    print_test_result "Invited User Creation & Login" "FAIL" "Invitation failed: $INVITE_RESULT"
    USER_TOKEN=""
fiNSIVE TEST SUITE"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base URL
BASE_URL="http://localhost:8080"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0

# Function to print test result
print_test_result() {
    local test_name="$1"
    local result="$2"
    local message="$3"

    ((TOTAL_TESTS++))
    if [ "$result" = "PASS" ]; then
        ((PASSED_TESTS++))
        echo -e "${GREEN}✅ PASS${NC} - $test_name"
        [ -n "$message" ] && echo -e "   $message"
    else
        echo -e "${RED}❌ FAIL${NC} - $test_name"
        [ -n "$message" ] && echo -e "   $message"
    fi
    echo ""
}

# Function to make API call
call_api() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local token="$4"

    local url="$BASE_URL$endpoint"
    local auth_header=""

    if [ -n "$token" ]; then
        auth_header="-H \"Authorization: Bearer $token\""
    fi

    if [ "$method" = "GET" ]; then
        eval "curl -s -X GET \"$url\" $auth_header"
    else
        eval "curl -s -X $method \"$url\" -H \"Content-Type: application/json\" $auth_header -d '$data'"
    fi
}

echo "🔍 Checking server status..."
LOGIN_TEST=$(call_api "POST" "/api/auth/login" '{"email": "newuser@example.com", "password": "test123"}' "")
if echo "$LOGIN_TEST" | grep -q '"success":true'; then
    print_test_result "Server Health Check" "PASS" "Server is running and API responding"
else
    print_test_result "Server Health Check" "FAIL" "Server API not responding"
    exit 1
fi

echo "=========================================="
echo "🔐 AUTHENTICATION TESTS"
echo "=========================================="

# Test 1: Admin Login
echo "1. Testing Admin Login..."
ADMIN_LOGIN=$(call_api "POST" "/api/auth/login" '{"email": "newuser@example.com", "password": "test123"}' "")
ADMIN_TOKEN=$(echo "$ADMIN_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -n "$ADMIN_TOKEN" ]; then
    print_test_result "Admin Login" "PASS" "Token: ${ADMIN_TOKEN:0:20}..."
else
    print_test_result "Admin Login" "FAIL" "Response: $ADMIN_LOGIN"
    exit 1
fi

# Test 2: Invited User Login
echo "2. Testing Invited User Login..."
USER_LOGIN=$(call_api "POST" "/api/auth/login" '{"email": "rewowe2165@fixwap.com", "password": "test123"}' "")
USER_TOKEN=$(echo "$USER_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -n "$USER_TOKEN" ]; then
    print_test_result "Invited User Login" "PASS" "Token: ${USER_TOKEN:0:20}..."
else
    print_test_result "Invited User Login" "FAIL" "Response: $USER_LOGIN"
fi

echo "=========================================="
echo "👥 USER INVITATION TESTS"
echo "=========================================="

# Test 3: Invite New User
echo "3. Testing User Invitation..."
TIMESTAMP=$(date +%s)
TEST_EMAIL="test${TIMESTAMP}@example.com"
TEST_NAME="Test User ${TIMESTAMP}"

INVITE_RESPONSE=$(call_api "POST" "/api/users" "{\"email\": \"$TEST_EMAIL\", \"name\": \"$TEST_NAME\", \"role\": \"editor\"}" "$ADMIN_TOKEN")
INVITE_SUCCESS=$(echo "$INVITE_RESPONSE" | grep -o '"success":true')

if [ -n "$INVITE_SUCCESS" ]; then
    ACTIVATION_CODE=$(echo "$INVITE_RESPONSE" | grep -o '"activation_code":"[^"]*' | cut -d'"' -f4)
    print_test_result "User Invitation" "PASS" "Invited: $TEST_EMAIL, Code: $ACTIVATION_CODE"
else
    print_test_result "User Invitation" "FAIL" "Response: $INVITE_RESPONSE"
    ACTIVATION_CODE=""
fi

# Test 4: Activate User Account
if [ -n "$ACTIVATION_CODE" ]; then
    echo "4. Testing User Activation..."
    ACTIVATE_RESPONSE=$(call_api "POST" "/api/auth/activate" "{\"email\": \"$TEST_EMAIL\", \"activation_code\": \"$ACTIVATION_CODE\", \"password\": \"testpass123\"}" "")
    ACTIVATE_SUCCESS=$(echo "$ACTIVATE_RESPONSE" | grep -o '"message":"Account activated successfully')

    if [ -n "$ACTIVATE_SUCCESS" ]; then
        print_test_result "User Activation" "PASS" "Activated: $TEST_EMAIL"
    else
        print_test_result "User Activation" "FAIL" "Response: $ACTIVATE_RESPONSE"
    fi
else
    print_test_result "User Activation" "SKIP" "No activation code available"
fi

echo "=========================================="
echo "📄 TEMPLATE SHARING TESTS"
echo "=========================================="

# Test 5: Admin Creates Template
echo "5. Testing Template Creation (Admin)..."
TEMPLATE_NAME="Test Template $(date +%s)"
# Create a simple base64 encoded HTML document
DOCUMENT_B64=$(echo '<html><body><h1>Test Document</h1><p>This is a test template.</p></body></html>' | base64 -w 0)
CREATE_TEMPLATE=$(call_api "POST" "/api/templates" "{\"name\": \"$TEMPLATE_NAME\", \"document\": \"$DOCUMENT_B64\", \"description\": \"Test template for team sharing\"}" "$ADMIN_TOKEN")
TEMPLATE_SUCCESS=$(echo "$CREATE_TEMPLATE" | grep -o '"success":true')

if [ -n "$TEMPLATE_SUCCESS" ]; then
    TEMPLATE_ID=$(echo "$CREATE_TEMPLATE" | grep -o '"id":[0-9]*' | cut -d':' -f2)
    print_test_result "Template Creation" "PASS" "Created template ID: $TEMPLATE_ID"
else
    print_test_result "Template Creation" "FAIL" "Response: $CREATE_TEMPLATE"
    TEMPLATE_ID=""
fi

# Test 6: Admin Sees Own Template
if [ -n "$TEMPLATE_ID" ]; then
    echo "6. Testing Admin Template Visibility..."
    ADMIN_TEMPLATES=$(call_api "GET" "/api/templates" "" "$ADMIN_TOKEN")
    ADMIN_SEES_TEMPLATE=$(echo "$ADMIN_TEMPLATES" | jq ".data[] | select(.id == $TEMPLATE_ID) | .name")

    if [ -n "$ADMIN_SEES_TEMPLATE" ]; then
        print_test_result "Admin Template Visibility" "PASS" "Admin sees template: $ADMIN_SEES_TEMPLATE"
    else
        print_test_result "Admin Template Visibility" "FAIL" "Admin cannot see own template"
    fi
else
    print_test_result "Admin Template Visibility" "SKIP" "No template created"
fi

# Test 7: Invited User Sees Team Template
if [ -n "$TEMPLATE_ID" ] && [ -n "$USER_TOKEN" ]; then
    echo "7. Testing Team Template Sharing..."
    USER_TEMPLATES=$(call_api "GET" "/api/templates" "" "$USER_TOKEN")
    USER_SEES_TEMPLATE=$(echo "$USER_TEMPLATES" | jq ".data[] | select(.id == $TEMPLATE_ID) | .name")

    if [ -n "$USER_SEES_TEMPLATE" ]; then
        print_test_result "Team Template Sharing" "PASS" "Invited user sees admin's template: $USER_SEES_TEMPLATE"
    else
        print_test_result "Team Template Sharing" "FAIL" "Invited user cannot see team template"
    fi
else
    print_test_result "Team Template Sharing" "SKIP" "Missing template or user token"
fi

# Test 8: New User Login and Template Access
if [ -n "$ACTIVATION_CODE" ] && [ -n "$TEMPLATE_ID" ]; then
    echo "8. Testing New User Login and Template Access..."
    NEW_USER_LOGIN=$(call_api "POST" "/api/auth/login" "{\"email\": \"$TEST_EMAIL\", \"password\": \"testpass123\"}" "")
    NEW_USER_TOKEN=$(echo "$NEW_USER_LOGIN" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

    if [ -n "$NEW_USER_TOKEN" ]; then
        NEW_USER_TEMPLATES=$(call_api "GET" "/api/templates" "" "$NEW_USER_TOKEN")
        NEW_USER_SEES_TEMPLATE=$(echo "$NEW_USER_TEMPLATES" | jq ".data[] | select(.id == $TEMPLATE_ID) | .name")

        if [ -n "$NEW_USER_SEES_TEMPLATE" ]; then
            print_test_result "New User Template Access" "PASS" "New user sees team template: $NEW_USER_SEES_TEMPLATE"
        else
            print_test_result "New User Template Access" "FAIL" "New user cannot see team template"
        fi
    else
        print_test_result "New User Template Access" "FAIL" "New user login failed: $NEW_USER_LOGIN"
    fi
else
    print_test_result "New User Template Access" "SKIP" "No new user created or no template available"
fi

echo "=========================================="
echo "📊 TEST RESULTS SUMMARY"
echo "=========================================="

echo "Total Tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $((TOTAL_TESTS - PASSED_TESTS))"

if [ "$PASSED_TESTS" -eq "$TOTAL_TESTS" ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo ""
    echo "✅ User invitation system working"
    echo "✅ Team-based template sharing working"
    echo "✅ Authentication system working"
else
    echo -e "${RED}⚠️  SOME TESTS FAILED${NC}"
    echo ""
    echo "❌ Check failed tests above"
fi

echo ""
echo "=========================================="
echo "🔧 TEST DATA CLEANUP"
echo "=========================================="

# Cleanup test data
# if [ -n "$TEMPLATE_ID" ]; then
#     echo "Cleaning up test template..."
#     DELETE_RESULT=$(call_api "DELETE" "/api/templates/$TEMPLATE_ID" "" "$ADMIN_TOKEN")
#     echo "Template cleanup: $(echo "$DELETE_RESULT" | grep -o '"success":true' || echo 'Failed')"
# fi

echo ""
echo "Test completed at $(date)"