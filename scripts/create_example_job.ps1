# PowerShell script to create an example job via API
# Usage: .\create_example_job.ps1 -JobFile "example_jobs\weather_api_job.json"

param(
    [Parameter(Mandatory=$false)]
    [string]$JobFile = "example_jobs\weather_api_job.json",
    
    [Parameter(Mandatory=$false)]
    [string]$ApiUrl = "http://localhost:8080"
)

# Check if job file exists
if (-not (Test-Path $JobFile)) {
    Write-Error "Job file not found: $JobFile"
    exit 1
}

# Read job configuration
$jobContent = Get-Content $JobFile -Raw | ConvertFrom-Json

# Create the job via API
Write-Host "Creating job: $($jobContent.job_name)" -ForegroundColor Cyan

try {
    $response = Invoke-RestMethod -Uri "$ApiUrl/jobs" `
        -Method Post `
        -ContentType "application/json" `
        -Body ($jobContent | ConvertTo-Json -Depth 10)
    
    Write-Host "✓ Job created successfully!" -ForegroundColor Green
    Write-Host "Job ID: $($response.job_id)" -ForegroundColor Yellow
    Write-Host "Job Name: $($response.job_name)" -ForegroundColor Yellow
    
    # Ask if user wants to trigger the job immediately
    $trigger = Read-Host "Do you want to trigger this job now? (y/n)"
    if ($trigger -eq "y") {
        Write-Host "Triggering job..." -ForegroundColor Cyan
        $runResponse = Invoke-RestMethod -Uri "$ApiUrl/jobs/$($response.job_id)/run" -Method Post
        Write-Host "✓ Job triggered successfully!" -ForegroundColor Green
        Write-Host "Check status at: $ApiUrl/runs" -ForegroundColor Yellow
    }
    
} catch {
    Write-Error "Failed to create job: $_"
    exit 1
}
