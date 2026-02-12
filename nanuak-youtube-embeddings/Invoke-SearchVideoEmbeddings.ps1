$releaseExe = "target/release/search_video_embeddings.exe"
if (Test-Path $releaseExe) {
    & $releaseExe
} else {
    cargo run --release --bin search_video_embeddings
}
# $debugExe = "target/debug/search_video_embeddings.exe"

# elseif (Test-Path $debugExe) {
#     & $debugExe
# }
# else {
#     cargo run --bin search_video_embeddings
# }