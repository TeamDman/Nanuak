. .\Get-DatabaseUrl.ps1
$images_dir = "..\image-embedding-experiments\images\"
$chosen = Get-ChildItem -File $images_dir `
| ForEach-Object { $_.FullName } `
| fzf --header "Pick images to index" --multi `
      --bind "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all"
foreach ($x in $chosen) {
    Write-Output $x
    cargo run -- "$x"
}
