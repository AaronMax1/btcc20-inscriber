# BTCC-20 铭文工具

这是一个本地 BTCC-20 铭文工具，基于 `ord` 修改而来。

用户在自己的电脑上运行它，连接自己的 BTCC Core 节点和钱包，用自己的钱包签名并广播 commit/reveal 交易。Explorer 浏览器只负责展示和生成命令，不应该替用户签名或广播。

## 支持功能

- 部署 BTCC-20：`deploy`
- 铸造 BTCC-20：`mint`
- 创建转账铭文：`transfer`
- 解码 BTCC-20 铭文：`decode`

BTCC-20 使用 Ordinals 铭文信封，协议字段是：

```json
{"p":"btcc-20"}
```

## 普通用户怎么用

如果你不会编译代码，等项目发布 Release 后，下载对应系统的压缩包。

Windows 用户下载类似这样的文件：

```text
btcc20-inscriber-版本号-x86_64-pc-windows-msvc.zip
```

解压后会看到：

```text
ord.exe
BTCC20-Inscriber.bat
btcc20-profiles.conf
README.md
```

使用步骤：

1. 双击 `BTCC20-Inscriber.bat`
2. 默认使用 `mainnet` 正式环境配置
3. 按菜单选择 deploy / mint / transfer

如果你的 BTCC Core RPC 密码或钱包名不一样，编辑同目录下的 `btcc20-profiles.conf`。

注意：你的 BTCC Core 钱包里必须有可用 BTCC，用来支付 commit/reveal 交易和手续费。

## 配置文件

RPC 和钱包配置统一放在 `btcc20-profiles.conf`：

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

默认 profile 是 `mainnet`，也就是正式环境。这个配置指向你本机运行的 BTCC Core 节点，不是公开远程 RPC。

本地开发或测试使用 `local`：

```sh
./btcc20-inscriber --profile local inscribe mint --tick cord --amt 1000
```

也可以用环境变量临时覆盖配置文件：

```sh
BTCC20_RPC_PASSWORD=你的密码 ./btcc20-inscriber inscribe mint --tick cord --amt 1000
```

## 开发者怎么编译

安装 Rust 后，在仓库根目录执行：

```sh
cargo build --bin ord
```

查看 BTCC-20 命令：

```sh
./target/debug/ord btcc20 --help
```

也可以使用 wrapper：

```sh
./btcc20-inscriber inscribe --help
```

wrapper 默认读取 `btcc20-profiles.conf` 的 `mainnet` profile，不需要每次手动传 RPC 参数。

本地 regtest 测试：

```sh
./btcc20-inscriber --profile local inscribe --help
```

环境变量仍然可以覆盖配置文件：

```sh
BTCC20_RPC_PASSWORD=PASS \
./btcc20-inscriber inscribe --wallet miner mint --tick cord --amt 1000
```

## RPC 参数

所有 RPC 参数都放在 `btcc20` 前面：

```sh
./target/debug/ord \
  --chain mainnet \
  --bitcoin-rpc-url http://127.0.0.1:28476 \
  --bitcoin-rpc-username USER \
  --bitcoin-rpc-password PASS \
  btcc20 inscribe --wallet miner mint --tick cord --amt 1000
```

Regtest 示例：

```sh
./target/debug/ord \
  --chain regtest \
  --bitcoin-rpc-url http://127.0.0.1:28577 \
  --bitcoin-rpc-username btcc20 \
  --bitcoin-rpc-password btcc20 \
  btcc20 inscribe --wallet btcc20-opensource mint --tick cord --amt 1000
```

## 部署 Deploy

```sh
./btcc20-inscriber inscribe \
  deploy \
  --tick cord \
  --max 21000000000 \
  --lim 1000
```

这个示例表示 `2100 万张，每张 1000，总量 210 亿`。

`dec` 是可选字段，不填时由索引器按协议默认值处理：

```sh
./btcc20-inscriber inscribe \
  deploy \
  --tick cord \
  --max 21000000000 \
  --lim 1000 \
  --dec 18
```

## 铸造 Mint

```sh
./btcc20-inscriber inscribe \
  mint \
  --tick cord \
  --amt 1000
```

## 连续铸造 Mint

如果要连续 mint 多张，可以用脚本：

```sh
./btcc20-mint-many --tick cord --amt 1000 --count 10
```

默认每 mint 一张会等待 reveal 交易 1 个确认，再继续下一张。这是最稳的方式。

本地测试：

```sh
./btcc20-mint-many --profile local --tick cord --amt 1000 --count 3
```

如果想连续广播，不等待确认：

```sh
./btcc20-mint-many --tick cord --amt 1000 --count 20 --no-wait
```

一直 mint，直到手动 `Ctrl+C`：

```sh
./btcc20-mint-many --tick cord --amt 1000 --forever
```

指定归属地址：

```sh
./btcc20-mint-many --tick cord --amt 1000 --count 10 --destination cc1...
```

## 转账铭文 Transfer

```sh
./btcc20-inscriber inscribe \
  transfer \
  --tick cord \
  --amt 250
```

这一步只是创建“转账铭文”。只有这张转账铭文所在的 UTXO 被花到接收方地址后，索引器才会把它识别为真正转账完成。

## 指定接收地址

如果不传 `--destination`，钱包会自动生成一个新地址作为铭文归属地址。

指定接收地址：

```sh
./btcc20-inscriber inscribe \
  mint \
  --tick cord \
  --amt 1000 \
  --destination cc1...
```

## Payload 示例

Deploy：

```json
{"p":"btcc-20","op":"deploy","tick":"cord","max":"21000000000","lim":"1000"}
```

Mint：

```json
{"p":"btcc-20","op":"mint","tick":"cord","amt":"1000"}
```

Transfer：

```json
{"p":"btcc-20","op":"transfer","tick":"cord","amt":"250"}
```

## 安全提醒

这个工具会使用你的本地钱包签名并广播交易。

不要把 RPC 暴露到公网。不要把 RPC 用户名和密码发给别人。不要用装有大量资金的钱包做测试。
