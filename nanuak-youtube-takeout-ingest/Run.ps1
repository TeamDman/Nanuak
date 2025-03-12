# Get the path to the documents folder
$documents = [Environment]::GetFolderPath("MyDocuments")
$takeout_backups = Join-Path $documents "Backups\takeout"
$latest_backup_dir = Get-ChildItem $takeout_backups | Sort-Object LastWriteTime -Descending | Select-Object -First 1
Write-Host "Getting the latest backup from $latest_backup_dir"
$history_dir = Join-Path $latest_backup_dir "Takeout\YouTube and YouTube Music\history"
$json_files = Get-ChildItem $history_dir -Filter "*.json"
# assert not empty
if ($json_files.Count -eq 0) {
    Write-Host "No JSON files found in $history_dir"
    exit 1
} else {
    Write-Host "Found $($json_files.Count) JSON files in $history_dir"
}

cargo run -- --ingest-dir $history_dir