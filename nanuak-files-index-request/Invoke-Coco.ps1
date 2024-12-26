. .\Get-DatabaseUrl.ps1
$images_dir = "G:\ml\coco\coco test2017\test2017"
$chosen = Get-ChildItem -File $images_dir `
| ForEach-Object { $_.FullName } `
| fzf --header "Pick images to index" --multi `
      --bind "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all"
$chosen | Set-Content files.txt
Write-Host "Processing $(Get-Content files.txt | Measure-Object -Line) files."
cargo run -- --file-path-txt files.txt