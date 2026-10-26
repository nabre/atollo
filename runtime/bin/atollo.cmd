@echo off
rem Atollo: the app's command line from the kit's terminals
rem (`atollo components`, `atollo check --kit`, `atollo help`): the exe
rem the kit runs with (ATOLLO_EXE, set by Atollo for its terminals).
if not defined ATOLLO_EXE (
  echo Atollo: ATOLLO_EXE is not set: open the terminal from Atollo 1>&2
  exit /b 1
)
"%ATOLLO_EXE%" %*
exit /b %ERRORLEVEL%
