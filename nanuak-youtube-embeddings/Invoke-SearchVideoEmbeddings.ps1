$releaseExe = "target/release/search_video_embeddings.exe"
$debugExe = "target/debug/search_video_embeddings.exe"

if (Test-Path $releaseExe) {
    & $releaseExe
}
elseif (Test-Path $debugExe) {
    & $debugExe
}
else {
    cargo run --bin search_video_embeddings
}