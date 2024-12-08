if (-not (Test-Path -Path Env:\YOUTUBE_API_KEY)) {
    Write-Host "[Get-YouTubeApiKey] Make sure to dot-source this file!"
    $credential = op item get "YouTube Data API v3 - Nanuak" --vault "Private" --field "credential"
    $env:YOUTUBE_API_KEY = $credential
}
