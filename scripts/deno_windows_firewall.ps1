param(
    [Parameter(Mandatory=$true)][ValidateSet("Install","Verify","Remove")][string]$Mode,
    [Parameter(Mandatory=$true)][string]$RuleName,
    [Parameter(Mandatory=$true)][string]$Executable
)
$ErrorActionPreference = "Stop"
$clock = [System.Diagnostics.Stopwatch]::StartNew()
function Trace-Phase([string]$Step) {
    [Console]::Error.WriteLine("PULQVA_FIREWALL_PHASE mode=$Mode elapsed_ms=$($clock.ElapsedMilliseconds) step=$Step")
}
Trace-Phase "script-start"
if ($RuleName -notmatch '^PULQVA-Deno-[0-9a-f]{32}$') { throw "Invalid owned rule name" }
if ($Mode -eq "Remove") {
    Trace-Phase "remove-rule-begin"
    Get-NetFirewallRule -Name $RuleName -ErrorAction SilentlyContinue | Remove-NetFirewallRule
    Trace-Phase "remove-rule-end-verify-absence-begin"
    if (Get-NetFirewallRule -Name $RuleName -ErrorAction SilentlyContinue) { throw "Rule remains" }
    Trace-Phase "verify-absence-end"
    exit 0
}
Trace-Phase "service-begin"
if ((Get-Service MpsSvc).Status -ne "Running") { throw "Firewall service not running" }
Trace-Phase "service-end-profiles-begin"
$profiles = @(Get-NetFirewallProfile -PolicyStore ActiveStore)
Trace-Phase "profiles-end"
if ($profiles.Count -ne 3 -or @($profiles | Where-Object { $_.Enabled -ne "True" }).Count -ne 0) {
    throw "All effective firewall profiles must already be enabled"
}
Trace-Phase "resolve-program-begin"
$program = (Resolve-Path -LiteralPath $Executable).Path
Trace-Phase "resolve-program-end"
if ($Mode -eq "Install") {
    Trace-Phase "create-rule-begin"
    New-NetFirewallRule -Name $RuleName -DisplayName $RuleName -Program $program -Direction Outbound -Action Block -Profile Any -Enabled True -Protocol Any -RemoteAddress Any | Out-Null
    Trace-Phase "create-rule-end"
}
Trace-Phase "active-rule-begin"
$rule = Get-NetFirewallRule -Name $RuleName -PolicyStore ActiveStore
Trace-Phase "active-rule-end"
if ($rule.Enabled -ne "True" -or $rule.Action -ne "Block" -or $rule.Direction -ne "Outbound" -or $rule.Profile -ne "Any") {
    throw "Effective rule mismatch"
}
Trace-Phase "application-filter-begin"
$filter = $rule | Get-NetFirewallApplicationFilter
Trace-Phase "application-filter-end"
if ($filter.Program -ine $program) { throw "Program filter mismatch" }
Trace-Phase "address-filter-begin"
$address = $rule | Get-NetFirewallAddressFilter
Trace-Phase "address-filter-end-port-filter-begin"
$port = $rule | Get-NetFirewallPortFilter
Trace-Phase "port-filter-end"
if ($address.RemoteAddress -ne "Any" -or $port.Protocol -ne "Any") { throw "Traffic filter mismatch" }
@{kind="windows_program_outbound_block"; effective_store="ActiveStore"; profiles_enabled=$true; program=$program; action="Block"; protocol="Any"; remote_address="Any"} | ConvertTo-Json -Compress
