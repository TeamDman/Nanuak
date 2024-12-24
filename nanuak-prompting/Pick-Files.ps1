$parent = pwd | Select-Object -ExpandProperty Path | Split-Path -Parent
$files = Get-ChildItem .. -recurse `
| ForEach-Object { $_.FullName} `
| fzf --multi --height '~100%'
$files `
| ForEach-Object { }