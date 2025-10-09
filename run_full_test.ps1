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

$pdfPath = "/home/giap/giap/Docuseal_Pro/fixed_test.pdf"
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

# Create all fields in bulk
$bulkFieldsBody = @{
    fields = @(
        @{
            name = "buyer_signature"
            field_type = "signature"
            required = $true
            display_order = 1
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
            position = @{
                x = 50.0
                y = 500.0
                width = 200.0
                height = 60.0
                page = 0
            }
        }
    )
} | ConvertTo-Json -Depth 10

try {
    $bulkFieldsResponse = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method POST -Headers $headers -Body $bulkFieldsBody
    Write-Host "Success Bulk fields created successfully!" -ForegroundColor Green
    
    # Extract field IDs from response
    $createdFields = $bulkFieldsResponse.data
    $field1Id = $createdFields[0].id
    $field2Id = $createdFields[1].id
    $field3Id = $createdFields[2].id
    
    Write-Host "Success Field 1 created: buyer_signature (page 0, x:50, y:100)" -ForegroundColor Green
    Write-Host "Success Field 2 created: seller_signature (page 0, x:50, y:300)" -ForegroundColor Green
    Write-Host "Success Field 3 created: witness_signature (page 0, x:50, y:500)" -ForegroundColor Green
} catch {
    Write-Host "Error Failed to create fields: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

Write-Host ""
Write-Host "Summary of created fields:" -ForegroundColor Cyan
Write-Host "  1. buyer_signature (ID: $field1Id) - Top of page" -ForegroundColor Gray
Write-Host "  2. seller_signature (ID: $field2Id) - Middle of page" -ForegroundColor Gray
Write-Host "  3. witness_signature (ID: $field3Id) - Bottom of page" -ForegroundColor Gray

# Save field IDs to file for signing script
$fieldIds = @($field1Id, $field2Id, $field3Id)
$fieldIds | Out-File -FilePath "/home/giap/giap/Docuseal_Pro/field_ids.txt" -Encoding UTF8
Write-Host ""
Write-Host "Field IDs saved to: /home/giap/giap/Docuseal_Pro/field_ids.txt" -ForegroundColor Cyan

# Step 5: Create Submission and Send Email
Write-Host ""
Write-Host "[Step 5] Creating submission and sending emails to 2 different Gmail addresses..." -ForegroundColor Yellow

$submissionBody = @{
    template_id = $templateId
    submitters = @(
        @{
            name = "Bui Hai Giap"
            email = "buihaigiap0101@gmail.com"
        },
        @{
            name = "Test User 2"
            email = "buihaigiap0102@gmail.com"
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
            Write-Host "Submitters created:" -ForegroundColor Cyan
            $tokens = @()
            
            for ($i = 0; $i -lt $submissionResponse.data.submitters.Count; $i++) {
                $submitter = $submissionResponse.data.submitters[$i]
                $tokens += $submitter.token
                Write-Host "  $($i+1). $($submitter.name) - $($submitter.email)" -ForegroundColor Gray
                Write-Host "     Token: $($submitter.token)" -ForegroundColor Cyan
            }
            
            Write-Host ""
            Write-Host "Emails sent to:" -ForegroundColor Green
            Write-Host "  - buihaigiap0101@gmail.com" -ForegroundColor Gray
            Write-Host "  - buihaigiap0102@gmail.com" -ForegroundColor Gray
            
            # Save all tokens to file (one per line)
            $tokens | Out-File -FilePath "/home/giap/giap/Docuseal_Pro/submitter_token.txt" -Encoding UTF8
            Write-Host "Tokens saved to: /home/giap/giap/Docuseal_Pro/submitter_token.txt" -ForegroundColor Cyan
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
Write-Host "Success 3 signature fields created with positions" -ForegroundColor Green
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
