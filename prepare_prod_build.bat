@echo off
setlocal

echo Starting asset preparation for Tauri installer...

:: --- Configuration ---
:: Path to the original client.exe (built from your client Rust project) relative to the project root.
set "ClientSourcePath=target\release\client.exe"

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
    echo Client executable not found. Running "cargo build -p %ClientCargoPackageName% --release"...
    cargo build -p "%ClientCargoPackageName%" --release
    if %errorlevel% neq 0 (
        echo Error: Failed to build client package "%ClientCargoPackageName%". Please check your client project setup.
        exit /b %errorlevel%
    )
    echo Client build complete.
) else (
    echo Client executable already exists. Skipping build.
)


:: --- Copying Assets ---

:: 1. Copy the ORIGINAL `client.exe` to the 'stub' subfolder in the staging directory,
::    keeping its name as `client.exe` for the installer.
echo Copying %ClientSourcePath% to %ClientStubDir%\client.exe
if exist "%ClientSourcePath%" (
    copy /y "%ClientSourcePath%" "%ClientStubDir%\client.exe" >nul
) else (
    echo Error: %ClientSourcePath% not found after build attempt! This indicates a problem.
    exit /b 1
)

:: 2. Copy countries.mmdb from /res/ to the 'resources' subdirectory in the staging directory.
echo Copying %CountriesMmdbSourcePath% to %ResourcesSubdir%
if exist "%CountriesMmdbSourcePath%" (
    copy /y "%CountriesMmdbSourcePath%" "%ResourcesSubdir%" >nul
) else (
    echo Error: %CountriesMmdbSourcePath% not found! Please place it in your project root's /res/ folder.
    exit /b 1
)

:: 3. Copy rcedit.bkp from /res/, rename it to rcedit.exe, and place it under the 'resources' subdirectory in the staging directory.
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