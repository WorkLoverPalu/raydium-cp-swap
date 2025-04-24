# raydium-cp-swap（恒定乘积交易协议）

这是一个经过重构的恒定乘积自动化做市商（AMM）程序，专为简化资金池部署而优化，并提供以下增强功能和集成支持：
- 创建资金池无需提供Openbook市场ID
- 支持Token22代币标准
- 内置价格预言机
- 基于Anchor框架进行优化

本程序已通过[MadShield](https://www.madshield.xyz/)安全审计，审计报告详见[此处](https://github.com/raydium-io/raydium-docs/tree/master/audit/MadShield%20Q1%202024)。

该项目资产适用于Raydium在[Immunefi](https://immunefi.com/bug-bounty/raydium/)平台推出的漏洞赏金计划。

## 环境配置

1. 安装Rust语言环境
2. 安装Solana工具链后，执行`solana-keygen new`命令在默认路径创建密钥对
3. 安装Anchor开发框架

## 快速开始

克隆代码库并测试程序：
```shell
git clone https://github.com/raydium-io/raydium-cp-swap
cd raydium-cp-swap && anchor test
```

## 许可证

Raydium恒定乘积交易协议采用Apache许可证2.0版本授权。
