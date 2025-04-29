#!/bin/bash

# API Tests using curl
# This script replicates the functionality of the yaak tests in api_tests.yaml

# ===== Environment variables =====
BASE_URL="${BASE_URL:-http://localhost:8000}"
USER_ID=""
POST_ID=""
COMMENT_ID=""
FOLLOWER_ID=""
FOLLOWING_ID=""

# Colors for terminal output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# ===== Helper functions =====

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is not installed.${NC}"
    echo "Please install jq to parse JSON responses properly:"
    echo "  - macOS: brew install jq"
    echo "  - Ubuntu/Debian: sudo apt-get install jq"
    echo "  - CentOS/RHEL: sudo yum install jq"
    exit 1
fi

# Check if the previous command was successful
check_status() {
    local exit_code=$?
    local test_name=$1
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ $test_name successful${NC}"
        return 0
    else
        echo -e "${RED}✗ $test_name failed with exit code $exit_code${NC}"
        return 1
    fi
}

# Execute curl request and process response
execute_request() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local extract_path=$5
    local extract_var=$6
    
    echo -e "\n${YELLOW}Executing: $name${NC}"
    echo "Request: $method $endpoint"
    
    local curl_cmd
    local response

    # Build curl command based on method and data
    if [ "$method" = "GET" ]; then
        curl_cmd="curl -s -X GET \"${BASE_URL}${endpoint}\""
    elif [ -n "$data" ]; then
        curl_cmd="curl -s -X $method \"${BASE_URL}${endpoint}\" -H \"Content-Type: application/json\" -d '$data'"
    else
        curl_cmd="curl -s -X $method \"${BASE_URL}${endpoint}\""
    fi
    
    # Execute the curl command and store the response
    response=$(eval $curl_cmd)
    check_status "$name" || return 1
    
    # Check for response status code in JSON (our API returns status field)
    status=$(echo "$response" | jq -r '.status' 2>/dev/null)
    if [ "$status" != "success" ]; then
        echo -e "${RED}API returned non-success status: $status${NC}"
        echo "Response: $response"
        return 1
    fi
    
    # If we need to extract a value from the response, do it
    if [ -n "$extract_path" ] && [ -n "$extract_var" ]; then
        local value
        value=$(echo "$response" | jq -r "$extract_path" 2>/dev/null)
        
        if [ -z "$value" ] || [ "$value" = "null" ]; then
            echo -e "${RED}Could not extract $extract_var using path $extract_path${NC}"
            echo "Response: $response"
            return 1
        fi
        
        # Export the variable for use in other functions
        export "$extract_var"="$value"
        echo "Extracted $extract_var: $value"
    fi
    
    echo "Response: $response"
    return 0
}

# ===== Test Functions =====

# Health Check Tests
health_check() {
    execute_request "Health Check" "GET" "/api/health" "" "" ""
}

# Database Management Tests
populate_database() {
    execute_request "Populate Database" "POST" "/api/database/populate" "" "" ""
}

# User Management Tests
create_user() {
    local data='{
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "hashedpassword123"
    }'
    execute_request "Create User" "POST" "/api/users" "$data" "$.data" "USER_ID"
}

get_all_users() {
    execute_request "Get All Users" "GET" "/api/users" "" "$.data[0]._id" "FOLLOWER_ID"
    execute_request "Get All Users (Second ID)" "GET" "/api/users" "" "$.data[1]._id" "FOLLOWING_ID"
}

get_user_by_id() {
    if [ -z "$USER_ID" ]; then
        echo -e "${RED}Error: USER_ID is not set. Run create_user first.${NC}"
        return 1
    fi
    execute_request "Get User by ID" "GET" "/api/users/$USER_ID" "" "" ""
}

# Post Management Tests
create_post() {
    if [ -z "$USER_ID" ]; then
        echo -e "${RED}Error: USER_ID is not set. Run create_user first.${NC}"
        return 1
    fi
    
    local data="{
        \"user_id\": \"$USER_ID\",
        \"content\": \"This is a test post content\",
        \"media_urls\": [],
        \"post_type\": \"Text\"
    }"
    execute_request "Create Post" "POST" "/api/posts" "$data" "$.data" "POST_ID"
}

get_all_posts() {
    execute_request "Get All Posts" "GET" "/api/posts" "" "" ""
}

get_post_by_id() {
    if [ -z "$POST_ID" ]; then
        echo -e "${RED}Error: POST_ID is not set. Run create_post first.${NC}"
        return 1
    fi
    execute_request "Get Post by ID" "GET" "/api/posts/$POST_ID" "" "" ""
}

get_posts_by_user_id() {
    if [ -z "$USER_ID" ]; then
        echo -e "${RED}Error: USER_ID is not set. Run create_user first.${NC}"
        return 1
    fi
    execute_request "Get Posts by User ID" "GET" "/api/users/posts/$USER_ID" "" "" ""
}

# Comment Management Tests
create_comment() {
    if [ -z "$USER_ID" ] || [ -z "$POST_ID" ]; then
        echo -e "${RED}Error: USER_ID or POST_ID not set. Run create_user and create_post first.${NC}"
        return 1
    fi
    
    local data="{
        \"post_id\": \"$POST_ID\",
        \"user_id\": \"$USER_ID\",
        \"content\": \"This is a test comment\"
    }"
    execute_request "Create Comment" "POST" "/api/comments" "$data" "$.data" "COMMENT_ID"
}

get_all_comments() {
    execute_request "Get All Comments" "GET" "/api/comments" "" "" ""
}

get_comments_by_post_id() {
    if [ -z "$POST_ID" ]; then
        echo -e "${RED}Error: POST_ID is not set. Run create_post first.${NC}"
        return 1
    fi
    execute_request "Get Comments by Post ID" "GET" "/api/posts/comments/$POST_ID" "" "" ""
}

get_comments_by_user_id() {
    if [ -z "$USER_ID" ]; then
        echo -e "${RED}Error: USER_ID is not set. Run create_user first.${NC}"
        return 1
    fi
    execute_request "Get Comments by User ID" "GET" "/api/users/comments/$USER_ID" "" "" ""
}

# Follow Management Tests
create_follow() {
    if [ -z "$FOLLOWER_ID" ] || [ -z "$FOLLOWING_ID" ]; then
        echo -e "${RED}Error: FOLLOWER_ID or FOLLOWING_ID not set. Run get_all_users first.${NC}"
        return 1
    fi
    
    local data="{
        \"follower_id\": \"$FOLLOWER_ID\",
        \"following_id\": \"$FOLLOWING_ID\"
    }"
    execute_request "Create Follow Relationship" "POST" "/api/follows" "$data" "" ""
}

get_following_users() {
    if [ -z "$FOLLOWER_ID" ]; then
        echo -e "${RED}Error: FOLLOWER_ID is not set. Run get_all_users first.${NC}"
        return 1
    fi
    execute_request "Get Following Users" "GET" "/api/users/following/$FOLLOWER_ID" "" "" ""
}

get_followers_users() {
    if [ -z "$FOLLOWING_ID" ]; then
        echo -e "${RED}Error: FOLLOWING_ID is not set. Run get_all_users first.${NC}"
        return 1
    fi
    execute_request "Get Followers Users" "GET" "/api/users/followers/$FOLLOWING_ID" "" "" ""
}

# Database Cleanup Tests
clean_database() {
    execute_request "Clean Database" "POST" "/api/database/clean" "" "" ""
}

# ===== Main Execution =====

# Display initial information
echo -e "${YELLOW}=== API Tests with curl ===${NC}"
echo "Base URL: $BASE_URL"

# Run tests in the right order
health_check || exit 1
populate_database || exit 1

# User Management Flow
create_user || exit 1
get_all_users || exit 1
get_user_by_id || exit 1

# Post Management Flow
create_post || exit 1
get_all_posts || exit 1
get_post_by_id || exit 1
get_posts_by_user_id || exit 1

# Comment Management Flow
create_comment || exit 1
get_all_comments || exit 1
get_comments_by_post_id || exit 1
get_comments_by_user_id || exit 1

# Follow Management Flow
create_follow || exit 1
get_following_users || exit 1
get_followers_users || exit 1

# Cleanup
clean_database || exit 1

echo -e "\n${GREEN}All tests completed successfully!${NC}"

