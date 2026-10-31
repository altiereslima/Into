@echo off
setlocal
cd /d "%~dp0"
set /p JOGO="Pasta do Into the Breach (ex.: C:\Program Files (x86)\Steam\steamapps\common\Into the Breach): "
cargo run --release -p itb-lua --bin itb-boot -- "%JOGO%" > boot_relatorio.txt
type boot_relatorio.txt
echo.
echo Relatorio salvo em boot_relatorio.txt
pause
