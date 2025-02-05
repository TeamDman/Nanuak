$a = 1..211 | % { "a" } | Join-String
$b = "b"
$c = 1..234 | % { "c" } | Join-String
$msg = "$a$b$c, what index is the needle?"

$resp = Invoke-WebRequest `
-Uri "http://localhost:5000/generate" `
-Method "POST" `
-ContentType "application/json" `
-Body @"
{"messages": [{"role": "user", "content": "$msg"}]}
"@
$resp.Content | ConvertFrom-Json | Select-Object -ExpandProperty "response" | Write-Host