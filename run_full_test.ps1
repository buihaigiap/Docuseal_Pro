# Complete Workflow Script - DocuSeal Pro
# Register -> Login -> Create Template -> Upload PDF -> Create Fields with Position -> Send Email -> Sign

$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:3000"

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

# Step 3: Upload PDF and Create Template
Write-Host ""
Write-Host "[Step 3] Creating template from PDF..." -ForegroundColor Yellow

$pdfPath = "d:\Docuseal_Pro\test.pdf"
if (-not (Test-Path $pdfPath)) {
    Write-Host "Error test.pdf not found at $pdfPath" -ForegroundColor Red
    exit
}

$templateName = "Contract Template $timestamp"

Write-Host "Uploading PDF..." -ForegroundColor Gray

try {
    $boundary = [System.Guid]::NewGuid().ToString()
    $LF = "`r`n"
    $pdfBytes = [System.IO.File]::ReadAllBytes($pdfPath)
    $pdfEnc = [System.Text.Encoding]::GetEncoding("iso-8859-1").GetString($pdfBytes)
    
    $bodyLines = @(
        "--$boundary",
        "Content-Disposition: form-data; name=`"pdf`"; filename=`"test.pdf`"",
        "Content-Type: application/pdf",
        "",
        $pdfEnc,
        "--$boundary",
        "Content-Disposition: form-data; name=`"name`"",
        "",
        $templateName,
        "--$boundary--"
    )
    
    $body = $bodyLines -join $LF
    $headers = @{ "Authorization" = "Bearer $token" }
    
    $createTemplateResult = Invoke-RestMethod -Uri "$baseUrl/api/templates/pdf" -Method POST -Headers $headers -Body $body -ContentType "multipart/form-data; boundary=$boundary"
    
    if ($createTemplateResult -and $createTemplateResult.data.id) {
        $templateId = $createTemplateResult.data.id
        Write-Host "Success Template created successfully!" -ForegroundColor Green
        Write-Host "Template ID: $templateId" -ForegroundColor Gray
        Write-Host "Template Name: $($createTemplateResult.data.name)" -ForegroundColor Gray
    } else {
        Write-Host "Error Template creation failed!" -ForegroundColor Red
        exit
    }
} catch {
    Write-Host "Error creating template: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Step 4: Create Template Fields with Positions
Write-Host ""
Write-Host "[Step 4] Creating signature fields with positions..." -ForegroundColor Yellow

$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

# Field 1: Buyer Signature
$field1Body = @{
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
} | ConvertTo-Json

try {
    $field1Response = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method POST -Headers $headers -Body $field1Body
    Write-Host "Success Field 1 created: buyer_signature (page 0, x:50, y:100)" -ForegroundColor Green
    $field1Id = $field1Response.data.id
} catch {
    Write-Host "Error Failed to create field 1: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Field 2: Seller Signature
$field2Body = @{
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
} | ConvertTo-Json

try {
    $field2Response = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method POST -Headers $headers -Body $field2Body
    Write-Host "Success Field 2 created: seller_signature (page 0, x:50, y:300)" -ForegroundColor Green
    $field2Id = $field2Response.data.id
} catch {
    Write-Host "Error Failed to create field 2: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Field 3: Witness Signature
$field3Body = @{
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
} | ConvertTo-Json

try {
    $field3Response = Invoke-RestMethod -Uri "$baseUrl/api/templates/$templateId/fields" -Method POST -Headers $headers -Body $field3Body
    Write-Host "Success Field 3 created: witness_signature (page 0, x:50, y:500)" -ForegroundColor Green
    $field3Id = $field3Response.data.id
} catch {
    Write-Host "Error Failed to create field 3: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

Write-Host ""
Write-Host "Summary of created fields:" -ForegroundColor Cyan
Write-Host "  1. buyer_signature (ID: $field1Id) - Top of page" -ForegroundColor Gray
Write-Host "  2. seller_signature (ID: $field2Id) - Middle of page" -ForegroundColor Gray
Write-Host "  3. witness_signature (ID: $field3Id) - Bottom of page" -ForegroundColor Gray

# Save field IDs to file for signing script
$fieldIds = @($field1Id, $field2Id, $field3Id)
$fieldIds | Out-File -FilePath "d:\Docuseal_Pro\field_ids.txt" -Encoding UTF8
Write-Host ""
Write-Host "Field IDs saved to: d:\Docuseal_Pro\field_ids.txt" -ForegroundColor Cyan

# Step 5: Create Submission and Send Email
Write-Host ""
Write-Host "[Step 5] Creating submission and sending email to buihaigiap0101@gmail.com..." -ForegroundColor Yellow

$submissionBody = @{
    template_id = $templateId
    submitters = @(
        @{
            name = "Bui Hai Giap"
            email = "buihaigiap0101@gmail.com"
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
            $submitterToken = $submissionResponse.data.submitters[0].token
            Write-Host "Submitter Token: $submitterToken" -ForegroundColor Cyan
            Write-Host ""
            Write-Host "Email sent to buihaigiap0101@gmail.com" -ForegroundColor Green
            
            $submitterToken | Out-File -FilePath "d:\Docuseal_Pro\submitter_token.txt" -Encoding UTF8
            Write-Host "Token saved to: d:\Docuseal_Pro\submitter_token.txt" -ForegroundColor Cyan
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
Write-Host "[Step 6] Getting public submission info..." -ForegroundColor Yellow

try {
    $publicSubmissionResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$submitterToken" -Method GET
    Write-Host "Success Public submission retrieved!" -ForegroundColor Green
    Write-Host "Template: $($publicSubmissionResponse.data.template_name)" -ForegroundColor Gray
    Write-Host "Submitter: $($publicSubmissionResponse.data.submitter.name)" -ForegroundColor Gray
    Write-Host "Status: $($publicSubmissionResponse.data.submitter.status)" -ForegroundColor Gray
    Write-Host "Fields to sign: $($publicSubmissionResponse.data.fields.Count)" -ForegroundColor Gray
    
    Write-Host ""
    Write-Host "Field details:" -ForegroundColor Cyan
    foreach ($field in $publicSubmissionResponse.data.fields) {
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
Write-Host "Success Email sent to buihaigiap0101@gmail.com" -ForegroundColor Green
Write-Host ""
Write-Host "NEXT STEPS:" -ForegroundColor Yellow
Write-Host "1. Check email at buihaigiap0101@gmail.com" -ForegroundColor White
Write-Host "2. Or sign programmatically with token: $submitterToken" -ForegroundColor White
Write-Host ""
Write-Host "To sign now, run:" -ForegroundColor White
Write-Host "   .\sign_simple.ps1" -ForegroundColor Cyan
Write-Host ""
Write-Host "Field IDs: $($fieldIds -join ', ')" -ForegroundColor Gray
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
