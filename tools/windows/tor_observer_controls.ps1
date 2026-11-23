param([switch]$SelfTest)

function Convert-WfpProcessNumber([string]$Value) {
    if ($Value -match '^0x[0-9a-f]+$') { return [Convert]::ToUInt64($Value.Substring(2), 16) }
    if ($Value -match '^[0-9]+$') { return [Convert]::ToUInt64($Value, 10) }
    throw 'Malformed WFP ProcessID'
}
function Test-WfpControlEvent($Record, $Control) {
    try {
        $d = $Record.data
        return ($Record.event_id -eq 5156 -and $Record.record_id -gt $Control.baseline_record_id -and
            (Convert-WfpProcessNumber $d.ProcessID) -eq $Control.process_id -and
            $d.Direction -eq '%%14593' -and $d.Protocol -eq '6' -and
            $d.SourceAddress -eq $Control.local_address -and $d.SourcePort -eq [string]$Control.local_port -and
            $d.DestAddress -eq $Control.remote_address -and $d.DestPort -eq [string]$Control.remote_port)
    } catch { return $false }
}
if ($SelfTest) {
    $control = @{ process_id = 4556; baseline_record_id = 100; local_address = '192.0.2.1'; local_port = 49152; remote_address = '192.0.2.2'; remote_port = 443 }
    $base = @{ ProcessID = '4556'; Direction = '%%14593'; Protocol = '6'; SourceAddress = '192.0.2.1'; SourcePort = '49152'; DestAddress = '192.0.2.2'; DestPort = '443' }
    $vectors = @(
        @{ id = 'decimal_pid'; field = 'ProcessID'; value = '4556'; expected = $true },
        @{ id = 'hex_pid'; field = 'ProcessID'; value = '0x11cc'; expected = $true },
        @{ id = 'wrong_pid'; field = 'ProcessID'; value = '4557'; expected = $false },
        @{ id = 'malformed_pid'; field = 'ProcessID'; value = 'not-a-pid'; expected = $false },
        @{ id = 'inbound'; field = 'Direction'; value = '%%14592'; expected = $false },
        @{ id = 'udp'; field = 'Protocol'; value = '17'; expected = $false },
        @{ id = 'wrong_source_address'; field = 'SourceAddress'; value = '192.0.2.3'; expected = $false },
        @{ id = 'wrong_source_port'; field = 'SourcePort'; value = '49153'; expected = $false },
        @{ id = 'wrong_destination'; field = 'DestAddress'; value = '192.0.2.3'; expected = $false },
        @{ id = 'wrong_destination_port'; field = 'DestPort'; value = '444'; expected = $false },
        @{ id = 'prior_record'; record_id = 100; expected = $false },
        @{ id = 'blocked_event'; event_id = 5157; expected = $false }
    )
    foreach ($v in $vectors) {
        $data = $base.Clone()
        if ($v.field) { $data[$v.field] = $v.value }
        $record = @{ event_id = 5156; record_id = 101; data = $data }
        if ($v.ContainsKey('record_id')) { $record.record_id = $v.record_id }
        if ($v.ContainsKey('event_id')) { $record.event_id = $v.event_id }
        if ((Test-WfpControlEvent $record $control) -ne $v.expected) { throw ('Matcher self-test failed: ' + $v.id) }
    }
    [pscustomobject]@{ scope = 'synthetic WFP matcher only; no policy/network access'; result = 'PASS'; count = $vectors.Count; cases = @($vectors.id) } | ConvertTo-Json -Depth 4
    return
}

$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_OS -ne 'Windows' -or
    $env:RUNNER_ENVIRONMENT -ne 'github-hosted') { throw 'Hosted Windows runner required' }

$auditArgs = @('/get', '/subcategory:Filtering Platform Connection', '/r')
$out = Join-Path $env:RUNNER_TEMP 'pulqva-network-observer'
New-Item -ItemType Directory -Path $out | Out-Null
$trace = Join-Path $env:RUNNER_TEMP 'pulqva-dns-observer.etl'
$policy = Join-Path $env:RUNNER_TEMP 'pulqva-audit-policy.csv'
$timer = [Diagnostics.Stopwatch]::StartNew()
$receipt = [ordered]@{
    schema = 2
    script_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    budgets = @{ tcp_connect_ms = 8000; wfp_delivery_ms = 10000; wfp_poll_ms = 250; max_events_per_read = 2048; max_candidate_records = 64 }
    cleanup = [ordered]@{ trace_stop_exitcode = $null; policy_restore_exitcode = $null; policy_restored_equal = $null }
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

    $receipt.audit_policy_before = @(& auditpol @auditArgs)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read original WFP audit policy' }
    & auditpol /backup "/file:$policy" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot back up WFP audit policy' }
    $backup = $true
    & auditpol /set '/subcategory:Filtering Platform Connection' /success:enable /failure:enable | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enable WFP audit' }

    $receipt.audit_policy_enabled = @(& auditpol @auditArgs)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read enabled WFP audit policy' }

    & logman start PulqvaDnsObserver -p Microsoft-Windows-DNS-Client 0xffffffffffffffff 5 -o $trace -ets | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Cannot start Windows DNS Client ETW' }
    $tracing = $true
    $bookmark = (Get-WinEvent -LogName Security -MaxEvents 1 -ErrorAction Stop).RecordId
    $receipt.control = [ordered]@{ process_id = $PID; process_path = (Get-Process -Id $PID).Path; process_start_utc = (Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o'); baseline_record_id = $bookmark; start_utc = [DateTime]::UtcNow.ToString('o') }
    $probeStart = $timer.ElapsedMilliseconds
    $client = [Net.Sockets.TcpClient]::new([Net.Sockets.AddressFamily]::InterNetwork)
    try {
        if (-not $client.ConnectAsync($addr, 443).Wait(8000)) { throw 'Direct TCP control timed out' }
        $receipt.control.local_address = $client.Client.LocalEndPoint.Address.IPAddressToString
        $receipt.control.local_port = $client.Client.LocalEndPoint.Port
        $receipt.control.remote_address = $client.Client.RemoteEndPoint.Address.IPAddressToString
        $receipt.control.remote_port = $client.Client.RemoteEndPoint.Port
        $receipt.control.connect_completed_utc = [DateTime]::UtcNow.ToString('o')
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
    $receipt.cleanup.trace_stop_exitcode = $LASTEXITCODE
    if ($LASTEXITCODE -ne 0) { throw 'Cannot flush Windows DNS Client ETW' }
    $tracing = $false

    $dns = @(Get-WinEvent -Path $trace -Oldest -ErrorAction Stop | Where-Object {
        $_.ProviderName -eq 'Microsoft-Windows-DNS-Client' -and
            (($_.Properties.Value -contains $canary) -or ($_.Properties.Value -contains ($canary + '.')))
    })
    $receipt.probes.ETW_DNS.matches = $dns.Count
    $receipt.probes.ETW_DNS.event_ids = @($dns.Id | Sort-Object -Unique)
    $receipt.probes.ETW_DNS.result = if ($dns.Count) { 'PASS' } else { 'FAIL' }
    $delivery = [Diagnostics.Stopwatch]::StartNew()
    $seen = @{}
    $matches = @()
    $readCount = 0
    $candidates = @()
    $receipt.wfp_query_errors = @()
    $xpath = "*[System[(EventID=5156 or EventID=5157) and EventRecordID > $bookmark]]"
    do {
        $events = @()
        $readCount++
        try { $events = @(Get-WinEvent -LogName Security -FilterXPath $xpath -Oldest -MaxEvents 2048 -ErrorAction Stop) }
        catch {
            if ($_.FullyQualifiedErrorId -notlike 'NoMatchingEventsFound*') {
                $receipt.wfp_query_errors += $_.Exception.Message
                throw
            }
        }
        if ($events.Count -ge 2048) { throw 'WFP observation exceeded bounded event capacity' }
        foreach ($event in $events) {
            $data = @{}
            foreach ($field in ([xml]$event.ToXml()).Event.EventData.Data) { $data[$field.Name] = $field.'#text' }
            $row = @{ event_id = $event.Id; record_id = $event.RecordId; time_utc = $event.TimeCreated.ToUniversalTime().ToString('o'); data = $data }
            if (-not $seen.ContainsKey($event.RecordId)) {
                $seen[$event.RecordId] = $true
                $sameProcess = $false
                try { $sameProcess = (Convert-WfpProcessNumber $data.ProcessID) -eq $PID } catch {}
                if ($sameProcess -or ($data.DestAddress -eq $receipt.control.remote_address -and $data.DestPort -eq '443')) {
                    if ($candidates.Count -ge 64) { throw 'WFP candidate evidence exceeded bounded capacity' }
                    $candidates += $row
                }
                if (Test-WfpControlEvent $row $receipt.control) { $matches += $row }
            }
        }
        if ($matches.Count -or $delivery.ElapsedMilliseconds -ge 10000) { break }
        Start-Sleep -Milliseconds 250
    } while ($delivery.ElapsedMilliseconds -lt 10000)
    $receipt.wfp_candidate_records = $candidates
    $receipt.wfp_delivery_ms = $delivery.ElapsedMilliseconds
    $receipt.wfp_reads = $readCount
    $receipt.wfp_event_count = $seen.Count
    $receipt.probes.WFP_DIRECT_TCP4.matches = $matches.Count
    $receipt.probes.WFP_DIRECT_TCP4.result = if ($matches.Count) { 'PASS' } else { 'FAIL' }
    if ($matches.Count -and $dns.Count) { $receipt.result = 'PASS' }
} catch {
    $failure = $_.Exception.Message
    if (Get-Variable candidates -ErrorAction SilentlyContinue) { $receipt.wfp_candidate_records = $candidates }
} finally {
    if ($tracing) {
        & logman stop PulqvaDnsObserver -ets | Out-Null
        $receipt.cleanup.trace_stop_exitcode = $LASTEXITCODE
        if ($LASTEXITCODE -ne 0) { $receipt.result = 'INCONCLUSIVE'; $failure = 'DNS trace cleanup failed' }
    }
    if ($backup) {
        & auditpol /restore "/file:$policy" | Out-Null
        $receipt.cleanup.policy_restore_exitcode = $LASTEXITCODE
        if ($LASTEXITCODE -ne 0) { $receipt.result = 'INCONCLUSIVE'; $failure = 'Audit policy restoration failed' }
        $receipt.audit_policy_restored = @(& auditpol @auditArgs)
        $getExit = $LASTEXITCODE
        $receipt.cleanup.policy_restored_equal = (($receipt.audit_policy_before -join "`n") -ceq ($receipt.audit_policy_restored -join "`n")) -and $getExit -eq 0
        if (-not $receipt.cleanup.policy_restored_equal) { $receipt.result = 'INCONCLUSIVE'; $failure = 'Audit policy readback mismatch' }
    }
    $receipt.error = $failure
    $receipt.elapsed_ms = $timer.ElapsedMilliseconds
    $receipt | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $out 'matrix_receipt.json') -Encoding utf8
    $receipt | ConvertTo-Json -Depth 7
}
if ($receipt.result -ne 'PASS') { throw 'Observer positive controls are incomplete' }
