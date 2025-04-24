# Raydium CP-Swap (CPMM) 漏洞赏金计划

Raydium在Immunefi平台的完整漏洞赏金计划详见：https://immunefi.com/bounty/raydium/

## 威胁等级对应奖励

根据Immunefi漏洞严重程度分类系统V2.3，赏金将按漏洞影响程度分配。该评估采用简化的五级标准，重点关注所报告漏洞的实际影响。

### 智能合约漏洞

| 严重程度 | 赏金金额                  |
| -------- | ------------------------- |
| 严重     | 50,000 至 505,000 美元    |
| 高危     | 40,000 美元               |
| 中危     | 5,000 美元                |

所有漏洞报告必须包含概念验证(PoC)，证明该漏洞如何影响范围内的资产才能获得奖励。严重和高危漏洞报告还应包含修复建议。纯文字说明不被视为有效PoC，必须提供可验证代码。

若发现的智能合约严重漏洞被利用，其赏金上限为受威胁直接资金的10%，但最低保证50,000美元奖励。

`raydium-sdk`及其他非智能合约代码的漏洞将根据具体情况评估。

## 报告提交方式

请发送邮件至security@reactorlabs.io，详细描述攻击向量。高危和严重级别的报告需包含概念验证。我们将在24小时内回复后续处理流程。

## 奖励支付

奖励由Raydium团队直接处理，以美元计价，可通过RAY、SOL或USDC支付。

## 排除范围及规则

以下漏洞类型不在本赏金计划范围内：
- 报告者已实施并造成实际损害的漏洞利用
- 需要获取泄露密钥/凭证的攻击
- 需要特权地址(治理/策略)权限的攻击
- 第三方预言机数据错误（不包括预言机操纵/闪电贷攻击）
- 基础经济治理攻击（如51%攻击）
- 流动性不足问题
- 最佳实践建议
- 女巫攻击
- 中心化风险
- 任何前端界面问题
- Solana核心运行时的漏洞（请提交至[Solana漏洞赏金计划](https://github.com/solana-labs/solana/security/policy)）
- 需要验证节点执行的漏洞
- 需要特权密钥的攻击
- 团队已知的MEV攻击向量

## 在范围内的AMM资产

| 目标文件                                                                                                                 | 类型                                  |
| ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/lib.rs                                     | 智能合约 - 主库文件                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/error.rs                                   | 智能合约 - 错误处理                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/collect_fund_fee.rs     | 智能合约 - 资金费收取                 |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/collect_protocol_fee.rs | 智能合约 - 协议费收取                 |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/create_config.rs        | 智能合约 - 配置创建                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/mod.rs                  | 智能合约 - 管理模块                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/update_config.rs        | 智能合约 - 配置更新                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/admin/update_pool_status.rs   | 智能合约 - 资金池状态更新             |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/deposit.rs                    | 智能合约 - 存款功能                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/initialize.rs                 | 智能合约 - 初始化功能                 |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/mod.rs                        | 智能合约 - 指令模块                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/swap_base_input.rs            | 智能合约 - 基础输入交换               |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/swap_base_output.rs           | 智能合约 - 基础输出交换               |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/instructions/withdraw.rs                   | 智能合约 - 取款功能                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/states/config.rs                           | 智能合约 - 配置状态                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/states/events.rs                           | 智能合约 - 事件状态                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/states/mod.rs                              | 智能合约 - 状态模块                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/states/pool.rs                             | 智能合约 - 资金池状态                 |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/utils/math.rs                              | 智能合约 - 数学工具                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/utils/mod.rs                               | 智能合约 - 工具模块                   |
| https://github.com/raydium-io/raydium-cp-swap/blob/master/programs/cp-swap/src/utils/token.rs                             | 智能合约 - 代币工具                   |

## 补充信息

Raydium CPMM公开测试网地址：
https://explorer.solana.com/address/CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW?cluster=devnet

OpenBook中央限价订单簿测试网地址：
https://explorer.solana.com/address/EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj

若发现任何未列在上述表格中但符合下文"影响范围"的Raydium管理资产存在严重漏洞，欢迎提交报告。此条款仅适用于严重级别漏洞。
