param(
    [Parameter(Mandatory = $true)]
    [string]$StructuredFile
)

# Read the entire structured file content
$rawContent = Get-Content -Path $StructuredFile -Raw

# Split on '=== file: '
# The first split element may be empty if the text starts immediately with '=== file', so we ignore it if empty
$sections = $rawContent -split "=== file: "

foreach ($section in $sections) {
    $section = $section.Trim()
    if ($section) {
        # The file name is on the first line
        $lines = $section -split "`r?`n"
        $fileName = $lines[0].Trim()

        # Everything after that line is file content
        $fileContent = $lines[1..($lines.Count - 1)] -join "`r`n"

        # In case we have an === end or another marker at the end, let's remove anything after '==='
        if ($fileContent -match "(?s)(.*?)===.*") {
            $fileContent = $Matches[1]
        }

        # Ensure the directory exists
        $dir = Split-Path $fileName
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Force -Path $dir | Out-Null
        }

        # Write the file
        Set-Content -Path $fileName -Value $fileContent
        Write-Host "Wrote $fileName"
    }
}