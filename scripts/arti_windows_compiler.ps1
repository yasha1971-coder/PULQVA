# Build-only selection. This is not a hermetic SDK or a product runtime change.
$ErrorActionPreference = 'Stop'
$pin = Get-Content -LiteralPath 'sidecars/arti/WINDOWS_COMPILER.json' -Raw | ConvertFrom-Json
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$roots = @(& $vswhere -all -products '*' -property installationPath)
if ($LASTEXITCODE -ne 0) { throw 'VS discovery failed' }
$matches = @($roots | ForEach-Object {
    $candidate = Join-Path $_ "VC\Tools\MSVC\$($pin.toolset)"
    if (Test-Path -LiteralPath $candidate) { $candidate }
})
if ($matches.Count -ne 1) { throw 'Pinned MSVC toolset missing or ambiguous; no fallback' }
$root = $matches[0]
$bin = Join-Path $root 'bin\Hostx64\x64'
foreach ($entry in $pin.files.PSObject.Properties) {
    $path = Join-Path $bin $entry.Name
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $entry.Value) { throw "Pinned compiler input mismatch: $($entry.Name)" }
}
$sdk = "${env:ProgramFiles(x86)}\Windows Kits\10"
$include = @((Join-Path $root 'include'))
foreach ($part in @('ucrt','shared','um','winrt')) {
    $include += Join-Path $sdk "Include\$($pin.sdk)\$part"
}
$lib = @((Join-Path $root 'lib\x64'), (Join-Path $sdk "Lib\$($pin.sdk)\ucrt\x64"), (Join-Path $sdk "Lib\$($pin.sdk)\um\x64"))
foreach ($path in ($include + $lib)) {
    if (-not (Test-Path -LiteralPath $path -PathType Container)) { throw "Pinned input directory missing: $path" }
}
$compiler = Join-Path $bin 'cl.exe'
$archiver = Join-Path $bin 'lib.exe'
$vars = [ordered]@{
    CC_x86_64_pc_windows_msvc = $compiler; CXX_x86_64_pc_windows_msvc = $compiler;
    AR_x86_64_pc_windows_msvc = $archiver;
    INCLUDE = ($include -join ';'); LIB = ($lib -join ';');
    VCToolsVersion = $pin.toolset; VCToolsInstallDir = "$root\";
    WindowsSdkDir = "$sdk\"; WindowsSDKVersion = "$($pin.sdk)\";
    CC_ENABLE_DEBUG_OUTPUT = '1'
}
foreach ($entry in $vars.GetEnumerator()) {
    "$($entry.Key)=$($entry.Value)" | Out-File -LiteralPath $env:GITHUB_ENV -Encoding utf8 -Append
}
$bin | Out-File -LiteralPath $env:GITHUB_PATH -Encoding utf8 -Append
$vars | ConvertTo-Json | Set-Content -LiteralPath "$env:RUNNER_TEMP\arti-selected-compiler.json" -Encoding utf8
Write-Output "PULQVA_ARTI_COMPILER_VERIFIED toolset=$($pin.toolset) files=$($pin.files.PSObject.Properties.Count) compiler=$compiler"
