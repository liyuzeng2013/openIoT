@echo off
REM OpenIoT Build Script

if "%1"=="" goto help
if "%1"=="build" goto build
if "%1"=="build-server" goto build-server
if "%1"=="build-client" goto build-client
if "%1"=="run-server" goto run-server
if "%1"=="run-client" goto run-client
if "%1"=="clean" goto clean
if "%1"=="deploy-esp32" goto deploy-esp32
if "%1"=="restart-esp32" goto restart-esp32
if "%1"=="repl-esp32" goto repl-esp32
if "%1"=="help" goto help
goto help

:build
echo [BUILD] Building workspace...
cargo build --workspace
goto end

:build-server
echo [BUILD] Building server...
cargo build -p openiot-server
goto end

:build-client
echo [BUILD] Building client...
cargo build -p openiot-client
goto end

:run-server
echo [RUN] Starting server on http://localhost:3000
cargo run -p openiot-server
goto end

:run-client
echo [RUN] Starting Windows client...
cargo run -p openiot-client
goto end

:clean
echo [CLEAN] Cleaning build artifacts...
cargo clean
goto end

:deploy-esp32
echo [DEPLOY] Deploying ESP32 firmware...
mpremote connect auto cp esp32/main.py esp32/config.py esp32/provision.py esp32/protocol.py esp32/mesh.py esp32/device.py :
goto end

:restart-esp32
echo [RESTART] Restarting ESP32...
mpremote connect auto reset
goto end

:repl-esp32
echo [REPL] Opening ESP32 REPL...
mpremote connect auto repl
goto end

:help
echo OpenIoT Build Commands:
echo   build.bat build          - Build entire workspace
echo   build.bat build-server   - Build server only
echo   build.bat build-client   - Build Windows client only
echo   build.bat run-server     - Run server (http://localhost:3000)
echo   build.bat run-client     - Run Windows client
echo   build.bat clean          - Clean build artifacts
echo   build.bat deploy-esp32   - Deploy ESP32 firmware
echo   build.bat restart-esp32  - Restart ESP32
echo   build.bat repl-esp32     - Open ESP32 REPL
goto end

:end
