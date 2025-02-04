if (-not (Test-Path -Path Env:\HF_TOKEN)) {
    Write-Host "[Get-HFToken] Make sure to dot-source this file!"
    $token = op item get "Huggingface CLI (read)" --vault "Private" --field "credential"
    $env:HF_TOKEN = $token
}
