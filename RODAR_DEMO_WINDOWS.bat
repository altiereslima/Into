@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
 echo Rust/Cargo nao encontrado. Instale via https://rustup.rs/ e abra novamente.
 pause
 exit /b 1
)
echo Compilando e abrindo laboratorio de teclado, mouse e controle...
cargo run --release -p itb-core --features sdl2-backend --bin input_lab
if errorlevel 1 (
 echo Falha de compilacao/execucao. Consulte README.md (Rust, MSVC/CMake e SDL2).
 pause
 exit /b 1
)
pause
