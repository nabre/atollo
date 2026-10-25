# Atollo: the editor's terminals, tasks and agents in the project's VM
# (written by Atollo.exe to data\kit\vm.ps1; edits are replaced at the next
# start). DECISIONS.md D-175.
#
# Started by VSCodium's profiles as:
#   pwsh -NoLogo -NoProfile [-NoExit] -ExecutionPolicy Bypass -File vm.ps1
#        -Shell pwsh|bash|cmd|zsh [-Exe <program>] [-Agent <id>]
# and, for a task, with "-Command <line>" added by VSCodium.
#
# The app notes in data\app\sandboxes\sessions\ what the project's VM needs
# (isolation/session.rs). With a session the terminal opens in the VM;
# with the VM on but no session (a folder that is no project's) nothing
# opens; without the VM the kit's own shell or agent runs on the PC.

param(
    [string]$Shell = 'pwsh',
    [string]$Exe = '',
    [string]$Agent = '',
    [string]$Command = $null
)

$ErrorActionPreference = 'Stop'
$sep = [IO.Path]::DirectorySeparatorChar
$sessions = [IO.Path]::Combine($env:CP_DATA, 'app', 'sandboxes', 'sessions')

function Leave([int]$code) { [Environment]::Exit($code) }

function Say([string]$text) { [Console]::Error.WriteLine("[Atollo] $text") }

# The project of the current folder: its name and the part below it.
$here = (Get-Location).ProviderPath
$projects = [IO.Path]::GetFullPath(($env:CP_PROJECTS_FWD -replace '/', $sep))
$project = $null
$below = ''
$ignore = if ($env:OS -eq 'Windows_NT' -or $IsMacOS) { 'OrdinalIgnoreCase' } else { 'Ordinal' }
if ($here.StartsWith($projects + $sep, [StringComparison]::$ignore)) {
    $rest = $here.Substring($projects.Length + 1)
    $parts = $rest.Split($sep, 2)
    $project = $parts[0]
    if ($parts.Count -gt 1) { $below = $parts[1] -replace '\\', '/' }
}

$vmOn = Test-Path -LiteralPath (Join-Path $sessions '.vm')
$session = if ($project) { Join-Path $sessions $project } else { $null }

# Being made ready by the app: wait, at most ten minutes.
if ($session -and (Test-Path -LiteralPath "$session.pending")) {
    Say 'La VM del progetto si sta preparando...'
    $until = (Get-Date).AddMinutes(10)
    while ((Test-Path -LiteralPath "$session.pending") -and (Get-Date) -lt $until) {
        Start-Sleep -Milliseconds 500
    }
}

if ($session -and (Test-Path -LiteralPath (Join-Path $session 'error'))) {
    Say ("La VM del progetto non e' pronta: " + (Get-Content -LiteralPath (Join-Path $session 'error') -Raw).Trim())
    Leave 1
}

if ($session -and (Test-Path -LiteralPath (Join-Path $session 'sbx'))) {
    $sbx = (Get-Content -LiteralPath (Join-Path $session 'sbx') -Raw).Trim()
    foreach ($line in Get-Content -LiteralPath (Join-Path $session 'env')) {
        $i = $line.IndexOf('=')
        if ($i -gt 0) { [Environment]::SetEnvironmentVariable($line.Substring(0, $i), $line.Substring($i + 1)) }
    }
    if ($Agent) {
        $file = Join-Path $session $Agent
        if (-not (Test-Path -LiteralPath $file)) {
            Say "$Agent non ha una sandbox di Docker Sandboxes: nella VM si usano Claude Code e Codex."
            Leave 1
        }
        $argv = @(Get-Content -LiteralPath $file)
    } elseif ($null -ne $Command -and $Command -ne '') {
        $argv = @(Get-Content -LiteralPath (Join-Path $session 'run'))
        $last = $argv.Count - 1
        $line = if ($below) { "cd '" + ($below -replace "'", "'\''") + "' && " + $Command } else { $Command }
        $argv[$last] = $argv[$last].Replace('__ATOLLO_LINE__', $line)
    } else {
        $argv = @(Get-Content -LiteralPath (Join-Path $session 'shell'))
        $w = [Array]::IndexOf($argv, '-w')
        if ($w -ge 0 -and $below) { $argv[$w + 1] = $argv[$w + 1].TrimEnd('/') + '/' + $below }
    }
    & $sbx @argv
    Leave $LASTEXITCODE
}

if ($vmOn) {
    Say "Con la VM attiva il terminale dell'editor si apre solo nella cartella di un progetto."
    Leave 1
}

# Without the VM: the kit's own, on the PC.
if ($Agent) {
    & $Exe
    Leave $LASTEXITCODE
}
if ($null -ne $Command -and $Command -ne '') {
    switch ($Shell) {
        'pwsh' { Invoke-Expression $Command; Leave $LASTEXITCODE }
        default { & $Exe -c $Command; Leave $LASTEXITCODE }
    }
}
switch ($Shell) {
    'pwsh' { $ErrorActionPreference = 'Continue'; . (Join-Path $PSScriptRoot 'profile.ps1') }
    'bash' { & $Exe --rcfile (Join-Path $PSScriptRoot 'bashrc') -i; Leave $LASTEXITCODE }
    'cmd' { & $Exe /d /k; Leave $LASTEXITCODE }
    default { & $Exe -i; Leave $LASTEXITCODE }
}
