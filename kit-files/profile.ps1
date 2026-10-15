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

# Versions of the languages chosen for the project (data\app\projects\<project>\
# versions.json, written by Atollo.exe): the editor's terminals start from
# the editor server, shared by every project, so the folders of the
# project's versions take the place of the kit's here (DECISIONS.md D-80).
$projects = Join-Path $env:CP_ROOT 'projects'
$here = (Get-Location).ProviderPath
if ($here.StartsWith($projects + '\', [StringComparison]::OrdinalIgnoreCase)) {
    $project = $here.Substring($projects.Length + 1).Split('\')[0]
    $file = Join-Path $env:CP_DATA "app\projects\$project\versions.json"
    if (Test-Path -LiteralPath $file) {
        try {
            $swaps = @((Get-Content -LiteralPath $file -Raw | ConvertFrom-Json).swaps)
            foreach ($name in @([Environment]::GetEnvironmentVariables().Keys)) {
                $value = [Environment]::GetEnvironmentVariable($name)
                $new = @($value -split ';' | ForEach-Object {
                    $entry = $_
                    foreach ($s in $swaps) {
                        if ($entry -ieq $s.from -or $entry.StartsWith($s.from + '\', [StringComparison]::OrdinalIgnoreCase)) {
                            $entry = $s.to + $entry.Substring($s.from.Length)
                            break
                        }
                    }
                    $entry
                }) -join ';'
                if ($new -cne $value) { [Environment]::SetEnvironmentVariable($name, $new) }
            }
        } catch {
            Write-Host "Atollo: versions of $project not applied ($_)." -ForegroundColor Yellow
        }
    }
}
Remove-Variable projects, here, project, file, swaps, name, value, new -ErrorAction Ignore

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
