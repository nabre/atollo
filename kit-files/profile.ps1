# Atollo: profile of the kit's PowerShell 7 (written by Atollo.exe to
# data\kit\profile.ps1; edits are replaced at the next start).
# Started as: pwsh -NoLogo -NoProfile -NoExit -ExecutionPolicy Bypass -File profile.ps1
# The PC's profiles and modules are never loaded: the terminal sees only the
# kit (DECISIONS.md D-23).

# Modules: PowerShell's own and the kit's, not Documents\PowerShell or
# Program Files of the PC.
$kitModules = Join-Path $env:CP_DATA 'pwsh\Modules'
$env:PSModulePath = (Join-Path $PSHOME 'Modules') + ';' + $kitModules

# .NET asks Windows for the AppData folder and ignores APPDATA: point the
# history at the kit explicitly.
if (Get-Module -ListAvailable PSReadLine) {
    Import-Module PSReadLine
    $history = Join-Path $env:APPDATA 'Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt'
    New-Item -ItemType Directory -Force -Path (Split-Path $history) | Out-Null
    Set-PSReadLineOption -HistorySavePath $history -PredictionSource History
}

function global:prompt {
    $here = $ExecutionContext.SessionState.Path.CurrentLocation.ProviderPath
    $projects = Join-Path $env:CP_ROOT 'projects'
    if ($here.StartsWith($projects, [StringComparison]::OrdinalIgnoreCase)) {
        $here = '~projects' + $here.Substring($projects.Length)
    }
    $esc = [char]27
    "$esc[38;5;75mAtollo$esc[0m $esc[38;5;180m$here$esc[0m$('>' * ($NestedPromptLevel + 1)) "
}

if (-not $env:ATOLLO_QUIET) {
    $tools = [ordered]@{
        'claude'  = 'claude\claude.exe'
        'codex'   = 'codex\bin\codex.exe'
        'agy'     = 'antigravity\agy.exe'
        'git'     = 'git\cmd\git.exe'
        'node'    = 'node\node.exe'
        'python'  = 'python\python.exe'
        'uv'      = 'tools\uv.exe'
        'gh'      = 'tools\gh.exe'
        'php'     = 'php\php.exe'
        'cargo'   = 'rust\cargo\bin\cargo.exe'
        'go'      = 'go\bin\go.exe'
        'java'    = 'java\bin\java.exe'
        'dotnet'  = 'dotnet\dotnet.exe'
    }
    $found = foreach ($name in $tools.Keys) {
        if (Test-Path -LiteralPath (Join-Path $env:CP_APPS $tools[$name])) { $name }
    }
    Write-Host "Atollo" -ForegroundColor Cyan -NoNewline
    Write-Host ("  " + ($found -join '  ')) -ForegroundColor DarkGray
}
