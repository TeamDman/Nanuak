Write-Host "[Get-DatabaseUrl] Make sure to dot-source this file!"
$password = op item get "PostgreSQL Local" --vault "Private" --field password
$env:DATABASE_URL = "postgres://postgres:$password@localhost/nanuak"
