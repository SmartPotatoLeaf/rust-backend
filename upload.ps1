# Parameters
param(
    [string]$Mode = ""
)

# Configuration
$containerAppName = "spl-rest-api"
$resourceGroup = "spl"
$envFile = ".env"
$templateFile = "containerapp.yaml"
$outputFile = "containerapp.generated.yaml"

# Check if skip mode is enabled
$skipConfirmations = $Mode -eq "skip"

# Display configuration
Write-Host "======================================"
Write-Host "Azure Container App Deployment"
Write-Host "======================================"
Write-Host "Container App Name: ${containerAppName}"
Write-Host "Resource Group: ${resourceGroup}"
Write-Host "Environment File: ${envFile}"
Write-Host "Template File: ${templateFile}"
Write-Host "Generated File: ${outputFile}"
if ($skipConfirmations) {
    Write-Host "Mode: SKIP (No confirmations)"
}
Write-Host "======================================`n"

# Validate required files exist
if (!(Test-Path $envFile)) {
    Write-Host "Error: .env file not found"
    exit 1
}

if (!(Test-Path $templateFile)) {
    Write-Host "Error: containerapp.yaml template not found"
    exit 1
}

# Read and parse .env file
$envObjects = @()

Get-Content $envFile | ForEach-Object {
    $line = $_.Trim()

    # Ignore comments and empty lines
    if ($line -and -not $line.StartsWith("#")) {
        if ($line -match "^[^=]+=") {
            $parts = $line.Split("=", 2)

            $envObjects += @{
                name  = $parts[0]
                value = $parts[1]
            }
        }
        else {
            Write-Host "Warning: Invalid line ignored: $line"
        }
    }
}

if ($envObjects.Count -eq 0) {
    Write-Host "Error: No valid variables found in .env file"
    exit 1
}

# Display variables found
Write-Host ""
Write-Host "Environment variables found in ${envFile}:"
Write-Host "------------------------------------"

foreach ($var in $envObjects) {
    Write-Host ("{0} = {1}" -f $var.name, $var.value)
}

# Confirmation prompt
if (-not $skipConfirmations) {
    Write-Host ""
    Write-Host "Type 'ok' to continue with deployment:"
    $confirmation = Read-Host

    if ($confirmation -ne "ok") {
        Write-Host "Deployment cancelled by user."
        exit 0
    }
}

# Build YAML env block
$envYaml = "env:`n"

foreach ($var in $envObjects) {
    $envYaml += "        - name: $($var.name)`n"
    $envYaml += "          value: `"$($var.value)`"`n"
}

# Read template YAML
$templateContent = Get-Content $templateFile -Raw

# Replace existing env block placeholder (must exist as: env: [])
$templateContent = $templateContent -replace "env:\s*\[\]", $envYaml

# Save generated YAML
$templateContent | Out-File $outputFile -Encoding utf8

Write-Host ""
Write-Host "Generated YAML file created: ${outputFile}"

# Second confirmation prompt
if (-not $skipConfirmations) {
    Write-Host ""
    Write-Host "The YAML file has been generated with the environment variables."
    Write-Host "Type 'ok' to proceed with Azure deployment:"
    $finalConfirmation = Read-Host

    if ($finalConfirmation -ne "ok") {
        Write-Host "Deployment cancelled by user."
        exit 0
    }
}

# Execute Azure Container App update using YAML only
Write-Host ""
Write-Host "Updating Azure Container App using YAML..."

az containerapp update `
    --name $containerAppName `
    --resource-group $resourceGroup `
    --yaml $outputFile

Write-Host ""
Write-Host "Deployment completed successfully."
