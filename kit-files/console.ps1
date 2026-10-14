# Atollo: start of a console in the kit's Windows Terminal (written by
# Atollo.exe to data\kit\console.ps1; edits are replaced at the next start).
# Started as: pwsh -NoLogo -NoProfile [-NoExit] -ExecutionPolicy Bypass -File console.ps1 <pipe> [<program>]
# Windows Terminal starts a new tab with its own environment, not the one of
# who asked for it: the app sends the kit's environment through the pipe and
# learns this process's id from it, to know when the console ends
# (DECISIONS.md D-68). Without the kit's environment nothing runs.

param(
    [Parameter(Mandatory)][string]$Pipe,
    [string]$Program
)

try {
    $client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $Pipe, [System.IO.Pipes.PipeDirection]::In)
    $client.Connect(10000)
    $text = [System.IO.StreamReader]::new($client).ReadToEnd()
    $client.Dispose()
    $vars = $text | ConvertFrom-Json -AsHashtable
    if (-not $vars -or -not $vars.Count) { throw 'empty' }
} catch {
    Write-Host "Atollo: the kit's environment did not arrive ($_)." -ForegroundColor Red
    Start-Sleep -Seconds 10
    exit 1
}

# Only the kit's variables: names are case-insensitive on Windows.
$keep = [System.Collections.Generic.HashSet[string]]::new([string[]]$vars.Keys, [StringComparer]::OrdinalIgnoreCase)
foreach ($name in @([Environment]::GetEnvironmentVariables().Keys)) {
    if (-not $keep.Contains($name)) { [Environment]::SetEnvironmentVariable($name, $null) }
}
foreach ($v in $vars.GetEnumerator()) {
    [Environment]::SetEnvironmentVariable($v.Key, [string]$v.Value)
}
Remove-Variable client, text, vars, keep, name, v -ErrorAction Ignore

if ($Program) {
    & $Program
    exit $LASTEXITCODE
}
& (Join-Path $PSScriptRoot 'profile.ps1')
