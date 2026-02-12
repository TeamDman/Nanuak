$ErrorActionPreference = "Stop"

# Ensure Postgres client libraries are discoverable for linking.
$postgresLib = "C:\Program Files\PostgreSQL\17\lib"
$postgresBin = "C:\Program Files\PostgreSQL\17\bin"

if (-not (Test-Path $postgresLib)) {
    throw "PostgreSQL lib path not found: $postgresLib"
}
if (-not (Test-Path $postgresBin)) {
    throw "PostgreSQL bin path not found: $postgresBin"
}

$env:LIB = "$postgresLib;" + $env:LIB
$env:PATH = "$postgresBin;" + $env:PATH

# cargo clean
cargo build
