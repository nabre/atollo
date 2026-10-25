# Atollo: the editor's terminals, tasks and agents in the project's VM
# (written by Atollo.exe to data\kit\vm.ps1; edits are replaced at the next
# start). DECISIONS.md D-175.
#
# Started by VSCodium's profiles as:
#   pwsh -NoLogo -NoProfile [-NoExit] -ExecutionPolicy Bypass -File vm.ps1
#        -Shell pwsh|bash|cmd|zsh [-Exe <program>] [-Agent <id>]
# and, for a task, with "-Command <line>" added by VSCodium.
#
# The app notes in data\app\sandboxes\sessions\ the level and what the
# project's VM needs (isolation/session.rs). Fail closed: with the level
# `vm` the terminal opens in the project's VM or not at all; only with the
# level `env` the kit's own shell or agent runs on the PC.

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

# The level the app noted when the editor opened (fail closed: only `env`
# lets anything run on the PC; none yet, a moment to write it, then nothing).
$levelFile = Join-Path $sessions '.level'
$until = (Get-Date).AddSeconds(15)
while (-not (Test-Path -LiteralPath $levelFile) -and (Get-Date) -lt $until) {
    Start-Sleep -Milliseconds 250
}
$level = if (Test-Path -LiteralPath $levelFile) { (Get-Content -LiteralPath $levelFile -Raw).Trim() } else { '' }
if ($level -ne 'vm' -and $level -ne 'env') {
    Say "Atollo non ha detto se la VM e' attiva: apri l'editor dalla pagina del progetto in Atollo."
    Leave 1
}
$session = if ($project -and $level -eq 'vm') { Join-Path $sessions $project } else { $null }

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
        if ($Agent -ne 'claude' -and $Agent -ne 'codex') {
            Say "$Agent non ha una sandbox di Docker Sandboxes: nella VM si usano Claude Code e Codex."
            Leave 1
        }
        # Its sandbox made ready by the app (tools, database, ports) on a
        # request: the first time, or again at every start of the editor.
        $file = Join-Path $session $Agent
        $failed = Join-Path $session "$Agent.error"
        if (-not (Test-Path -LiteralPath $file)) {
            Remove-Item -LiteralPath $failed -ErrorAction SilentlyContinue
            Set-Content -LiteralPath "$session.$Agent.request" -Value '1'
            Say "La sandbox di $Agent si sta preparando..."
            $until = (Get-Date).AddMinutes(10)
            while (-not (Test-Path -LiteralPath $file) -and -not (Test-Path -LiteralPath $failed) -and (Get-Date) -lt $until) {
                Start-Sleep -Milliseconds 500
            }
        }
        if (-not (Test-Path -LiteralPath $file)) {
            $why = if (Test-Path -LiteralPath $failed) { (Get-Content -LiteralPath $failed -Raw).Trim() } else { 'nessuna risposta da Atollo' }
            Say "La sandbox di $Agent non e' pronta: $why"
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

if ($level -eq 'vm') {
    if ($session) {
        Say "La VM di questo progetto non e' stata preparata: riapri l'editor dalla pagina del progetto in Atollo."
    } else {
        Say "Con la VM attiva il terminale dell'editor si apre solo nella cartella di un progetto."
    }
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
