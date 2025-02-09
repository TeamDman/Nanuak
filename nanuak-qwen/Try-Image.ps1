$resp = Invoke-WebRequest `
-Uri "http://localhost:5000/generate" `
-Method "POST" `
-ContentType "application/json" `
-Body @"
{
    "messages": [
        {"role": "user", "content": [
            {"type":"image","image":"file:///C:\\Users\\TeamD\\OneDrive\\Pictures\\Screenshots\\ShareX Screenshots\\2025-01\\bg3_dx11_uEWybFs9GC.png"},
            {"type":"text","text":"Describe this image. Include ALL text from the image in your description."}
        ]}
    ]
}
"@
if ($resp.StatusCode -eq 200) {
    $resp.Content | ConvertFrom-Json | Select-Object -ExpandProperty "response" | Write-Host
}