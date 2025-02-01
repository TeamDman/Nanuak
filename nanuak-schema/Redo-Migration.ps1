. .\Get-DatabaseUrl.ps1
diesel migration list
# ask are you sure
$resp = Read-Host "Are you sure you want to undo the last migration? (y/n)"
if ($resp -eq "y") {
    diesel migration redo
}