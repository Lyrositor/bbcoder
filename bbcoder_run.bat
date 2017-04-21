@ECHO OFF
set PATH=%PATH%;%CD%;%~dp0
SET project="%~1"
IF "%~1"=="" (SET project="project.xml")
ECHO Running bbcoder on %project%...
bbcoder -p %project% %~2
ECHO Complete.
PAUSE
