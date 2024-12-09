. .\Get-YouTubeApiKey.ps1
$resp = Invoke-WebRequest "https://www.googleapis.com/youtube/v3/videoCategories?regionCode=CA&key=$($Env:YOUTUBE_API_KEY)"
if ($?) {
    $resp.Content
}