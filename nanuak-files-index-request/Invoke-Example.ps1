. .\Get-DatabaseUrl.ps1
$images_dir = "..\image-embedding-experiments\images\"
$chosen = Get-ChildItem -File $images_dir `
| ForEach-Object { $_.FullName } `
| fzf --header "Pick an image to index" --multi
foreach ($x in $chosen) {
    Write-Output $x
    cargo run -- "$x"
}