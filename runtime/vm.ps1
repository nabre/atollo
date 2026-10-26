# Atollo: the editor's terminals, tasks and agents in the project's VM
# (written by Atollo.exe to data\kit\vm.ps1; edits are replaced at the next
# start). DECISIONS.md D-175, D-178.
#
# Started by VSCodium's profiles as:
#   pwsh -NoLogo -NoProfile [-NoExit] -ExecutionPolicy Bypass -File vm.ps1
#        -Shell pwsh|bash|cmd|zsh [-Exe <program>] [-Agent <id>]
# and, for a task, with "-Command <line>" added by VSCodium.
#
# The app notes in data\app\sandboxes\sessions\ the level and what the
# project's VM needs (isolation/session.rs). Fail closed: the terminal opens
# in the project's VM or not at all. Without the VM (level `env`) nothing
# opens on the PC (D-178): the reason the app noted is shown, with what to
# do. -Shell and -Exe are what the profiles pass; nothing of the PC runs.

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

# The level the app noted when the editor opened (none yet: a moment to
# write it, then nothing).
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

# Without the VM: nothing of the editor starts on the PC (D-178), only why.
if ($level -eq 'env') {
    $reasonFile = Join-Path $sessions '.reason'
    if (Test-Path -LiteralPath $reasonFile) {
        foreach ($line in Get-Content -LiteralPath $reasonFile -Encoding utf8) {
            if ($line.Trim()) { Say $line }
        }
    } else {
        Say "Senza VM: i terminali, i task e gli agenti dell'editor partono solo nella VM del progetto. Il motivo e cosa fare sono nell'avviso 'Senza VM' di Atollo."
    }
    Leave 1
}

# Antigravity is a desktop editor: it cannot run in a microVM (D-178).
if ($Agent -eq 'antigravity') {
    Say "Antigravity e' un editor per il desktop e non puo' girare in una microVM: su un progetto non si apre. Nella VM usa Claude Code, Codex o Pi."
    Leave 1
}
if ($Agent -and $Agent -ne 'claude' -and $Agent -ne 'codex' -and $Agent -ne 'pi') {
    Say "$Agent non parte nella VM: nella VM si usano Claude Code, Codex e Pi."
    Leave 1
}

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
        # Its session made ready by the app (Claude Code and Codex in their
        # sandboxes, Pi in the project's VM: tools, database, ports) on a
        # request: the first time, or again at every start of the editor.
        $file = Join-Path $session $Agent
        $failed = Join-Path $session "$Agent.error"
        if (-not (Test-Path -LiteralPath $file)) {
            Remove-Item -LiteralPath $failed -ErrorAction SilentlyContinue
            Set-Content -LiteralPath "$session.$Agent.request" -Value '1'
            Say "La sessione di $Agent nella VM si sta preparando..."
            $until = (Get-Date).AddMinutes(10)
            while (-not (Test-Path -LiteralPath $file) -and -not (Test-Path -LiteralPath $failed) -and (Get-Date) -lt $until) {
                Start-Sleep -Milliseconds 500
            }
        }
        if (-not (Test-Path -LiteralPath $file)) {
            $why = if (Test-Path -LiteralPath $failed) { (Get-Content -LiteralPath $failed -Raw).Trim() } else { 'nessuna risposta da Atollo' }
            Say "La sessione di $Agent nella VM non e' pronta: $why"
            Leave 1
        }
        $argv = @(Get-Content -LiteralPath $file)
        # Pi starts in the folder the terminal is in, like a shell.
        $w = [Array]::IndexOf($argv, '-w')
        if ($w -ge 0 -and $below) { $argv[$w + 1] = $argv[$w + 1].TrimEnd('/') + '/' + $below }
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
    if ($Agent -eq 'pi') {
        # Pi's session, seen by the app on the PC (D-178): a marker with
        # this PowerShell's pid while `sbx exec` runs, beside the project's
        # session in a folder no VM mounts. The port of the local models
        # stays open only while a marker's pid runs the kit's PowerShell.
        $marker = Join-Path $sessions "$project.pi.$PID"
        Set-Content -LiteralPath $marker -Value $PID
        try {
            & $sbx @argv
        } finally {
            Remove-Item -LiteralPath $marker -ErrorAction SilentlyContinue
        }
    } else {
        & $sbx @argv
    }
    Leave $LASTEXITCODE
}

if ($session) {
    Say "La VM di questo progetto non e' stata preparata: riapri l'editor dalla pagina del progetto in Atollo."
} else {
    Say "Con la VM attiva il terminale dell'editor si apre solo nella cartella di un progetto."
}
Leave 1
