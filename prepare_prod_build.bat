@echo off
setlocal

echo Starting asset preparation for Tauri installer...

:: --- Parameter Handling ---
set "BuildMode=--release"
if /I "%1"=="-manual" (
    echo Manual mode detected. Using debug build.
    set "BuildMode=--debug"
)

:: --- Configuration ---
:: Path to the original client.exe depending on build mode
if "%BuildMode%"=="--release" (
    set "ClientSourcePath=target\release\client.exe"
) else (
    set "ClientSourcePath=target\debug\client.exe"
)

:: IMPORTANT: This should be the 'name' from your client's Cargo.toml (e.g., [package] name = "client")
set "ClientCargoPackageName=client"

:: Source paths for rcedit.bkp and countries.mmdb from your /res/ folder.
set "RceditSourcePath=.\res\rcedit.bkp"
set "CountriesMmdbSourcePath=.\res\countries.mmdb"

:: Staging directory where assets will be prepared for Tauri's bundler.
set "StagingDir=.\.tauri_installer_assets_staging"

:: Subdirectory for the client.exe within the staging area.
set "ClientStubDir=%StagingDir%\stub"

:: Subdirectory for countries.mmdb and rcedit.exe within the staging area.
set "ResourcesSubdir=%StagingDir%\resources"

:: --- Cleanup & Setup ---
echo Cleaning up previous staging directory: %StagingDir%
if exist "%StagingDir%" rd /s /q "%StagingDir%"

echo Creating staging directories...
mkdir "%ClientStubDir%" >nul 2>nul
mkdir "%ResourcesSubdir%" >nul 2>nul


:: --- Check and Build Client (if needed) ---
echo Checking for client executable at: %ClientSourcePath%
if not exist "%ClientSourcePath%" (
    echo Client executable not found. Running "cargo build -p %ClientCargoPackageName% %BuildMode%"...
    cargo build -p "%ClientCargoPackageName%" %BuildMode%
    if %errorlevel% neq 0 (
        echo Error: Failed to build client package "%ClientCargoPackageName%". Please check your client project setup.
        exit /b %errorlevel%
    )
    echo Client build complete.
) else (
    echo Client executable already exists. Skipping build.
)


:: --- Copying Assets ---

:: 1. Copy client.exe to stub folder
echo Copying %ClientSourcePath% to %ClientStubDir%\client.exe
if exist "%ClientSourcePath%" (
    copy /y "%ClientSourcePath%" "%ClientStubDir%\client.exe" >nul
) else (
    echo Error: %ClientSourcePath% not found after build attempt! This indicates a problem.
    exit /b 1
)

:: 2. Copy countries.mmdb
echo Copying %CountriesMmdbSourcePath% to %ResourcesSubdir%
if exist "%CountriesMmdbSourcePath%" (
    copy /y "%CountriesMmdbSourcePath%" "%ResourcesSubdir%" >nul
) else (
    echo Error: %CountriesMmdbSourcePath% not found! Please place it in your project root's /res/ folder.
    exit /b 1
)

:: 3. Copy and rename rcedit.bkp
echo Copying and renaming %RceditSourcePath% to %ResourcesSubdir%\rcedit.exe
if exist "%RceditSourcePath%" (
    copy /y "%RceditSourcePath%" "%ResourcesSubdir%\rcedit.exe" >nul
) else (
    echo Error: %RceditSourcePath% not found! Please place it in your project root's /res/ folder.
    exit /b 1
)

echo Asset preparation complete. Staging directory: %StagingDir%
echo Contents:
dir /b /s "%StagingDir%" 2>nul

endlocal
