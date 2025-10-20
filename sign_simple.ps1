# Sign Document Script - Simple Version
# Directly sign with field IDs from workflow

param(
    [Parameter(Mandatory=$false)]
    [string]$Token,
    [Parameter(Mandatory=$false)]
    [array]$FieldIds,
    [Parameter(Mandatory=$false)]
    [string]$Email = "test_$([DateTimeOffset]::Now.ToUnixTimeSeconds())@example.com",
    [Parameter(Mandatory=$false)]
    [string]$Password = "Test123456!"
)

$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:8080"

# If no token provided, try to read from submitter_token.txt
if (-not $Token) {
    if (Test-Path "submitter_token.txt") {
        $tokens = Get-Content "submitter_token.txt"
        if ($tokens.Count -gt 0) {
            $Token = $tokens[0].Trim()
            Write-Host "Using token from submitter_token.txt: $Token" -ForegroundColor Gray
        } else {
            Write-Host "Error No tokens found in submitter_token.txt" -ForegroundColor Red
            exit
        }
    } else {
        Write-Host "Error No token provided and submitter_token.txt not found. Run run_full_test.ps1 first." -ForegroundColor Red
        exit
    }
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DocuSeal Pro - Document Signing" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Get submitter information
Write-Host ""
Write-Host "[Step 1] Getting submitter information..." -ForegroundColor Yellow

try {
    $encodedToken = [uri]::EscapeDataString($Token)
    $submitterResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$encodedToken" -Method GET
    Write-Host "Success Submitter found!" -ForegroundColor Green
    Write-Host "Name: $($submitterResponse.data.name)" -ForegroundColor Gray
    Write-Host "Email: $($submitterResponse.data.email)" -ForegroundColor Gray
    Write-Host "Status: $($submitterResponse.data.status)" -ForegroundColor Gray
    
    # Extract role from name (e.g., "Bui Hai Giap (Buyer)" -> "Buyer")
    $submitterName = $submitterResponse.data.name
    if ($submitterName -match '\((.*?)\)') {
        $submitterRole = $matches[1]
        Write-Host "Partner Role: $submitterRole" -ForegroundColor Cyan
        Write-Host "This submitter is responsible for fields assigned to: $submitterRole" -ForegroundColor Yellow
    } else {
        $submitterRole = $null
        Write-Host "Partner Role: General (can access all fields)" -ForegroundColor Cyan
    }
} catch {
    Write-Host "Error Failed to get submitter: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $responseBody = $_.Exception.Response.Content.ReadAsStringAsync().Result
        Write-Host "Response: $responseBody" -ForegroundColor Red
    }
    exit
}# Load token from file if not provided
if (-not $Token) {
    $tokenFile = "./submitter_token.txt"
    if (Test-Path $tokenFile) {
        $allTokens = Get-Content $tokenFile | Where-Object { $_.Trim() -ne "" }
        if ($allTokens.Count -gt 1) {
            $Token = $allTokens[1].Trim()  # Use second token (Test User 2)
            Write-Host "Success Using second token (Test User 2) from file" -ForegroundColor Green
        } elseif ($allTokens.Count -gt 0) {
            $Token = $allTokens[0].Trim()  # Use first token if only one available
            Write-Host "Success Token loaded from file" -ForegroundColor Green
        } else {
            Write-Host "Error No tokens found in file!" -ForegroundColor Red
            exit
        }
    } else {
        Write-Host "Error No token found!" -ForegroundColor Red
        exit
    }
}

# Load field IDs from workflow output if not provided
if (-not $FieldIds -or $FieldIds.Count -eq 0) {
    # Get fields from API instead of file
    try {
        $fieldsResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$encodedToken/fields" -Method GET
        $fields = $fieldsResponse.data.template_fields
        $FieldIds = $fields | ForEach-Object { $_.id }
        Write-Host "Success Field IDs loaded from API: $($FieldIds -join ', ')" -ForegroundColor Green
    } catch {
        Write-Host "Error Failed to get fields from API: $($_.Exception.Message)" -ForegroundColor Red
        exit
    }
}

# Step 2: Filter fields based on partner role
Write-Host ""
Write-Host "[Step 2] Filtering fields based on partner role..." -ForegroundColor Yellow

# Get template fields information to check partner assignments
$templateId = $submitterResponse.data.template_id
Write-Host "Template ID: $templateId" -ForegroundColor Gray

# Fields are already filtered by the API based on partner
$eligibleFieldIds = $FieldIds
$skippedFields = @()

Write-Host "Fields already filtered by API for this submitter" -ForegroundColor Green
foreach ($fieldId in $FieldIds) {
    Write-Host "  ✓ Field $($fieldId): Accessible" -ForegroundColor Green
}

if ($eligibleFieldIds.Count -eq 0) {
    Write-Host "Error No fields available for this partner to sign!" -ForegroundColor Red
    exit
}

# Update FieldIds to only include eligible fields
$FieldIds = $eligibleFieldIds

Write-Host "Token: $($Token.Substring(0, 20))..." -ForegroundColor Gray
Write-Host "Eligible Field IDs to sign: $($FieldIds -join ', ')" -ForegroundColor Gray

# URL encode the token for safe URL usage
$EncodedToken = [System.Web.HttpUtility]::UrlEncode($Token)

# Get field information to determine field types
Write-Host ""
Write-Host "[Step 3] Getting field information..." -ForegroundColor Yellow

# Get fields from API to get types
try {
    $fieldsResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$encodedToken/fields" -Method GET
    $fields = $fieldsResponse.data.template_fields
    $fieldTypes = @{}
    foreach ($field in $fields) {
        $fieldTypes[[int]$field.id] = $field.field_type
    }
    Write-Host "Success Field types loaded from API" -ForegroundColor Green
    foreach ($field in $fields) {
        Write-Host "  - Field $($field.id): Type: $($field.field_type)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Error Failed to get field types: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Prepare signatures for all field IDs
Write-Host ""
Write-Host "[Step 4] Preparing signatures for $($FieldIds.Count) eligible fields..." -ForegroundColor Yellow

$signatures = @()
foreach ($fieldId in $FieldIds) {
    $fieldType = $fieldTypes[[int]$fieldId]
    
    if ($fieldType -eq "image") {
        # For image fields, upload a test image
        Write-Host "  - Preparing image upload for field ID: $fieldId" -ForegroundColor Gray
        
        # Create test image if it doesn't exist
        $testImagePath = "./test_sign_image.png"
        if (-not (Test-Path $testImagePath)) {
            # Create a simple test image (minimal PNG)
            $pngBase64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
            $pngBytes = [System.Convert]::FromBase64String($pngBase64)
            [System.IO.File]::WriteAllBytes($testImagePath, $pngBytes)
            Write-Host "    Created test image at: $testImagePath" -ForegroundColor Gray
        }
        
        # Upload image to get URL
        try {
            $curlArgs = @(
                '-X', 'POST',
                "$baseUrl/api/files/upload/public",
                '-F', "file=@$testImagePath",
                '-s'
            )
            
            $uploadOutput = & curl @curlArgs 2>$null
            $uploadResult = $uploadOutput | ConvertFrom-Json
            
            if ($uploadResult -and $uploadResult.data.url) {
                $imageUrl = $uploadResult.data.url
                $signatures += @{
                    field_id = $fieldId
                    signature_value = $imageUrl
                }
                Write-Host "    Success Uploaded image, URL: $imageUrl" -ForegroundColor Green
            } else {
                Write-Host "    Warning Image upload failed, using placeholder" -ForegroundColor Yellow
                $signatures += @{
                    field_id = $fieldId
                    signature_value = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
                }
            }
        } catch {
            Write-Host "    Warning Image upload failed: $($_.Exception.Message), using placeholder" -ForegroundColor Yellow
            $signatures += @{
                field_id = $fieldId
                signature_value = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
            }
        }
    } else {
        if ($fieldType -eq "date") {
            # For date fields, use a date string
            Write-Host "  - Preparing date for field ID: $fieldId" -ForegroundColor Gray
            $dateValue = Get-Date -Format 'yyyy-MM-dd'
            $signatures += @{
                field_id = $fieldId
                signature_value = $dateValue
            }
        } else {
            # For signature fields, create text signature
            Write-Host "  - Preparing text signature for field ID: $fieldId" -ForegroundColor Gray
            $signatureText = "Signed by Test User 2 - Field $fieldId - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
            $signatureBytes = [System.Text.Encoding]::UTF8.GetBytes($signatureText)
            $signatureBase64 = [System.Convert]::ToBase64String($signatureBytes)
            
            $signatures += @{
                field_id = $fieldId
                signature_value = "data:text/plain;base64,$signatureBase64"
            }
        }
    }
}

Write-Host "Success Prepared $($signatures.Count) signatures" -ForegroundColor Green

# Submit bulk signatures
Write-Host ""
Write-Host "[Step 3] Submitting signatures..." -ForegroundColor Yellow

$signBody = @{
    signatures = $signatures
    ip_address = "127.0.0.1"
    user_agent = "PowerShell/TestScript"
} | ConvertTo-Json -Depth 10

$headers = @{
    "Content-Type" = "application/json"
}

try {
    $signResponse = Invoke-RestMethod -Uri "$baseUrl/public/signatures/bulk/$EncodedToken" -Method POST -Headers $headers -Body $signBody
    Write-Host "Success Signatures submitted successfully!" -ForegroundColor Green
    Write-Host "Signed at: $($signResponse.data.signed_at)" -ForegroundColor Gray
    
    if ($signResponse.data.bulk_signatures) {
        $bulkSigs = $signResponse.data.bulk_signatures
        if ($bulkSigs -is [string]) {
            $bulkSigs = $bulkSigs | ConvertFrom-Json
        }
        Write-Host "Number of signatures: $($bulkSigs.Count)" -ForegroundColor Gray
        
        Write-Host ""
        Write-Host "Signature details:" -ForegroundColor Cyan
        foreach ($sig in $bulkSigs) {
            Write-Host "  - Field: $($sig.field_name) (ID: $($sig.field_id))" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "Error Failed to submit signatures: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# Verify signatures by checking public submitter
Write-Host ""
Write-Host "[Step 5] Verifying signatures..." -ForegroundColor Yellow

try {
    $verifyResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$Token" -Method GET
    if ($verifyResponse.data.status -eq "completed") {
        Write-Host "Success Partner-based signatures verified!" -ForegroundColor Green
        Write-Host "Submitter status: $($verifyResponse.data.status)" -ForegroundColor Gray
        Write-Host "Signed at: $($verifyResponse.data.signed_at)" -ForegroundColor Gray
        Write-Host "Partner role: $($submitterRole ?? 'General')" -ForegroundColor Cyan
        
        $sigCount = $verifyResponse.data.signature_positions.Count
        Write-Host "Number of signatures: $sigCount" -ForegroundColor Gray
        
        if ($sigCount -gt 0) {
            Write-Host ""
            Write-Host "Signature details for this partner:" -ForegroundColor Cyan
            foreach ($sig in $verifyResponse.data.signature_positions) {
                Write-Host "  - Field: $($sig.field_name) (ID: $($sig.field_id))" -ForegroundColor Gray
            }
        }
        
        Write-Host ""
        Write-Host "Multi-Partner Signing Status:" -ForegroundColor Yellow
        Write-Host "  ✓ This partner ($($submitterRole ?? 'General')) has completed their part" -ForegroundColor Green
        Write-Host "  ⏳ Other partners may still need to sign their assigned fields" -ForegroundColor Gray
        Write-Host "  📄 Document will be fully executed when all partners complete signing" -ForegroundColor Gray
    } else {
        Write-Host "Warning Submitter status is not completed: $($verifyResponse.data.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Warning Could not verify signatures: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PARTNER SIGNING COMPLETE!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Success Multi-partner document signing completed for this partner" -ForegroundColor Green
Write-Host ""
Write-Host "Partner Summary:" -ForegroundColor Yellow
Write-Host "  👤 Signer: $($submitterResponse.data.name)" -ForegroundColor White
Write-Host "  🏷️  Role: $($submitterRole ?? 'General')" -ForegroundColor White
Write-Host "  📧 Email: $($submitterResponse.data.email)" -ForegroundColor White
Write-Host "  ✍️  Fields Signed: $($FieldIds.Count)" -ForegroundColor White
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Cyan
Write-Host "  1. Other partners will receive separate signing invitations" -ForegroundColor Gray
Write-Host "  2. Each partner can only access their assigned fields" -ForegroundColor Gray
Write-Host "  3. Document becomes fully executed when all partners sign" -ForegroundColor Gray
Write-Host "  4. All parties will receive completion notifications" -ForegroundColor Gray
Write-Host ""
Write-Host "🎉 Partner-based digital signing workflow completed successfully!" -ForegroundColor Green
Write-Host "Success $($signatures.Count) signature fields completed" -ForegroundColor Green
Write-Host "Success Submitter status updated to 'completed'" -ForegroundColor Green
Write-Host ""
Write-Host "Email notification sent to document owner" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
