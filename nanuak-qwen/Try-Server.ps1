$resp = Invoke-WebRequest `
-Uri "http://localhost:5000/generate" `
-Method "POST" `
-ContentType "application/json" `
-Body '{"messages": [{"role": "user", "content": "Tell me a joke."}]}'
$resp.Content | ConvertFrom-Json | Select-Object -ExpandProperty "response" | Write-Host