if (-not (Test-Path -Path Env:\OLLAMA_API_BASE)) {
    Write-Host "[Get-OllamaEnvVar] Make sure to dot-source this file!"
    $env:OLLAMA_API_BASE="http://127.0.0.1:11434"
}

