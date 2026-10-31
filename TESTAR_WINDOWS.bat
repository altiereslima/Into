@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
 echo Rust/Cargo nao encontrado. Instale via https://rustup.rs/ e abra novamente.
 pause
 exit /b 1
)
echo Testando todo o workspace (nao precisa dos arquivos do jogo)...
cargo test --workspace
if errorlevel 1 (
 echo Falha nos testes. Copie as mensagens desta janela para o proximo chat.
 pause
 exit /b 1
)
echo Testes concluidos.
pause
