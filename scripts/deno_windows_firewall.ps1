param(
    [Parameter(Mandatory=$true)][ValidateSet("Install","Verify","Remove")][string]$Mode,
    [Parameter(Mandatory=$true)][string]$RuleName,
    [Parameter(Mandatory=$true)][string]$Executable
)
$ErrorActionPreference = "Stop"
if ($RuleName -notmatch '^PULQVA-Deno-[0-9a-f]{32}$') { throw "Invalid owned rule name" }
if ($Mode -eq "Remove") {
    Get-NetFirewallRule -Name $RuleName -ErrorAction SilentlyContinue | Remove-NetFirewallRule
    if (Get-NetFirewallRule -Name $RuleName -ErrorAction SilentlyContinue) { throw "Rule remains" }
    exit 0
}
if ((Get-Service MpsSvc).Status -ne "Running") { throw "Firewall service not running" }
$profiles = @(Get-NetFirewallProfile -PolicyStore ActiveStore)
if ($profiles.Count -ne 3 -or @($profiles | Where-Object { $_.Enabled -ne "True" }).Count -ne 0) {
    throw "All effective firewall profiles must already be enabled"
}
$program = (Resolve-Path -LiteralPath $Executable).Path
if ($Mode -eq "Install") {
    New-NetFirewallRule -Name $RuleName -DisplayName $RuleName -Program $program -Direction Outbound -Action Block -Profile Any -Enabled True -Protocol Any -RemoteAddress Any | Out-Null
}
$rule = Get-NetFirewallRule -Name $RuleName -PolicyStore ActiveStore
if ($rule.Enabled -ne "True" -or $rule.Action -ne "Block" -or $rule.Direction -ne "Outbound" -or $rule.Profile -ne "Any") {
    throw "Effective rule mismatch"
}
$filter = $rule | Get-NetFirewallApplicationFilter
if ($filter.Program -ine $program) { throw "Program filter mismatch" }
$address = $rule | Get-NetFirewallAddressFilter
$port = $rule | Get-NetFirewallPortFilter
if ($address.RemoteAddress -ne "Any" -or $port.Protocol -ne "Any") { throw "Traffic filter mismatch" }
@{kind="windows_program_outbound_block"; effective_store="ActiveStore"; profiles_enabled=$true; program=$program; action="Block"; protocol="Any"; remote_address="Any"} | ConvertTo-Json -Compress
