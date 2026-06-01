# Windows double-click release

These files are included in the Windows release archive:

```text
ord.exe
BTCC20-Inscriber.bat
btcc20-profiles.conf
deploy.txt
mint.txt
transfer.txt
README.md
```

## First use

1. Double-click `BTCC20-Inscriber.bat`.
2. Edit `btcc20-profiles.conf` if your RPC username, password, URL, or wallet name is different.
3. Edit `deploy.txt`, `mint.txt`, or `transfer.txt`.
4. Choose Deploy, Mint, or Transfer from the menu.

## Parameter files

Deploy uses `deploy.txt`:

```ini
tick=cord
max=21000000000
lim=1000
dec=18
destination=
```

Mint uses `mint.txt`. `count` controls how many mint inscriptions to create:

```ini
tick=cord
amt=1000
count=1
destination=
```

Transfer uses `transfer.txt`:

```ini
tick=cord
amt=1000
destination=
```

Leave `destination=` empty to let the wallet generate a new owner address.

## Profiles

The default profile is `mainnet`. You can switch to the local regtest profile in the menu, or start the script with:

```bat
BTCC20-Inscriber.bat --profile local
```

## Safety

This tool signs and broadcasts transactions with your local BTCC Core wallet.
Do not share `btcc20-profiles.conf`, because it may contain RPC credentials.
