@echo off
rem Atollo: agy with its Google login carried into Windows before the run
rem and back into the kit after it (Atollo.exe keyring, see keyring.rs).
rem Written by Atollo.exe; edits are replaced at the next start.
setlocal
rem The Gemini API key of the settings, for agy only: the kit's
rem environment does not hold it.
if not defined GEMINI_API_KEY if exist "%CP_DATA%\app\secrets\gemini-api-key" set /p GEMINI_API_KEY=<"%CP_DATA%\app\secrets\gemini-api-key"
"%ATOLLO_EXE%" keyring restore
"%CP_ANTIGRAVITY%\agy.exe" %*
set "AGY_EXIT=%ERRORLEVEL%"
"%ATOLLO_EXE%" keyring save
exit /b %AGY_EXIT%
