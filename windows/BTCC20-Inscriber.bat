@echo off
setlocal EnableExtensions EnableDelayedExpansion

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
  echo [ERROR] ord.exe was not found in the current directory.
  echo Make sure the Windows release archive was fully extracted.
  pause
  exit /b 1
)

if not exist "%CONFIG_FILE%" (
  echo [ERROR] %CONFIG_FILE% was not found in the current directory.
  echo Make sure the release archive was fully extracted.
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
echo Chain:   %CHAIN%
echo RPC:     %RPC_URL%
echo Wallet:  %WALLET%
echo.
echo Parameter files:
echo   Deploy:   deploy.txt
echo   Mint:     mint.txt
echo   Transfer: transfer.txt
echo.
echo 1. Deploy from deploy.txt
echo 2. Mint from mint.txt
echo 3. Transfer inscription from transfer.txt
echo 4. Help
echo 5. Switch profile
echo 0. Exit
echo.
set /p ACTION=Choose:

if "%ACTION%"=="1" goto deploy
if "%ACTION%"=="2" goto mint
if "%ACTION%"=="3" goto transfer
if "%ACTION%"=="4" goto help
if "%ACTION%"=="5" goto switch_profile
if "%ACTION%"=="0" exit /b 0
goto menu

:deploy
call :load_params deploy.txt
if errorlevel 1 goto pause_menu
if "%TICK%"=="" set TICK=cord
if "%MAX%"=="" set MAX=21000000000
if "%LIM%"=="" set LIM=1000
if "%DEC%"=="" set DEC=18
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% deploy --tick %TICK% --max %MAX% --lim %LIM%
if not "%DEC%"=="" set CMD=%CMD% --dec %DEC%
if not "%DESTINATION%"=="" set CMD=%CMD% --destination %DESTINATION%
goto run_single

:mint
call :load_params mint.txt
if errorlevel 1 goto pause_menu
if "%TICK%"=="" set TICK=cord
if "%AMT%"=="" set AMT=1000
if "%COUNT%"=="" set COUNT=1
set /a MINT_COUNT=%COUNT% >nul 2>nul
if "%MINT_COUNT%"=="" set MINT_COUNT=1
if %MINT_COUNT% LSS 1 set MINT_COUNT=1
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% mint --tick %TICK% --amt %AMT%
if not "%DESTINATION%"=="" set CMD=%CMD% --destination %DESTINATION%
goto run_mint_many

:transfer
call :load_params transfer.txt
if errorlevel 1 goto pause_menu
if "%TICK%"=="" set TICK=cord
if "%AMT%"=="" set AMT=1000
set CMD=ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --wallet %WALLET% transfer --tick %TICK% --amt %AMT%
if not "%DESTINATION%"=="" set CMD=%CMD% --destination %DESTINATION%
goto run_single

:help
ord.exe --chain %CHAIN% --bitcoin-rpc-url %RPC_URL% --bitcoin-rpc-username %RPC_USER% --bitcoin-rpc-password %RPC_PASSWORD% btcc20 inscribe --help
pause
goto menu

:switch_profile
echo.
echo 1. mainnet
echo 2. local regtest
echo.
set /p PROFILE_ACTION=Choose:
if "%PROFILE_ACTION%"=="1" set PROFILE=mainnet
if "%PROFILE_ACTION%"=="2" set PROFILE=local
call :load_profile
if errorlevel 1 pause
goto menu

:run_single
echo.
echo Command to run:
echo %CMD%
echo.
set /p CONFIRM=Type y to execute:
if /i not "%CONFIRM%"=="y" goto menu
echo.
%CMD%
echo.
pause
goto menu

:run_mint_many
echo.
echo Command to run:
echo %CMD%
echo.
echo Mint count: %MINT_COUNT%
echo.
set /p CONFIRM=Type y to execute:
if /i not "%CONFIRM%"=="y" goto menu
echo.
for /l %%I in (1,1,%MINT_COUNT%) do (
  echo [%%I/%MINT_COUNT%] Minting...
  %CMD%
  if errorlevel 1 (
    echo [ERROR] Mint %%I failed. Stopping.
    goto pause_menu
  )
)
echo.
echo Done.
goto pause_menu

:pause_menu
echo.
if "%BTCC20_NO_PAUSE%"=="1" goto menu
pause
goto menu

:load_params
set PARAM_FILE=%~1
set TICK=
set MAX=
set LIM=
set DEC=
set AMT=
set COUNT=
set DESTINATION=
if not exist "%PARAM_FILE%" (
  echo [ERROR] %PARAM_FILE% was not found.
  echo Create it in the same directory as this script.
  exit /b 1
)
for /f "usebackq tokens=1,* delims==" %%A in ("%PARAM_FILE%") do (
  set KEY=%%A
  set VALUE=%%B
  if not "!KEY!"=="" if not "!KEY:~0,1!"=="#" (
    if /i "!KEY!"=="tick" set TICK=!VALUE!
    if /i "!KEY!"=="max" set MAX=!VALUE!
    if /i "!KEY!"=="lim" set LIM=!VALUE!
    if /i "!KEY!"=="dec" set DEC=!VALUE!
    if /i "!KEY!"=="amt" set AMT=!VALUE!
    if /i "!KEY!"=="count" set COUNT=!VALUE!
    if /i "!KEY!"=="destination" set DESTINATION=!VALUE!
  )
)
exit /b 0

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
  echo [ERROR] Profile not found in %CONFIG_FILE%: %PROFILE%
  exit /b 1
)
exit /b 0
