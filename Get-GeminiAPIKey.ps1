if (-not (Test-Path -Path Env:\GEMINI_API_KEY)) {
    Write-Host "[Get-GeminiAPIKey] Make sure to dot-source this file!"
    $api_key = op read "op://Private/Nanuak Gemini Aider API key/credential" --no-newline
    $env:GEMINI_API_KEY=$api_key
}
