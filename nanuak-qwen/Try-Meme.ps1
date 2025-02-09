$resp = Invoke-WebRequest `
-Uri "http://localhost:5000/generate" `
-Method "POST" `
-ContentType "application/json" `
-Body @"
{
    "messages": [
        {"role": "user", "content": [
            {"type":"image","image":"C:\\Users\\TeamD\\OneDrive\\Memes\\Bulk 6\\7n1e910ozuga1.jpg"},
            {"type":"text","text":"I'm visually impaired, please describe this image from social media. Please also provide an interpretation of the image and any humor or other relevant context."}
        ]}
    ]
}
"@
if ($resp.StatusCode -eq 200) {
    $resp.Content | ConvertFrom-Json | Select-Object -ExpandProperty "response" | Write-Host
}