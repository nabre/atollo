@echo off
rem Atollo: the kit's MCP servers that need a secret of the kit (D-156):
rem `atollo-mcp github` is GitHub's server with the kit's account. The
rem token is read by Atollo.exe and sent to GitHub only: it is never in
rem the agent's environment, so the MCP servers of a project do not see it.
rem Written by Atollo.exe; edits are replaced at the next start.
"%ATOLLO_EXE%" mcp %*
exit /b %ERRORLEVEL%
