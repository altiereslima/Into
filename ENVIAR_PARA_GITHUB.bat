@echo off
setlocal
cd /d "%~dp0"
echo === Enviar este projeto para https://github.com/altiereslima/Into ===
where git >nul 2>nul
if errorlevel 1 (
 echo O Git nao esta instalado. Baixe em https://git-scm.com/download/win , instale e rode este arquivo de novo.
 pause
 exit /b 1
)
git remote get-url origin >nul 2>nul
if errorlevel 1 git remote add origin https://github.com/altiereslima/Into.git
rem Se houver um pacote .bundle novo nesta pasta (ex.: Into_etapa08.bundle), incorpora antes de enviar.
for %%B in (Into_etapa*.bundle) do (
 echo Incorporando %%B ...
 git pull --ff-only "%%B" main
)
echo Enviando... (na primeira vez o Git abre o navegador para voce entrar no GitHub)
git push -u origin main
if errorlevel 1 (
 echo.
 echo O envio falhou. Se o repositorio no GitHub ja tinha algum arquivo criado pelo site,
 echo rode:  git pull --rebase origin main   e depois este arquivo de novo.
 pause
 exit /b 1
)
echo.
echo Pronto! Confira em https://github.com/altiereslima/Into
pause
