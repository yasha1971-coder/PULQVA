$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or
    $env:RUNNER_ENVIRONMENT -ne 'github-hosted') { throw 'Hosted Windows runner required' }

$out = Join-Path $env:RUNNER_TEMP 'pulqva-network-observer'
New-Item -ItemType Directory -Path $out | Out-Null
$trace = Join-Path $env:RUNNER_TEMP 'pulqva-dns-observer.etl'
$policy = Join-Path $env:RUNNER_TEMP 'pulqva-audit-policy.csv'
$timer = [Diagnostics.Stopwatch]::StartNew()
$receipt = [ordered]@{
    schema = 1
    scope = 'observer-positive-controls'
    checkout = (git rev-parse HEAD).Trim()
    package_commit = '8b01623d8a66c06d05cad89a0e472c8712623b2e'
    package_sha256 = '4ff69b6d91f7535d6361b640f9572743fdecd9a36772b8d782bc4d3d073a3efa'
    generation_id = [guid]::NewGuid().ToString()
    run = $env:GITHUB_RUN_ID
    job = $env:GITHUB_JOB
    attempt = $env:GITHUB_RUN_ATTEMPT
    os = [Environment]::OSVersion.VersionString
    arch = $env:RUNNER_ARCH
    image_version = $env:ImageVersion
    runtime_executed = $false
    probes = [ordered]@{
        WFP_DIRECT_TCP4 = [ordered]@{ expected = 'observer sees a direct IPv4 TCP connection from the control process'; result = 'SKIP'; duration_ms = $null }
        ETW_DNS = [ordered]@{ expected = 'observer sees a fresh control DNS name'; result = 'SKIP'; duration_ms = $null }
    }
    result = 'INCONCLUSIVE'
}
$backup = $false
$tracing = $false
$failure = $null
try {
    if ($receipt.checkout -ne $env:GITHUB_SHA) { throw 'Checkout identity mismatch' }
    $addr = [Net.Dns]::GetHostAddresses('api.github.com') |
        Where-Object AddressFamily -eq InterNetwork | Select-Object -First 1
    if (-not $addr) { throw 'Control endpoint has no IPv4 address' }

    & auditpol /backup "/file:$policy" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot back up WFP audit policy' }
    $backup = $true
    & auditpol /set '/subcategory:Filtering Platform Connection' /success:enable /failure:enable | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enable WFP audit' }

    & logman start PulqvaDnsObserver -p Microsoft-Windows-DNS-Client 0xffffffffffffffff 5 -o $trace -ets | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot start Windows DNS Client ETW' }
    $tracing = $true
    $start = Get-Date
    $probeStart = $timer.ElapsedMilliseconds
    $client = [Net.Sockets.TcpClient]::new([Net.Sockets.AddressFamily]::InterNetwork)
    try {
        if (-not $client.ConnectAsync($addr, 443).Wait(8000)) { throw 'Direct TCP control timed out' }
    } finally { $client.Dispose() }
    $receipt.probes.WFP_DIRECT_TCP4.duration_ms = $timer.ElapsedMilliseconds - $probeStart

    $probeStart = $timer.ElapsedMilliseconds
    $canary = 'pulqva-' + [guid]::NewGuid().ToString('N') + '.example.com'
    try { [Net.Dns]::GetHostAddresses($canary) | Out-Null }
    catch [Net.Sockets.SocketException] {
        if ($_.Exception.SocketErrorCode -notin @('HostNotFound','NoData')) { throw }
    }
    $receipt.probes.ETW_DNS.duration_ms = $timer.ElapsedMilliseconds - $probeStart
    & logman stop PulqvaDnsObserver -ets | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot flush Windows DNS Client ETW' }
    $tracing = $false

    $wfp = @(Get-WinEvent -FilterHashtable @{ LogName = 'Security'; Id = 5156; StartTime = $start } -ErrorAction SilentlyContinue |
        Where-Object {
            $data = @{}
            foreach ($field in ([xml]$_.ToXml()).Event.EventData.Data) { $data[$field.Name] = $field.'#text' }
            $data.ProcessID -eq [string]$PID -and $data.DestAddress -eq $addr.IPAddressToString -and
                $data.DestPort -eq '443' -and $data.Protocol -eq '6'
        })
    $receipt.probes.WFP_DIRECT_TCP4.matches = $wfp.Count
    $receipt.probes.WFP_DIRECT_TCP4.result = if ($wfp.Count) { 'PASS' } else { 'FAIL' }

    $dns = @(Get-WinEvent -Path $trace -Oldest -ErrorAction Stop | Where-Object {
        $_.ProviderName -eq 'Microsoft-Windows-DNS-Client' -and
            (($_.Properties.Value -contains $canary) -or ($_.Properties.Value -contains ($canary + '.')))
    })
    $receipt.probes.ETW_DNS.matches = $dns.Count
    $receipt.probes.ETW_DNS.event_ids = @($dns.Id | Sort-Object -Unique)
    $receipt.probes.ETW_DNS.result = if ($dns.Count) { 'PASS' } else { 'FAIL' }
    if ($wfp.Count -and $dns.Count) { $receipt.result = 'PASS' }
} catch {
    $failure = $_.Exception.Message
} finally {
    if ($tracing) { & logman stop PulqvaDnsObserver -ets | Out-Null }
    if ($backup) {
        & auditpol /restore "/file:$policy" | Out-Null
        if ($LASTEXITCODE -ne 0) { $receipt.result = 'INCONCLUSIVE'; $failure = 'Audit policy restoration failed' }
    }
    $receipt.error = $failure
    $receipt.elapsed_ms = $timer.ElapsedMilliseconds
    $receipt | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $out 'matrix_receipt.json') -Encoding utf8
    $receipt | ConvertTo-Json -Depth 7
}
if ($receipt.result -ne 'PASS') { throw 'Observer positive controls are incomplete' }
