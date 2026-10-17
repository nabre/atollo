# Dev-Kit: profile of the kit's PowerShell 7 (written by Dev-Kit.exe to
# data\kit\profile.ps1; edits are replaced at the next start).
# Started as: pwsh -NoLogo -NoProfile -NoExit -ExecutionPolicy Bypass -File profile.ps1
# The PC's profiles and modules are never loaded: the terminal sees only the
# kit (DECISIONS.md D-23).

# Modules: PowerShell's own and the kit's, not Documents\PowerShell or
# Program Files of the PC. Paths and lists are built with the separators of
# the system (';' and '\' on Windows, ':' and '/' on macOS and Linux).
$kitModules = [IO.Path]::Combine($env:CP_DATA, 'pwsh', 'Modules')
$env:PSModulePath = (Join-Path $PSHOME 'Modules') + [IO.Path]::PathSeparator + $kitModules

# .NET asks Windows for the AppData folder and ignores APPDATA: point the
# history at the kit explicitly. On macOS and Linux PSReadLine keeps it under
# XDG_DATA_HOME, already in the kit.
if (Get-Module -ListAvailable PSReadLine) {
    Import-Module PSReadLine
    if ($env:OS -eq 'Windows_NT') {
        $history = [IO.Path]::Combine($env:APPDATA, 'Microsoft', 'Windows', 'PowerShell', 'PSReadLine', 'ConsoleHost_history.txt')
        New-Item -ItemType Directory -Force -Path (Split-Path $history) | Out-Null
        Set-PSReadLineOption -HistorySavePath $history -PredictionSource History
    } else {
        Set-PSReadLineOption -PredictionSource History
    }
}

# Paths compare ignoring case where the file system does (Windows, macOS),
# exactly on Linux ($IsLinux is unset, so false, in Windows PowerShell 5.1).
$sep = [IO.Path]::PathSeparator
$slash = [IO.Path]::DirectorySeparatorChar
$cmp = if ($IsLinux) { [StringComparison]::Ordinal } else { [StringComparison]::OrdinalIgnoreCase }

# Versions of the languages chosen for the project (data\projects\<project>\
# versions.json, written by Dev-Kit.exe): the editor's terminals start from
# the editor server, shared by every project, so the folders of the
# project's versions take the place of the kit's here (DECISIONS.md D-80).
# A kit terminal (DEVKIT_TERMINAL, the Terminals page) keeps its own.
$projects = Join-Path $env:CP_ROOT 'projects'
$here = (Get-Location).ProviderPath
if (-not $env:DEVKIT_TERMINAL -and $here.StartsWith($projects + $slash, $cmp)) {
    $project = $here.Substring($projects.Length + 1).Split($slash)[0]
    $file = [IO.Path]::Combine($env:CP_DATA, 'projects', $project, 'versions.json')
    if (Test-Path -LiteralPath $file) {
        try {
            $swaps = @((Get-Content -LiteralPath $file -Raw | ConvertFrom-Json).swaps)
            foreach ($name in @([Environment]::GetEnvironmentVariables().Keys)) {
                $value = [Environment]::GetEnvironmentVariable($name)
                if ($null -eq $value) { continue }
                $new = @($value.Split($sep) | ForEach-Object {
                    $entry = $_
                    foreach ($s in $swaps) {
                        if ([string]::Equals($entry, $s.from, $cmp) -or $entry.StartsWith($s.from + $slash, $cmp)) {
                            $entry = $s.to + $entry.Substring($s.from.Length)
                            break
                        }
                    }
                    $entry
                }) -join $sep
                if ($new -cne $value) { [Environment]::SetEnvironmentVariable($name, $new) }
            }
        } catch {
            Write-Host "Dev-Kit: versions of $project not applied ($_)." -ForegroundColor Yellow
        }
    }
}
Remove-Variable projects, here, project, file, swaps, name, value, new, sep, slash, cmp -ErrorAction Ignore

function global:prompt {
    $here = $ExecutionContext.SessionState.Path.CurrentLocation.ProviderPath
    $projects = Join-Path $env:CP_ROOT 'projects'
    $cmp = if ($IsLinux) { [StringComparison]::Ordinal } else { [StringComparison]::OrdinalIgnoreCase }
    if ($here.StartsWith($projects, $cmp)) {
        $here = '~projects' + $here.Substring($projects.Length)
    }
    $esc = [char]27
    "$esc[38;5;75mDev-Kit$esc[0m $esc[38;5;180m$here$esc[0m$('>' * ($NestedPromptLevel + 1)) "
}

if (-not $env:DEVKIT_QUIET) {
    # On the kit's PATH: the languages are in the folder of the version in
    # use (data\apps\<platform>\node\<line>), not at a fixed path.
    $tools = 'claude', 'codex', 'agy', 'git', 'node', 'python', 'uv', 'gh', 'php', 'cargo', 'go', 'java', 'dotnet'
    $found = foreach ($name in $tools) {
        if (Get-Command -Name $name -CommandType Application -ErrorAction Ignore) { $name }
    }
    Write-Host "Dev-Kit" -ForegroundColor Cyan -NoNewline
    if ($env:DEVKIT_TERMINAL) { Write-Host " · $env:DEVKIT_TERMINAL" -ForegroundColor Cyan -NoNewline }
    Write-Host ("  " + ($found -join '  ')) -ForegroundColor DarkGray
}
