# Windows 双击版说明

这个目录里的文件会被打进 Windows release 压缩包。

用户下载并解压后会看到：

```text
ord.exe
BTCC20-Inscriber.bat
btcc20-profiles.conf
README.md
```

## 第一次使用

1. 双击 `BTCC20-Inscriber.bat`
2. 默认使用 `mainnet` 正式环境配置
3. 按菜单选择 Deploy / Mint / Transfer

如果你的 RPC 密码或钱包名不一样，右键编辑同目录下的 `btcc20-profiles.conf`。

## btcc20-profiles.conf 示例

```ini
[mainnet]
chain=mainnet
rpc_url=http://127.0.0.1:28476
rpc_user=user
rpc_password=pass
wallet=miner

[local]
chain=regtest
rpc_url=http://127.0.0.1:28577
rpc_user=btcc20
rpc_password=btcc20
wallet=btcc20-opensource
```

双击菜单里可以切换 `mainnet` / `local`。命令行也可以这样启动本地测试环境：

```bat
BTCC20-Inscriber.bat --profile local
```

## 使用菜单

菜单支持：

- Deploy 部署
- Mint 铸造
- Transfer 创建转账铭文

执行前会显示即将运行的命令，并要求输入 `y` 确认。

## 注意

这个工具会使用本地 BTCC Core 钱包签名并广播交易。

不要把 `btcc20-profiles.conf` 发给别人，因为里面可能有 RPC 用户名和密码。
