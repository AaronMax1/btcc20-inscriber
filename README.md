# BTCC20 Inscriber

这是 BTCC-20 铭文工具。普通用户直接下载 Windows 压缩包使用，不需要自己编译源码。

## 下载

打开项目的 GitHub Releases 页面，下载 Windows 压缩包：

```text
btcc20-inscriber-v1.0.0-x86_64-pc-windows-msvc.zip
```

解压后会看到这些文件：

```text
ord.exe
BTCC20-Inscriber.bat
btcc20-profiles.conf
deploy.txt
mint.txt
transfer.txt
README.md
```

## 使用前准备

1. 先运行并同步好自己的 BTCC Core 节点。
2. 确认本地钱包里有可用 BTCC，用来支付 commit/reveal 交易和手续费。
3. 打开 `btcc20-profiles.conf`，把 RPC 地址、用户名、密码、钱包名改成自己的配置。

默认使用 `[mainnet]` 配置。

## 填写参数

根据要做的操作，编辑对应 txt 文件。

部署：

```text
deploy.txt
```

铸造：

```text
mint.txt
```

转移铭文：

```text
transfer.txt
```

`mint.txt` 里的 `count=` 表示连续 mint 几次。

`destination=` 是铭文归属地址。不填时，程序会自动从钱包生成新地址。

## 开始使用

双击：

```text
BTCC20-Inscriber.bat
```

然后按菜单选择：

```text
1. Deploy
2. Mint
3. Transfer
```

脚本会自动读取当前文件夹里的 txt 参数并执行。

## 注意

不要把 `btcc20-profiles.conf` 里的 RPC 用户名和密码发给别人。

不要把 BTCC Core RPC 暴露到公网。
