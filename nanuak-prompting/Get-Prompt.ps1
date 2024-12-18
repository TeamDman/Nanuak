$files = Get-Content .\files.txt
$prompt = "# File summary`n`n"

foreach ($file in $files) {
    $file_ext = $file -split "\." | Select-Object -Last 1
    $file_content = Get-Content $file -Raw
    $prompt += "## $file`n````````$file_ext`n$file_content`n`````````n`n"
}

Write-Host "Writing file"
$prompt | Out-File .\prompt.md

Write-Host "Copying to clipboard"
$prompt | Set-Clipboard

Write-Host "Opening file"
code .\prompt.md