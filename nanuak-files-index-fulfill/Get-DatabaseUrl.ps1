if (-not (Test-Path -Path Env:\DATABASE_URL)) {
    Write-Host "[Get-DatabaseUrl] Make sure to dot-source this file!"
    $password = op read "op://Private/PostgreSQL Local/password" --no-newline
    $env:DATABASE_URL = "postgres://postgres:$password@localhost/nanuak"
}
