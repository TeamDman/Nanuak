. .\Get-HuggingFaceToken.ps1
$file = "$(Get-Content target_file.txt)"
$file = $file.Replace("`"", "")
uv run whisperx `
--hf_token "$Env:HF_TOKEN" `
--highlight_words True `
--task "transcribe" `
--language "en" `
--output_dir "output" `
--device "cuda" `
--model "large-v2" `
--diarize `
--min_speakers 2 `
--max_speakers 2 `
$file