if (-not (Test-Path -Path Env:\YOUTUBE_API_KEY)) {
    Write-Host "[Get-YouTubeApiKey] Make sure to dot-source this file!"
    $credential = op read "op://Private/YouTube Data API v3 - Nanuak/credential" --no-newline
    $env:YOUTUBE_API_KEY = $credential
}
