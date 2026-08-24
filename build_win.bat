@echo off
echo ========================================
echo   Building Windows (x86_64) Release
echo ========================================
cargo build --release
if %errorlevel% neq 0 (
    echo [FAIL] Windows build failed!
    pause
    exit /b %errorlevel%
)
echo [OK] Windows build complete: target\release\cube_3d.exe
pause
