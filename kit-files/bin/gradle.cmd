@echo off
rem Dev-Kit: the kit's Gradle on the newest JDK of the kit (Gradle 9 needs
rem Java 17 or later); a project's Java compiles through its toolchain,
rem which finds the kit's JDKs (data\gradle\gradle.properties).
setlocal
if not exist "%CP_APPS%\gradle\bin\gradle.bat" (
  echo Gradle is not installed: Tools ^> Java 1>&2
  exit /b 1
)
if defined DEVKIT_GRADLE_JAVA set "JAVA_HOME=%DEVKIT_GRADLE_JAVA%"
call "%CP_APPS%\gradle\bin\gradle.bat" %*
exit /b %ERRORLEVEL%
