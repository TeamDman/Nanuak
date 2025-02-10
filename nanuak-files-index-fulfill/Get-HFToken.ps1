if (-not (Test-Path -Path Env:\HF_TOKEN)) {
    Write-Host "[Get-HFToken] Make sure to dot-source this file!"
    $token = op read "op://Private/o24pfzdtppu4asfopqhzya5rg4/credential" --no-newline
    $env:HF_TOKEN = $token
}
