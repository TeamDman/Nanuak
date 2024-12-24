if (-not (Test-Path -Path Env:\GEMINI_API_KEY)) {
    Write-Host "[Get-GeminiAPIKey] Make sure to dot-source this file!"
    $api_key = op item get "Nanuak Gemini Aider API key" --vault "Private" --field credential
    $env:GEMINI_API_KEY=$api_key
}
