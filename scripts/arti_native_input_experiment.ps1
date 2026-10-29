param([ValidateSet('restored','runner')][string]$NativeInputs)
$ErrorActionPreference='Stop'
$selected=Get-Content -LiteralPath "$env:RUNNER_TEMP/arti-selected-compiler.json" -Raw | ConvertFrom-Json
$pin=Get-Content -LiteralPath 'sidecars/arti/WINDOWS_COMPILER.json' -Raw | ConvertFrom-Json
$restored=Split-Path ($selected.VCToolsInstallDir.TrimEnd('\')) -Parent
$restored=Join-Path $restored $pin.toolset
$runner=Join-Path $env:ProgramFiles "Microsoft Visual Studio\18\Enterprise\VC\Tools\MSVC\$($pin.toolset)"
$roots=[ordered]@{restored=$restored;runner=$runner}
$manifests=[ordered]@{}
foreach($arm in $roots.Keys){
 $root=$roots[$arm]
 if(-not(Test-Path -LiteralPath $root -PathType Container)){throw "Missing experimental input tree: $arm"}
 $files=@()
 foreach($sub in @('include','lib\x64')){
  $dir=Join-Path $root $sub
  foreach($f in Get-ChildItem -LiteralPath $dir -Recurse -File){
   if($f.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Symlink input rejected'}
   $files += [ordered]@{path=[IO.Path]::GetRelativePath($root,$f.FullName).Replace('\','/');size=$f.Length;sha256=(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
  }
 }
 $manifests[$arm]=@($files | Sort-Object { $_.path })
}
$root=$roots[$NativeInputs]
$include=@($selected.INCLUDE -split ';');$libs=@($selected.LIB -split ';')
$include[0]=Join-Path $root 'include';$libs[0]=Join-Path $root 'lib\x64'
# Compiler/archiver/DLL selection, Rust, SDK and flags stay unchanged.
"INCLUDE=$($include -join ';')" | Out-File -LiteralPath $env:GITHUB_ENV -Encoding utf8 -Append
"LIB=$($libs -join ';')" | Out-File -LiteralPath $env:GITHUB_ENV -Encoding utf8 -Append
[ordered]@{schema=1;hypothesis='MSVC include/CRT library root affects exact Arti bytes';arm=$NativeInputs;source_sha=$env:GITHUB_SHA;image=$env:ImageVersion;compiler=$selected;effective_include=$include;effective_lib=$libs;trees=$manifests;limitation='Root path and contents are a single factor; SDK content is common to arms, not proven historical-equal'} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath "$env:RUNNER_TEMP/arti-native-input-experiment.json" -Encoding utf8
Write-Output "PULQVA_ARTI_NATIVE_INPUT_ARM=$NativeInputs"
