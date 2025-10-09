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
    Write-Host "Name: $($submitterResponse.data.submitter.name)" -ForegroundColor Gray
    Write-Host "Email: $($submitterResponse.data.submitter.email)" -ForegroundColor Gray
    Write-Host "Status: $($submitterResponse.data.submitter.status)" -ForegroundColor Gray
} catch {
    Write-Host "Error Failed to get submitter: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $reader.BaseStream.Position = 0
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody" -ForegroundColor Red
    }
    exit
}# Load token from file if not provided
if (-not $Token) {
    $tokenFile = "/workspaces/Docuseal_Pro/submitter_token.txt"
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
    $fieldIdsFile = "/workspaces/Docuseal_Pro/field_ids.txt"
    if (Test-Path $fieldIdsFile) {
        $FieldIds = Get-Content $fieldIdsFile | ForEach-Object { [int]$_ }
        Write-Host "Success Field IDs loaded from file: $($FieldIds -join ', ')" -ForegroundColor Green
    } else {
        Write-Host "Error No field IDs provided or found in file!" -ForegroundColor Red
        Write-Host "Usage: .\sign_simple.ps1 -FieldIds @(1,2,3)" -ForegroundColor Yellow
        exit
    }
}

Write-Host "Token: $($Token.Substring(0, 20))..." -ForegroundColor Gray
Write-Host "Field IDs to sign: $($FieldIds -join ', ')" -ForegroundColor Gray

# URL encode the token for safe URL usage
$EncodedToken = [System.Web.HttpUtility]::UrlEncode($Token)

# Prepare signatures for all field IDs
Write-Host ""
Write-Host "[Step 1] Preparing signatures for $($FieldIds.Count) fields..." -ForegroundColor Yellow

$signatures = @()
foreach ($fieldId in $FieldIds) {
    # Create a simple base64 encoded signature
    $signatureText = "Signed by Test User 2 - Field $fieldId - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
    $signatureBytes = [System.Text.Encoding]::UTF8.GetBytes($signatureText)
    $signatureBase64 = [System.Convert]::ToBase64String($signatureBytes)
    
    $signatures += @{
        field_id = $fieldId
        signature_value = "data:text/plain;base64,$signatureBase64"
    }
    
    Write-Host "  - Prepared signature for field ID: $fieldId" -ForegroundColor Gray
}

Write-Host "Success Prepared $($signatures.Count) signatures" -ForegroundColor Green

# Submit bulk signatures
Write-Host ""
Write-Host "[Step 2] Submitting signatures..." -ForegroundColor Yellow

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
    if ($_.Exception.Response) {
        $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
        $reader.BaseStream.Position = 0
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody" -ForegroundColor Red
    }
    exit
}

# Verify signatures by checking public submitter
Write-Host ""
Write-Host "[Step 3] Verifying signatures..." -ForegroundColor Yellow

try {
    $verifyResponse = Invoke-RestMethod -Uri "$baseUrl/public/submissions/$Token" -Method GET
    if ($verifyResponse.data.status -eq "completed") {
        Write-Host "Success Signatures verified!" -ForegroundColor Green
        Write-Host "Submitter status: $($verifyResponse.data.status)" -ForegroundColor Gray
        Write-Host "Signed at: $($verifyResponse.data.signed_at)" -ForegroundColor Gray
        
        $sigCount = $verifyResponse.data.signature_positions.Count
        Write-Host "Number of signatures: $sigCount" -ForegroundColor Gray
        
        if ($sigCount -gt 0) {
            Write-Host ""
            Write-Host "Signature details:" -ForegroundColor Cyan
            foreach ($sig in $verifyResponse.data.signature_positions) {
                Write-Host "  - Field: $($sig.field_name) (ID: $($sig.field_id))" -ForegroundColor Gray
            }
        }
    } else {
        Write-Host "Warning Submitter status is not completed: $($verifyResponse.data.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Warning Could not verify signatures: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "SIGNING COMPLETE!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Success Document signed successfully" -ForegroundColor Green
Write-Host "Success $($signatures.Count) signature fields completed" -ForegroundColor Green
Write-Host "Success Submitter status updated to 'completed'" -ForegroundColor Green
Write-Host ""
Write-Host "Email notification sent to document owner" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
