@echo off
chcp 65001 >nul
cd /d "%~dp0"
invoice-printer.exe --dir . --out output.pdf
echo.
echo ===== 完成！已生成 output.pdf =====
pause
