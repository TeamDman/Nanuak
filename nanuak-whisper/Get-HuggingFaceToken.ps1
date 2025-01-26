if (-not (Test-Path -Path Env:\HF_TOKEN)) {
    Write-Host "[Get-HuggingFaceToken] Make sure to dot-source this file!"
    $password = op item get "Huggingface CLI (read)" --vault "Private" --field "credential"
    $env:HF_TOKEN = $password
}