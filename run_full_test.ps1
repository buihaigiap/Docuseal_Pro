# Complete Workflow Script - DocuSeal Pro
# Register -> Login -> Create Template -> Upload PDF -> Create Fie# Step 4: Creating signature fields with positions (Bulk)
Write-Host ""
Write-Host "[Step 4] Creating signature fields with positions..." -ForegroundColor Yellow

$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

# Check if fields already exist and clean them up
Write-Host "Checking for existing fields..." -ForegroundColor Gray
try {
    $existingFieldsResponse = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method GET -Headers $headers
    $existingFields = $existingFieldsResponse.data
    
    if ($existingFields -and $existingFields.Count -gt 0) {
        Write-Host "Found $($existingFields.Count) existing fields. Cleaning up..." -ForegroundColor Yellow
        
        foreach ($field in $existingFields) {
            try {
                Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields/$($field.id)" -Method DELETE -Headers $headers | Out-Null
                Write-Host "  Deleted field: $($field.name) (ID: $($field.id))" -ForegroundColor Gray
            } catch {
                Write-Host "  Warning: Could not delete field $($field.name): $($_.Exception.Message)" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "No existing fields found." -ForegroundColor Gray
    }
} catch {
    Write-Host "Warning: Could not check existing fields: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Create all fields in bulkosition -> Send Email -> Sign

$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:8080"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DocuSeal Pro - Complete Workflow" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Generate unique email for testing
$timestamp = [DateTimeOffset]::Now.ToUnixTimeSeconds()
$testEmail = "test_$timestamp@example.com"
$testPassword = "Test123456!"
$testName = "Test User $timestamp"

# Step 1: Register User
Write-Host ""
Write-Host "[Step 1] Registering new user..." -ForegroundColor Yellow
Write-Host "Email: $testEmail" -ForegroundColor Gray

$registerBody = @{
    name = $testName
    email = $testEmail
    password = $testPassword
} | ConvertTo-Json

try {
    $registerResponse = Invoke-RestMethod -Uri "$baseUrl/api/auth/register" -Method POST -Body $registerBody -ContentType "application/json"
    Write-Host "Success User registered successfully!" -ForegroundColor Green
    Write-Host "User ID: $($registerResponse.data.id)" -ForegroundColor Gray
} catch {
    Write-Host "Error Registration failed: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Step 2: Login
Write-Host ""
Write-Host "[Step 2] Logging in..." -ForegroundColor Yellow

$loginBody = @{
    email = $testEmail
    password = $testPassword
} | ConvertTo-Json

try {
    $loginResponse = Invoke-RestMethod -Uri "$baseUrl/api/auth/login" -Method POST -Body $loginBody -ContentType "application/json"
    $token = $loginResponse.data.token
    Write-Host "Success Login successful!" -ForegroundColor Green
    Write-Host "Token: $($token.Substring(0, 20))..." -ForegroundColor Gray
} catch {
    Write-Host "Error Login failed: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Step 3: Upload PDF file
Write-Host ""
Write-Host "[Step 3] Uploading PDF file..." -ForegroundColor Yellow

$pdfPath = "./fixed_test.pdf"
if (-not (Test-Path $pdfPath)) {
    Write-Host "Error fixed_test.pdf not found at $pdfPath" -ForegroundColor Red
    exit
}

Write-Host "Uploading PDF..." -ForegroundColor Gray

try {
    # Use curl for reliable binary upload
    $curlArgs = @(
        '-X', 'POST',
        "$baseUrl/api/files/upload",
        '-H', "Authorization: Bearer $token",
        '-F', "file=@$pdfPath",
        '-s'  # Silent mode
    )
    
    $curlOutput = & curl @curlArgs 2>$null
    $uploadResult = $curlOutput | ConvertFrom-Json
    
    if ($uploadResult -and $uploadResult.data.id) {
        $fileId = $uploadResult.data.id
        $fileName = $uploadResult.data.filename
        $fileUrl = $uploadResult.data.url
        Write-Host "Success File uploaded successfully!" -ForegroundColor Green
        Write-Host "File ID: $fileId" -ForegroundColor Gray
        Write-Host "File Name: $fileName" -ForegroundColor Gray
        Write-Host "File URL: $fileUrl" -ForegroundColor Gray
        Write-Host "File Type: $($uploadResult.data.file_type)" -ForegroundColor Gray
        Write-Host "File Size: $($uploadResult.data.file_size) bytes" -ForegroundColor Gray
    } else {
        Write-Host "Error File upload failed!" -ForegroundColor Red
        Write-Host "Response: $curlOutput" -ForegroundColor Red
        exit
    }
} catch {
    Write-Host "Error uploading file: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Create test image file for image field
Write-Host ""
Write-Host "Creating test image file..." -ForegroundColor Gray
$testImagePath = "./test_image.png"
# Create a simple 1x1 pixel PNG as base64 (minimal valid PNG)
$pngBase64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
$pngBytes = [System.Convert]::FromBase64String($pngBase64)
[System.IO.File]::WriteAllBytes($testImagePath, $pngBytes)
Write-Host "Success Test image created at: $testImagePath" -ForegroundColor Green

# Step 4: Create Template from uploaded file
Write-Host ""
Write-Host "[Step 4] Creating template from uploaded file..." -ForegroundColor Yellow

$templateName = "Contract Template $timestamp"

$createTemplateBody = @{
    file_id = $fileId
    name = $templateName
} | ConvertTo-Json

$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

try {
    $createTemplateResponse = Invoke-RestMethod -Uri "$baseUrl/api/templates/from-file" -Method POST -Headers $headers -Body $createTemplateBody
    
    if ($createTemplateResponse -and $createTemplateResponse.data.id) {
        $templateId = $createTemplateResponse.data.id
        Write-Host "Success Template created successfully!" -ForegroundColor Green
        Write-Host "Template ID: $templateId" -ForegroundColor Gray
        Write-Host "Template Name: $($createTemplateResponse.data.name)" -ForegroundColor Gray
    } else {
        Write-Host "Error Template creation failed!" -ForegroundColor Red
        Write-Host "Response: $($createTemplateResponse | ConvertTo-Json)" -ForegroundColor Red
        exit
    }
} catch {
    Write-Host "Error creating template: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Step 5: Create Template Fields with Positions (Bulk)
Write-Host ""
Write-Host "[Step 5] Creating signature fields with positions..." -ForegroundColor Yellow

$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

# Check if fields already exist and clean them up
Write-Host "Checking for existing fields..." -ForegroundColor Gray
try {
    $existingFieldsResponse = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method GET -Headers $headers
    $existingFields = $existingFieldsResponse.data
    
    if ($existingFields -and $existingFields.Count -gt 0) {
        Write-Host "Found $($existingFields.Count) existing fields. Cleaning up..." -ForegroundColor Yellow
        
        foreach ($field in $existingFields) {
            try {
                Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields/$($field.id)" -Method DELETE -Headers $headers | Out-Null
                Write-Host "  Deleted field: $($field.name) (ID: $($field.id))" -ForegroundColor Gray
            } catch {
                Write-Host "  Warning: Could not delete field $($field.name): $($_.Exception.Message)" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "No existing fields found." -ForegroundColor Gray
    }
} catch {
    Write-Host "Warning: Could not check existing fields: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Create all fields in bulk with partner information
$bulkFieldsBody = @{
    fields = @(
        @{
            name = "buyer_signature"
            field_type = "signature"
            required = $true
            display_order = 1
            partner = "Buyer"  # Partner information
            position = @{
                x = 50.0
                y = 100.0
                width = 200.0
                height = 60.0
                page = 0
            }
        },
        @{
            name = "seller_signature"
            field_type = "signature"
            required = $true
            display_order = 2
            partner = "Seller"  # Partner information
            position = @{
                x = 50.0
                y = 300.0
                width = 200.0
                height = 60.0
                page = 0
            }
        },
        @{
            name = "witness_signature"
            field_type = "signature"
            required = $true
            display_order = 3
            partner = "Witness"  # Partner information
            position = @{
                x = 50.0
                y = 500.0
                width = 200.0
                height = 60.0
                page = 0
            }
        },
        @{
            name = "buyer_photo"
            field_type = "image"
            required = $false
            display_order = 4
            partner = "Buyer"  # Same partner as buyer signature
            position = @{
                x = 300.0
                y = 100.0
                width = 150.0
                height = 100.0
                page = 0
            }
        },
        @{
            name = "seller_company_stamp"
            field_type = "image"
            required = $false
            display_order = 5
            partner = "Seller"  # Seller's company stamp
            position = @{
                x = 300.0
                y = 300.0
                width = 150.0
                height = 100.0
                page = 0
            }
        },
        @{
            name = "contract_date"
            field_type = "date"
            required = $true
            display_order = 0
            # No partner - this is a general field
            position = @{
                x = 400.0
                y = 50.0
                width = 120.0
                height = 30.0
                page = 0
            }
        }
    )
} | ConvertTo-Json -Depth 10

try {
    $bulkFieldsResponse = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method POST -Headers $headers -Body $bulkFieldsBody
    Write-Host "Success Bulk fields created successfully!" -ForegroundColor Green
    
    # Extract field IDs from response and display partner information
    $createdFields = $bulkFieldsResponse.data
    $field1Id = $createdFields[0].id  # buyer_signature
    $field2Id = $createdFields[1].id  # seller_signature
    $field3Id = $createdFields[2].id  # witness_signature
    $field4Id = $createdFields[3].id  # buyer_photo
    $field5Id = $createdFields[4].id  # seller_company_stamp
    $field6Id = $createdFields[5].id  # contract_date
    
    Write-Host "Success Multi-Partner Fields Created:" -ForegroundColor Green
    Write-Host "  - Buyer Fields:" -ForegroundColor Cyan
    Write-Host "    * buyer_signature (ID: $field1Id) - Partner: Buyer" -ForegroundColor Gray
    Write-Host "    * buyer_photo (ID: $field4Id) - Partner: Buyer" -ForegroundColor Gray
    Write-Host "  - Seller Fields:" -ForegroundColor Cyan
    Write-Host "    * seller_signature (ID: $field2Id) - Partner: Seller" -ForegroundColor Gray
    Write-Host "    * seller_company_stamp (ID: $field5Id) - Partner: Seller" -ForegroundColor Gray
    Write-Host "  - Witness Fields:" -ForegroundColor Cyan
    Write-Host "    * witness_signature (ID: $field3Id) - Partner: Witness" -ForegroundColor Gray
    Write-Host "  - General Fields:" -ForegroundColor Cyan
    Write-Host "    * contract_date (ID: $field6Id) - No Partner" -ForegroundColor Gray
    
    # Display fields grouped by partner
    Write-Host ""
    Write-Host "Fields by Partner:" -ForegroundColor Yellow
    foreach ($field in $createdFields) {
        if ($field.partner) {
            Write-Host "  [$($field.partner)] $($field.name) ($($field.field_type)) - Required: $($field.required)" -ForegroundColor White
        } else {
            Write-Host "  [General] $($field.name) ($($field.field_type)) - Required: $($field.required)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "Error Failed to create fields: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

Write-Host ""
Write-Host "Summary of created fields:" -ForegroundColor Cyan
Write-Host "  1. buyer_signature (ID: $field1Id) - Top of page" -ForegroundColor Gray
Write-Host "  2. seller_signature (ID: $field2Id) - Middle of page" -ForegroundColor Gray
Write-Host "  3. witness_signature (ID: $field3Id) - Bottom of page" -ForegroundColor Gray
Write-Host "  4. buyer_photo (ID: $field4Id) - Top right (image field)" -ForegroundColor Gray

# Save field IDs to file for signing script
$allFieldIds = @($field1Id, $field2Id, $field3Id, $field4Id, $field5Id, $field6Id)
$allFieldIds | Out-File -FilePath "./field_ids.txt" -Encoding UTF8
Write-Host ""
Write-Host "All Field IDs saved to: ./field_ids.txt" -ForegroundColor Cyan

# Step 6: Create Multi-Partner Submission and Send Emails
Write-Host ""
Write-Host "[Step 6] Creating multi-partner submission with role-based assignments..." -ForegroundColor Yellow

$submissionBody = @{
    template_id = $templateId
    submitters = @(
        @{
            name = "Bui Hai Giap (Buyer)"
            email = "buihaigiap0101@gmail.com"
            role = "Buyer"  # Role matches partner field
        },
        @{
            name = "Seller Company Rep"
            email = "buihaigiap0102@gmail.com"
            role = "Seller"  # Role matches partner field
        },
        @{
            name = "Legal Witness"
            email = "buihaigiap0103@gmail.com"
            role = "Witness"  # Role matches partner field
        }
    )
} | ConvertTo-Json

try {
    $submissionResponse = Invoke-RestMethod -Uri "$baseUrl/api/submissions" -Method POST -Headers $headers -Body $submissionBody -ContentType "application/json"
    
    if ($submissionResponse -and $submissionResponse.data.id) {
        $submissionId = $submissionResponse.data.id
        Write-Host "Success Submission created successfully!" -ForegroundColor Green
        Write-Host "Submission ID: $submissionId" -ForegroundColor Gray
        
        if ($submissionResponse.data.submitters -and $submissionResponse.data.submitters.Count -gt 0) {
            Write-Host "Multi-Partner Submitters Created:" -ForegroundColor Cyan
            $tokens = @()
            
            for ($i = 0; $i -lt $submissionResponse.data.submitters.Count; $i++) {
                $submitter = $submissionResponse.data.submitters[$i]
                $tokens += $submitter.token
                $role = if ($submitter.role) { $submitter.role } else { "General" }
                Write-Host "  $($i+1). [$role] $($submitter.name) - $($submitter.email)" -ForegroundColor White
                Write-Host "     Token: $($submitter.token)" -ForegroundColor Cyan
                Write-Host "     Status: $($submitter.status)" -ForegroundColor Gray
            }
            
            Write-Host ""
            Write-Host "Partner-Based Email Notifications Sent:" -ForegroundColor Green
            Write-Host "  🏢 Buyer: buihaigiap0101@gmail.com (responsible for buyer fields)" -ForegroundColor Gray
            Write-Host "  🏭 Seller: buihaigiap0102@gmail.com (responsible for seller fields)" -ForegroundColor Gray
            Write-Host "  👨‍💼 Witness: buihaigiap0103@gmail.com (responsible for witness fields)" -ForegroundColor Gray
            
            Write-Host ""
            Write-Host "Signing Workflow:" -ForegroundColor Yellow
            Write-Host "  1. Each partner receives an email with their specific signing link" -ForegroundColor Gray
            Write-Host "  2. Buyer will only see and sign buyer-related fields" -ForegroundColor Gray
            Write-Host "  3. Seller will only see and sign seller-related fields" -ForegroundColor Gray
            Write-Host "  4. Witness will only see and sign witness-related fields" -ForegroundColor Gray
            Write-Host "  5. General fields (like date) can be filled by any partner" -ForegroundColor Gray
            
            # Save all tokens to file (one per line)
            $tokens | Out-File -FilePath "./submitter_token.txt" -Encoding UTF8
            Write-Host "All tokens saved to: ./submitter_token.txt" -ForegroundColor Cyan
        }
    } else {
        Write-Host "Error Submission creation failed!" -ForegroundColor Red
        exit
    }
} catch {
    Write-Host "Error creating submission: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Step 6: Get Public Submission Info
Write-Host ""
Write-Host "[Step 6] Getting public submission info for first submitter..." -ForegroundColor Yellow

try {
    $firstToken = $tokens[0]
    $publicSubmissionResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$firstToken" -Method GET
    Write-Host "Success Public submission retrieved!" -ForegroundColor Green
    Write-Host "Template: $($publicSubmissionResponse.data.template.name)" -ForegroundColor Gray
    Write-Host "Submitter: $($publicSubmissionResponse.data.submitter.name) ($($publicSubmissionResponse.data.submitter.email))" -ForegroundColor Gray
    Write-Host "Status: $($publicSubmissionResponse.data.submitter.status)" -ForegroundColor Gray
    Write-Host "Fields to sign: $($publicSubmissionResponse.data.template.template_fields.Count)" -ForegroundColor Gray
    Write-Host "Total submitters in this submission: 2" -ForegroundColor Gray
    
    Write-Host ""
    Write-Host "Field details:" -ForegroundColor Cyan
    foreach ($field in $publicSubmissionResponse.data.template.template_fields) {
        Write-Host "  - $($field.name) (ID: $($field.id), Type: $($field.field_type))" -ForegroundColor Gray
        if ($field.position) {
            Write-Host "    Position: page $($field.position.page), x:$($field.position.x), y:$($field.position.y)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "Warning Could not get public submission info: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "WORKFLOW COMPLETED SUCCESSFULLY!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Success User registered: $testEmail" -ForegroundColor Green
Write-Host "Success User logged in" -ForegroundColor Green
Write-Host "Success Template created (ID: $templateId)" -ForegroundColor Green
Write-Host "Success PDF uploaded to S3/MinIO" -ForegroundColor Green
Write-Host "Success 4 signature and image fields created with positions" -ForegroundColor Green
Write-Host "Success Submission created (ID: $submissionId)" -ForegroundColor Green
Write-Host "Success Emails sent to 2 different Gmail addresses:" -ForegroundColor Green
Write-Host "  - buihaigiap0101@gmail.com" -ForegroundColor Gray
Write-Host "  - buihaigiap0102@gmail.com" -ForegroundColor Gray
Write-Host ""
Write-Host "NEXT STEPS:" -ForegroundColor Yellow
Write-Host "1. Check emails at both Gmail addresses" -ForegroundColor White
Write-Host "2. Or sign programmatically with the tokens saved in submitter_token.txt" -ForegroundColor White
Write-Host ""
Write-Host "To sign now, run:" -ForegroundColor White
Write-Host "   .\sign_simple.ps1" -ForegroundColor Cyan
Write-Host ""
Write-Host "Field IDs: $($fieldIds -join ', ')" -ForegroundColor Gray
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
