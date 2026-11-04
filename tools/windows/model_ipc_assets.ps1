# MODEL-IPC-01 / phase A: read-only, bounded local asset inventory.
# No executable is started. No network calls, environment exports, or file writes.
# Invoke: powershell -NoProfile -File .\model_ipc_assets.ps1 -Root "C:\PULQVA-NATIVE"
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Root,
    [int]$MaxFiles = 40
)
$ErrorActionPreference = 'Stop'
if ($MaxFiles -lt 1 -or $MaxFiles -gt 200) { throw "MaxFiles must be 1..200" }
$resolved = (Resolve-Path -LiteralPath $Root -ErrorAction Stop).ProviderPath
if (-not (Test-Path -LiteralPath $resolved -PathType Container)) { throw "Root must be a directory" }
$patterns = @('llama-cli.exe', 'llama-server.exe', '*.gguf')
$found = @(
    Get-ChildItem -LiteralPath $resolved -Recurse -File -ErrorAction Stop |
    Where-Object {
        $name = $_.Name
        $patterns | Where-Object { $name -like $_ }
    } |
    Sort-Object FullName -Unique
)
Write-Host "MODEL_IPC_ASSET_INVENTORY_V1"
Write-Host "FOUND:" $found.Count
if ($found.Count -gt $MaxFiles) { throw "Too many candidates: $($found.Count); narrow Root (no hashing performed)" }
foreach ($file in $found) {
    if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Reparse point candidate rejected"
    }
    $sha = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256 -ErrorAction Stop).Hash.ToLowerInvariant()
    [pscustomobject]@{
        Kind = if ($file.Extension -ieq '.gguf') { 'MODEL' } elseif ($file.Name -ieq 'llama-cli.exe') { 'CLI' } else { 'SERVER' }
        Path = $file.FullName
        Bytes = $file.Length
        SHA256 = $sha
    } | Format-List
}
if (-not $found) { Write-Host "RESULT: NO_CANDIDATES_IN_ROOT" }
