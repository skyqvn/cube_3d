@echo off
echo ========================================
echo   Building WASM (wasm32) Release
echo ========================================

echo [1/2] Compiling Rust to WASM...
cargo build --target wasm32-unknown-unknown --release
if %errorlevel% neq 0 (
    echo [FAIL] WASM build failed!
    pause
    exit /b %errorlevel%
)

echo [2/2] Generating JS glue code...
wasm-bindgen target/wasm32-unknown-unknown/release/cube_3d.wasm --out-dir www/pkg --target web
if %errorlevel% neq 0 (
    echo [FAIL] wasm-bindgen failed!
    pause
    exit /b %errorlevel%
)

echo ========================================
echo [OK] WASM build complete!
echo.
echo   Serve locally with:
echo     python -m http.server 8000 -d www
echo     Then open http://127.0.0.1:8000
echo ========================================
pause
