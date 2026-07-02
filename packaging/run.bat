@echo off
chcp 65001 >nul
cd /d "%~dp0"
invoice-printer.exe --dir ./source --out ./out/output.pdf
echo.
echo ===== 完成！已生成 ./out/output.pdf =====
pause
