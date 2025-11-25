@echo off
echo 构建服务器和客户端...
cargo build --bin server
if %errorlevel% neq 0 (
    echo 服务器构建失败
    pause
    exit /b
)

cargo build --bin client
if %errorlevel% neq 0 (
    echo 客户端构建失败
    pause
    exit /b
)

echo 启动服务器...
start "物理沙盒服务器" .\target\debug\server.exe
echo 等待服务器启动...
timeout /t 3
echo 启动客户端...
.\target\debug\client.exe
pause