$name = Read-Host "Enter the name of the migration"
diesel migration generate $name
$migration = Get-ChildItem .\migrations `
| Where-Object { $_.Name -like "*$name*"} `
| Select-Object -First 1 -ExpandProperty Name
code ".\migrations\$migration\up.sql"
code ".\migrations\$migration\down.sql"