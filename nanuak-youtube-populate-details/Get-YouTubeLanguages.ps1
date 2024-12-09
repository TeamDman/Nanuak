. .\Get-YouTubeApiKey.ps1
$resp = Invoke-WebRequest "https://www.googleapis.com/youtube/v3/i18nLanguages?key=$($Env:YOUTUBE_API_KEY)"
if ($?) {
    $resp.Content
}