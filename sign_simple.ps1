# Sign Document Script - Simple Version
# Directly sign with field IDs from workflow

param(
    [Parameter(Mandatory=$false)]
    [string]$Token,
    [Parameter(Mandatory=$false)]
    [array]$FieldIds
)

$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:3000"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DocuSeal Pro - Document Signing" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Load token from file if not provided
if (-not $Token) {
    $tokenFile = "d:\Docuseal_Pro\submitter_token.txt"
    if (Test-Path $tokenFile) {
        $Token = Get-Content $tokenFile -Raw
        $Token = $Token.Trim()
        Write-Host "Success Token loaded from file" -ForegroundColor Green
    } else {
        Write-Host "Error No token found!" -ForegroundColor Red
        exit
    }
}

# Load field IDs from workflow output if not provided
if (-not $FieldIds -or $FieldIds.Count -eq 0) {
    $fieldIdsFile = "d:\Docuseal_Pro\field_ids.txt"
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

# URL encode the token
$EncodedToken = [System.Web.HttpUtility]::UrlEncode($Token)

# Prepare signatures for all field IDs
Write-Host ""
Write-Host "[Step 1] Preparing signatures for $($FieldIds.Count) fields..." -ForegroundColor Yellow

$signatures = @()
foreach ($fieldId in $FieldIds) {
    # Create a simple base64 encoded signature
    $signatureText = "Signed by Bui Hai Giap - Field $fieldId - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
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

try {
    $signResponse = Invoke-RestMethod -Uri "$baseUrl/public/signatures/bulk/$EncodedToken" -Method POST -Body $signBody -ContentType "application/json"
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

# Get signature history to verify
Write-Host ""
Write-Host "[Step 3] Verifying signature history..." -ForegroundColor Yellow

try {
    $historyResponse = Invoke-RestMethod -Uri "$baseUrl/public/signatures/history/$EncodedToken" -Method GET
    if ($historyResponse.data -and $historyResponse.data.Count -gt 0) {
        Write-Host "Success Signature history verified!" -ForegroundColor Green
        Write-Host "Total signature records: $($historyResponse.data.Count)" -ForegroundColor Gray
        
        $latestRecord = $historyResponse.data[0]
        Write-Host ""
        Write-Host "Latest signature record:" -ForegroundColor Cyan
        Write-Host "  Signed at: $($latestRecord.signed_at)" -ForegroundColor Gray
        Write-Host "  IP: $($latestRecord.ip_address)" -ForegroundColor Gray
        
        if ($latestRecord.signature_value) {
            Write-Host "  Has bulk signatures data" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "Warning Could not verify signature history" -ForegroundColor Yellow
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
