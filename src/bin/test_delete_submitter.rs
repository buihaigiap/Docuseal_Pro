use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    let client = Client::new();

    println!("🚀 Testing DELETE /submitters API");
    println!("==================================");

    // Test 1: Test with non-existent submitter ID
    println!("\n1. Testing DELETE /submitters/99999 (non-existent ID)");
    let resp = client.delete("http://localhost:8080/submitters/99999")
        .bearer_auth("fake-token")
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            println!("   Status: {}", status);
            let body = response.text().await.unwrap_or_default();
            println!("   Response: {}", body);

            if status == 401 {
                println!("   ✅ API endpoint exists (401 Unauthorized - authentication required)");
            } else if status == 404 {
                println!("   ✅ API endpoint exists (404 Not Found - submitter not found)");
            } else {
                println!("   ⚠️  Unexpected status: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Request error: {}", e);
            return;
        }
    }

    // Test 2: Test without authentication
    println!("\n2. Testing DELETE /submitters/1 (without authentication)");
    let resp = client.delete("http://localhost:8080/submitters/1")
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            println!("   Status: {}", status);
            if status == 401 {
                println!("   ✅ Authentication required (401 Unauthorized)");
            } else {
                println!("   ⚠️  Unexpected status: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Request error: {}", e);
        }
    }

    // Test 3: Test with invalid method
    println!("\n3. Testing GET /submitters/1 (wrong method)");
    let resp = client.get("http://localhost:8080/submitters/1")
        .bearer_auth("fake-token")
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            println!("   Status: {}", status);
            if status == 404 {
                println!("   ✅ GET method not allowed for this endpoint (404 Not Found)");
            } else {
                println!("   ⚠️  Unexpected status: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Request error: {}", e);
        }
    }

    println!("\n🎉 All basic tests completed!");
    println!("\n📝 Note: To test with real data, you need to:");
    println!("   1. Register a user: POST /auth/register");
    println!("   2. Login to get token: POST /auth/login");
    println!("   3. Create a template: POST /templates");
    println!("   4. Create a submission: POST /submissions");
    println!("   5. Then test DELETE /submitters/{{id}} with the real submitter ID");
}
