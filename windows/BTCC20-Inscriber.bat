@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul

cd /d "%~dp0"

set PROFILE=mainnet
if "%~1"=="--profile" (
  if not "%~2"=="" set PROFILE=%~2
)
if /i "%~1"=="--profile=local" set PROFILE=local
if /i "%~1"=="--profile=mainnet" set PROFILE=mainnet
set CONFIG_FILE=btcc20-profiles.conf
if not exist "%CONFIG_FILE%" if exist "..\btcc20-profiles.conf" set CONFIG_FILE=..\btcc20-profiles.conf

if not exist "ord.exe" (
  echo [错误] 当前目录没有 ord.exe
  echo 请确认你下载的是 Windows release 压缩包，并且已经完整解压。
  pause
  exit /b 1
)

if not exist "%CONFIG_FILE%" (
  echo [错误] 当前目录没有 %CONFIG_FILE%
  echo 请确认 release 压缩包已经完整解压。
  pause
  exit /b 1
)

call :load_profile
if errorlevel 1 exit /b 1

:menu
cls
echo ==========================================
echo              BTCC-20 Inscriber
echo ==========================================
echo Profile: %PROFILE%
echo 链:     %CHAIN%
echo RPC:    %RPC_URL%
echo 钱包:   %WALLET%
echo.
echo 1. Deploy 部署
echo 2. Mint 铸造
echo 3. Transfer 创建转账铭文
echo 4. 查看帮助
echo 5. 切换 Profile
echo 0. 退出
echo.
set /p ACTION=请选择:

if "%ACTION%"=="1" goto deploy
if "%ACTION%"=="2" goto mint
if "%ACTION%"=="3" goto transfer
if "%ACTION%"=="4" goto help
if "%ACTION%"=="5" goto switch_profile
if "%ACTION%"=="0" exit /b 0
goto menu

:deploy
call :common_tick
set /p MAX=Max Supply 默认 21000000000:
if "%MAX%"=="" set MAX=21000000000
set /p LIM=Mint Limit 默认 1000:
if "%LIM%"=="" set LIM=1000
set /p DEC=Decimals 默认 18:
if "%DEC%"=="" set DEC=18
set /p DEST=Destination 可留空:
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% deploy --tick %TICK% --max %MAX% --lim %LIM%
if not "%DEC%"=="" set CMD=%CMD% --dec %DEC%
if not "%DEST%"=="" set CMD=%CMD% --destination %DEST%
goto run

:mint
call :common_tick
set /p AMT=Amount 默认 1000:
if "%AMT%"=="" set AMT=1000
set /p DEST=Destination 可留空:
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% mint --tick %TICK% --amt %AMT%
if not "%DEST%"=="" set CMD=%CMD% --destination %DEST%
goto run

:transfer
call :common_tick
set /p AMT=Amount 默认 1000:
if "%AMT%"=="" set AMT=1000
set /p DEST=Destination 可留空:
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% transfer --tick %TICK% --amt %AMT%
if not "%DEST%"=="" set CMD=%CMD% --destination %DEST%
goto run

:help
ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --help
pause
goto menu

:switch_profile
echo.
echo 1. mainnet 正式环境
echo 2. local 本地 regtest
echo.
set /p PROFILE_ACTION=请选择:
if "%PROFILE_ACTION%"=="1" set PROFILE=mainnet
if "%PROFILE_ACTION%"=="2" set PROFILE=local
call :load_profile
if errorlevel 1 pause
goto menu

:common_tick
set /p TICK=Ticker 默认 cord:
if "%TICK%"=="" set TICK=cord
exit /b 0

:run
echo.
echo 即将执行:
echo %CMD%
echo.
set /p CONFIRM=确认执行? 输入 y 继续:
if /i not "%CONFIRM%"=="y" goto menu
echo.
%CMD%
echo.
pause
goto menu

:load_profile
set CHAIN=
set RPC_URL=
set RPC_USER=
set RPC_PASSWORD=
set WALLET=
set IN_SECTION=0
set FOUND=0
for /f "usebackq tokens=* delims=" %%L in ("%CONFIG_FILE%") do (
  set LINE=%%L
  for /f "tokens=1 delims=#" %%A in ("!LINE!") do set LINE=%%A
  if not "!LINE!"=="" (
    if "!LINE:~0,1!"=="[" (
      set SECTION=!LINE:~1,-1!
      if /i "!SECTION!"=="%PROFILE%" (
        set IN_SECTION=1
        set FOUND=1
      ) else (
        set IN_SECTION=0
      )
    ) else if "!IN_SECTION!"=="1" (
      for /f "tokens=1,* delims==" %%A in ("!LINE!") do (
        if /i "%%A"=="chain" set CHAIN=%%B
        if /i "%%A"=="rpc_url" set RPC_URL=%%B
        if /i "%%A"=="rpc_user" set RPC_USER=%%B
        if /i "%%A"=="rpc_password" set RPC_PASSWORD=%%B
        if /i "%%A"=="wallet" set WALLET=%%B
      )
    )
  )
)
if "%FOUND%"=="0" (
  echo [错误] %CONFIG_FILE% 里找不到 profile: %PROFILE%
  exit /b 1
)
exit /b 0
