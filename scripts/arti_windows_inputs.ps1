# CI diagnostics: inventories available native inputs, not proof of linker selection.
$ErrorActionPreference = 'Stop'
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$installations = & $vswhere -all -products '*' -format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or -not $installations) { throw 'No VS inventory' }
$files = @()
foreach ($installation in $installations) {
    $root = Join-Path $installation.installationPath 'VC\Tools\MSVC'
    foreach ($version in Get-ChildItem -LiteralPath $root -Directory) {
        foreach ($pattern in @('bin\Hostx64\x64\*.exe', 'bin\Hostx64\x64\*.dll', 'lib\x64\*.lib')) {
            $files += Get-ChildItem -Path (Join-Path $version.FullName $pattern) -File
        }
    }
}
$sdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\Lib"
foreach ($version in Get-ChildItem -LiteralPath $sdkRoot -Directory) {
    foreach ($component in @('ucrt', 'um')) {
        $files += Get-ChildItem -Path (Join-Path $version.FullName "$component\x64\*.lib") -File
    }
}
$records = @($files | Sort-Object FullName -Unique | ForEach-Object {
    [ordered]@{ path = $_.FullName; size = $_.Length; version = $_.VersionInfo.FileVersion;
        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
})
if ($records.Count -eq 0) { throw 'Empty native input inventory' }
[ordered]@{
    schema = 1; scope = 'available inputs; linker map required to identify selection';
    image = $env:ImageOS; image_version = $env:ImageVersion;
    installations = $installations; inputs = $records
} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath "$env:RUNNER_TEMP\arti-native-inputs.json" -Encoding utf8
Write-Output "PULQVA_ARTI_NATIVE_INPUTS count=$($records.Count)"
