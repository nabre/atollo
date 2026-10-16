@echo off
rem Dev-Kit: agy with its Google login carried into Windows before the run
rem and back into the kit after it (Dev-Kit.exe keyring, see keyring.rs).
rem Written by Dev-Kit.exe; edits are replaced at the next start.
setlocal
"%DEVKIT_EXE%" keyring restore
"%CP_APPS%\antigravity\agy.exe" %*
set "AGY_EXIT=%ERRORLEVEL%"
"%DEVKIT_EXE%" keyring save
exit /b %AGY_EXIT%
