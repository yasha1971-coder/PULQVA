# Build-only recovery candidate. Not wired into CI until reviewed on Windows.
# Source: https://learn.microsoft.com/en-us/visualstudio/releases/2026/release-history
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'Windows required' }
if (-not $env:RUNNER_TEMP) { throw 'Owned runner temp required' }
$url = 'https://download.visualstudio.microsoft.com/download/pr/7437128c-6580-48ab-9c69-f7452be2ee7f/160f5e9c319e3408867cae9de83f5d8803bdf7c34fcc8463e8fc28286e49d99e/vs_BuildTools.exe'
$work = Join-Path $env:RUNNER_TEMP 'pulqva-vs18101-recovery'
if (Test-Path -LiteralPath $work) { throw 'Recovery namespace already exists; no reuse' }
New-Item -ItemType Directory -Path $work | Out-Null
$install = Join-Path $work 'BuildTools'
$exe = Join-Path $work 'vs_BuildTools.exe'
$clock = [Diagnostics.Stopwatch]::StartNew()
$receipt = [ordered]@{schema=1; source_sha=$env:GITHUB_SHA; url=$url; expected_installation_version='18.10.12210.168'; status='ERROR'; installer_sha256=$null; signer=$null; exit_code=$null; duration_ms=$null; error=$null; scope='installer and compiler restoration only; not build or product acceptance'}
try {
    Invoke-WebRequest -Uri $url -OutFile $exe -TimeoutSec 120 -MaximumRedirection 0
    $receipt.installer_sha256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
    $signature = Get-AuthenticodeSignature -LiteralPath $exe
    if ($signature.Status -ne 'Valid' -or -not $signature.SignerCertificate) { throw 'Installer signature invalid' }
    $receipt.signer = $signature.SignerCertificate.Subject
    if ($signature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) -ne 'Microsoft Corporation') { throw 'Installer signer is not Microsoft Corporation' }
    # The measured digest is provenance, not a preauthenticated content pin.
    # Signed installer plus exact installed version and all64 existing hashes gate use.
    $arguments = @('--quiet','--wait','--norestart','--noUpdateInstaller',
        '--installPath', ('"' + $install + '"'),
        '--add','Microsoft.VisualStudio.Workload.VCTools','--includeRecommended')
    $child = Start-Process -FilePath $exe -ArgumentList $arguments -PassThru
    if (-not $child.WaitForExit(1200000)) {
        & taskkill.exe /PID $child.Id /T /F | Out-Null
        throw 'Installer exceeded 1200s; owned process tree termination requested'
    }
    $child.Refresh()
    $receipt.exit_code = $child.ExitCode
    if ($child.ExitCode -ne 0) { throw "Installer exit $($child.ExitCode); reboot/retry not admitted" }
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $raw = & $vswhere -all -products '*' -format json
    if ($LASTEXITCODE -ne 0) { throw 'VS discovery failed' }
    $instances = @($raw | ConvertFrom-Json | Where-Object { $_.installationPath -eq $install })
    if ($instances.Count -ne 1 -or $instances[0].installationVersion -ne '18.10.12210.168') { throw 'Restored VS instance version/path mismatch' }
    & "$PSScriptRoot/arti_windows_compiler.ps1" -InstallationPath $install
    $receipt.status = 'PASS'
} catch {
    $receipt.error = $_.Exception.Message
    throw
} finally {
    $receipt.duration_ms = $clock.ElapsedMilliseconds
    $receipt | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $work 'restoration_receipt.json') -Encoding utf8
}
