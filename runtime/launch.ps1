# Atollo: the request client of the levels (written by Atollo.exe to
# data\kit\launch.ps1; edits are replaced at the next start). DECISIONS.md
# D-179, docs/app/LEVELS.md section 4.3.
#
# Started by the editor's profiles and by the forwarders of the agents'
# commands (data\kit\forward), as:
#   pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File launch.ps1
#        -Origin editorTask -Command <line>          (a task: VSCodium adds
#                                                     -Command, or -c, and the line)
#        -Origin editorAgentProfile -Agent <id>      (an agent's profile)
#        -Origin forwarded -Agent <id> -- <args...>  (claude, codex, pi, agy typed)
#        -Origin editorExtension -Extension <ext-id> -Agent <id> -- <args...>
#                                                    (an editor extension's wrapper,
#                                                     LEVELS.md 6.9; not wired, Q2)
#
# It decides nothing: it writes a request in data\app\levels\requests\ with
# the origin and the current folder, waits for Atollo's reply, and runs what
# the reply says in this terminal (stdio passed through), exiting with its
# code. A refusal prints why and what to do, and exits 1.
#
# The words are read here, not bound by PowerShell: what the user typed
# after an agent's command (after `--`, or right after -Agent <id> of a
# forwarder) is passed on as it is, even a word like -c; VSCodium passes a
# task's line after -Command on Windows and -c on macOS and Linux.

$ErrorActionPreference = 'Stop'

function Say([string]$text) { [Console]::Error.WriteLine($text) }

$words = @($args)
$Origin = ''
$Agent = ''
$From = ''
$Extension = ''
$Command = $null
$rest = @()
$i = 0
while ($i -lt $words.Count) {
    $word = [string]$words[$i]
    if ($word -eq '--' -or (($Origin -eq 'forwarded' -or $Origin -eq 'editorExtension') -and $Agent)) {
        $at = if ($word -eq '--') { $i + 1 } else { $i }
        if ($at -lt $words.Count) { $rest = @($words[$at..($words.Count - 1)]) }
        break
    }
    $value = if ($i + 1 -lt $words.Count) { [string]$words[$i + 1] } else { '' }
    switch ($word) {
        '-Origin' { $Origin = $value }
        '-Agent' { $Agent = $value }
        '-From' { $From = $value }
        '-Extension' { $Extension = $value }
        { $_ -eq '-Command' -or $_ -eq '-c' } { $Command = $value }
        default {
            Say "launch.ps1: what is $word?"
            exit 2
        }
    }
    $i += 2
}

$requests = [IO.Path]::Combine($PSScriptRoot, '..', 'app', 'levels', 'requests')
$here = (Get-Location).ProviderPath

switch ($Origin) {
    'editorTask' {
        if ($null -eq $Command -or $Command -eq '') {
            Say 'launch.ps1: -Origin editorTask -Command <line>'
            exit 2
        }
        $what = [ordered]@{ kind = 'editorTask'; command = $Command }
    }
    'editorAgentProfile' {
        $what = [ordered]@{ kind = 'editorAgentProfile'; agent = $Agent }
    }
    'forwarded' {
        $level = if ($From) { $From } else { $env:ATOLLO_LEVEL }
        if ($level -ne 'manual' -and $level -ne 'project') { $level = 'manual' }
        $what = [ordered]@{ kind = 'forwarded'; agent = $Agent; from = $level }
    }
    'editorExtension' {
        $what = [ordered]@{ kind = 'editorExtension'; extension = $Extension; agent = $Agent }
    }
    default {
        Say 'launch.ps1: -Origin editorTask|editorAgentProfile|forwarded|editorExtension'
        exit 2
    }
}

# The request, written whole then renamed: Atollo never reads half of one.
$id = [guid]::NewGuid().ToString('N')
[void][IO.Directory]::CreateDirectory($requests)
$request = Join-Path $requests "$id.json"
$tmp = Join-Path $requests "$id.tmp"
$body = [ordered]@{ v = 1; origin = $what; cwd = $here; args = [string[]]$rest } | ConvertTo-Json -Depth 5 -Compress
[IO.File]::WriteAllText($tmp, $body, [Text.UTF8Encoding]::new($false))
[IO.File]::Move($tmp, $request)

# The reply: within seconds, or minutes the first time a VM starts. Atollo
# takes the request at once while it runs: one still there after the wait
# was never taken, and is taken back.
$replyJson = Join-Path $requests "$id.reply.json"
$replySh = Join-Path $requests "$id.reply.sh"
$wait = 15
if ($env:ATOLLO_LAUNCH_WAIT -match '^[0-9]+$') { $wait = [int]$env:ATOLLO_LAUNCH_WAIT }
$start = Get-Date
$said = $false
# The reply for sh is written last: with it both are there.
while (-not (Test-Path -LiteralPath $replySh)) {
    $waited = ((Get-Date) - $start).TotalSeconds
    if ($waited -ge $wait -and (Test-Path -LiteralPath $request)) {
        $gone = $false
        try { Remove-Item -LiteralPath $request -ErrorAction Stop; $gone = $true } catch { }
        if ($gone) {
            Say "Atollo non e' aperto: apri il terminale da Atollo."
            exit 1
        }
    }
    if ($waited -ge 600) {
        Remove-Item -LiteralPath $request -ErrorAction SilentlyContinue
        Say "Atollo non ha risposto in 10 minuti: riprova, o guarda il registro di Atollo."
        exit 1
    }
    if (-not $said -and $waited -ge 3) {
        Say '[Atollo] In preparazione...'
        $said = $true
    }
    Start-Sleep -Milliseconds 200
}
$reply = [IO.File]::ReadAllText($replyJson, [Text.Encoding]::UTF8) | ConvertFrom-Json
Remove-Item -LiteralPath $replyJson, $replySh -ErrorAction SilentlyContinue

if ($reply.placement -ne 'vm' -and $reply.placement -ne 'pc') {
    Say $reply.text
    exit 1
}
if ($reply.banner) { Say $reply.banner }
if ($reply.env.set) {
    foreach ($var in $reply.env.set.PSObject.Properties) {
        [Environment]::SetEnvironmentVariable($var.Name, [string]$var.Value)
    }
}
foreach ($name in @($reply.env.unset)) {
    if ($name) { [Environment]::SetEnvironmentVariable($name, $null) }
}
$argv = @($reply.argv)

if ($reply.marker) {
    # Pi's session, seen by Atollo on the PC (D-178): a marker with this
    # PowerShell's pid while it runs, in a folder no VM mounts. The port of
    # the local models stays open only while a marker's pid runs a client.
    $marker = Join-Path $requests "$($reply.marker).$PID"
    Set-Content -LiteralPath $marker -Value $PID
    try {
        & $reply.program @argv
    } finally {
        Remove-Item -LiteralPath $marker -ErrorAction SilentlyContinue
    }
} else {
    & $reply.program @argv
}
exit $LASTEXITCODE
