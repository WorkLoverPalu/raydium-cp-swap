# raydium-cp-swap（恒定乘积交易协议）

这是一个经过重构的恒定乘积自动化做市商（AMM）程序，专为简化资金池部署而优化，并提供以下增强功能和集成支持：
- 创建资金池无需提供Openbook市场ID
- 支持Token22代币标准
- 内置价格预言机
- 基于Anchor框架进行优化

本程序已通过[MadShield](https://www.madshield.xyz/)安全审计，审计报告详见[此处](https://github.com/raydium-io/raydium-docs/tree/master/audit/MadShield%20Q1%202024)。

该项目资产适用于Raydium在[Immunefi](https://immunefi.com/bug-bounty/raydium/)平台推出的漏洞赏金计划。

## 环境配置

1. Install `Rust`

   ```shell
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default 1.81.0
   ```

2. Install `Solana `

   ```shell
   sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.0/install)"
   ```

   then run `solana-keygen new` to create a keypair at the default location.

3. install `Anchor`

   ```shell
   # Installing using Anchor version manager (avm) 
   cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
   # Install anchor
   avm install 0.31.0
   ```

## 快速开始

克隆代码库并测试程序：
```shell
git clone https://github.com/raydium-io/raydium-cp-swap
cd raydium-cp-swap && yarn && anchor test
```

## 许可证

Raydium恒定乘积交易协议采用Apache许可证2.0版本授权。
