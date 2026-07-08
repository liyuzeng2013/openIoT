@echo off
chcp 65001 >nul
echo ============================================
echo   OpenIoT Virtual Device Simulator
echo ============================================
echo.

where python >nul 2>nul
if not errorlevel 1 (
    set PY=python
    goto :found
)

where py >nul 2>nul
if not errorlevel 1 (
    set PY=py
    goto :found
)

where python3 >nul 2>nul
if not errorlevel 1 (
    set PY=python3
    goto :found
)

if exist "C:\Python311\python.exe" (
    set PY=C:\Python311\python.exe
    goto :found
)
if exist "C:\Python312\python.exe" (
    set PY=C:\Python312\python.exe
    goto :found
)

echo [ERROR] Python not found in PATH
echo Please install Python from https://python.org and check "Add to PATH"
pause
exit /b 1

:found
echo Using Python: %PY%
echo.

%PY% -c "import requests" 2>nul
if errorlevel 1 (
    echo [INFO] First run, installing dependencies...
    %PY% -m pip install requests websocket-client
    echo.
)

%PY% "%~dp0virtual_device.py"
pause
