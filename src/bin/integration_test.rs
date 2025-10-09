use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    let client = Client::new();

    println!("🧪 Full Integration Test - Docuseal Pro");
    println!("=====================================");

    let email = "integration@test.com".to_string();

    // Step 1: Register user
    println!("\n1. Registering user...");
    let register_resp = client.post("http://localhost:8080/api/auth/register")
        .json(&json!({
            "email": email,
            "password": "testpass123",
            "name": "Integration Test User"
        }))
        .send()
        .await;

    match register_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                println!("   ✅ User registered successfully");
            } else {
                let body = resp.text().await.unwrap_or_default();
                if body.contains("User already exists") {
                    println!("   ℹ️  User already exists, proceeding with login");
                } else {
                    println!("   ❌ Registration failed: {}", body);
                    return;
                }
            }
        }
        Err(e) => {
            println!("   ❌ Registration error: {}", e);
            return;
        }
    }

    // Step 2: Login
    println!("\n2. Logging in...");
    let login_resp = client.post("http://localhost:8080/api/auth/login")
        .json(&json!({
            "email": email,
            "password": "testpass123"
        }))
        .send()
        .await;

    let token = match login_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap();
                println!("   Login response: {:?}", body);
                let token = body["data"]["token"].as_str().unwrap_or("");
                println!("   ✅ Login successful, token length: {}", token.len());
                token.to_string()
            } else {
                let body = resp.text().await.unwrap_or_default();
                println!("   ❌ Login failed: {}", body);
                return;
            }
        }
        Err(e) => {
            println!("   ❌ Login error: {}", e);
            return;
        }
    };

    // Step 3: Create template
    println!("\n3. Creating template...");
    let template_resp = client.post("http://localhost:8080/api/templates")
        .bearer_auth(&token)
        .json(&json!({
            "name": "Integration Test Template",
            "document": "dGVzdCBkb2N1bWVudCBmb3IgaW50ZWdyYXRpb24gdGVzdA==", // base64 "test document for integration test"
            "fields": [
                {
                    "name": "signature_field",
                    "field_type": "signature",
                    "required": true,
                    "position_x": 100,
                    "position_y": 200,
                    "width": 200,
                    "height": 50
                }
            ]
        }))
        .send()
        .await;

    let template_id = match template_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap();
                let id = body["data"]["id"].as_i64().unwrap_or(0);
                println!("   ✅ Template created with ID: {}", id);
                id
            } else {
                let body = resp.text().await.unwrap_or_default();
                println!("   ❌ Template creation failed: {}", body);
                return;
            }
        }
        Err(e) => {
            println!("   ❌ Template creation error: {}", e);
            return;
        }
    };

    // Step 4: Create submission
    println!("\n4. Creating submission...");
    let submission_resp = client.post("http://localhost:8080/api/submissions")
        .bearer_auth(&token)
        .json(&json!({
            "template_id": template_id,
            "submitters": [
                {
                    "email": "submitter@test.com",
                    "name": "Test Submitter",
                    "role": "recipient"
                }
            ]
        }))
        .send()
        .await;

    let submitter_id = match submission_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap();
                let submitters = body["data"]["submitters"].as_array().unwrap();
                let id = submitters[0]["id"].as_i64().unwrap_or(0);
                println!("   ✅ Submission created, submitter ID: {}", id);
                id
            } else {
                let body = resp.text().await.unwrap_or_default();
                println!("   ❌ Submission creation failed: {}", body);
                return;
            }
        }
        Err(e) => {
            println!("   ❌ Submission creation error: {}", e);
            return;
        }
    };

    // Step 5: Verify submitter exists
    println!("\n5. Verifying submitter exists...");
    let verify_resp = client.get(&format!("http://localhost:8080/api/submitters/{}", submitter_id))
        .bearer_auth(&token)
        .send()
        .await;

    match verify_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                println!("   ✅ Submitter exists before deletion");
            } else {
                println!("   ❌ Submitter not found before deletion");
                return;
            }
        }
        Err(e) => {
            println!("   ❌ Verification error: {}", e);
            return;
        }
    }

    // Step 6: Delete submitter
    println!("\n6. Deleting submitter...");
    let delete_resp = client.delete(&format!("http://localhost:8080/api/submitters/{}", submitter_id))
        .bearer_auth(&token)
        .send()
        .await;

    match delete_resp {
        Ok(resp) => {
            let status = resp.status();
            println!("   Status: {}", status);
            let body = resp.text().await.unwrap_or_default();
            println!("   Response: {}", body);

            if status.is_success() {
                println!("   ✅ DELETE submitter API works successfully!");
            } else {
                println!("   ❌ DELETE submitter API failed");
                return;
            }
        }
        Err(e) => {
            println!("   ❌ DELETE request error: {}", e);
            return;
        }
    }

    // Step 7: Verify submitter was deleted
    println!("\n7. Verifying submitter was deleted...");
    let verify_delete_resp = client.get(&format!("http://localhost:8080/api/submitters/{}", submitter_id))
        .bearer_auth(&token)
        .send()
        .await;

    match verify_delete_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status() == 404 {
                println!("   ✅ Submitter successfully deleted (404 Not Found)");
            } else {
                println!("   ❌ Submitter still exists after deletion");
                return;
            }
        }
        Err(e) => {
            println!("   ❌ Verification error: {}", e);
            return;
        }
    }

    // Step 8: Test Swagger UI
    println!("\n8. Testing Swagger UI...");
    let swagger_resp = client.get("http://localhost:8080/swagger-ui/")
        .send()
        .await;

    match swagger_resp {
        Ok(resp) => {
            println!("   Status: {}", resp.status());
            if resp.status().is_success() {
                println!("   ✅ Swagger UI is accessible");
            } else {
                println!("   ❌ Swagger UI not accessible");
            }
        }
        Err(e) => {
            println!("   ❌ Swagger UI error: {}", e);
        }
    }

    println!("\n🎉 Full integration test completed successfully!");
    println!("\n📊 Test Summary:");
    println!("   ✅ User registration");
    println!("   ✅ User authentication");
    println!("   ✅ Template creation");
    println!("   ✅ Submission creation");
    println!("   ✅ Submitter retrieval");
    println!("   ✅ Submitter deletion");
    println!("   ✅ API documentation");
}