# Raydium CP-Swap 交易日志解析器文档

## 功能概述
该代码实现了一个用于解析Raydium CP-Swap智能合约交易日志和指令的工具，主要功能包括：
- 解析程序执行日志中的事件数据
- 解码交易中的指令数据
- 支持多种编码格式(Base58/Base64/Hex)
- 可视化显示交易结构和指令参数

## 核心模块

### 1. 日志解析模块

#### `parse_program_event`
```rust
pub fn parse_program_event(
    self_program_str: &str,  // 当前程序ID
    meta: Option<UiTransactionStatusMeta>  // 交易元数据
) -> Result<(), ClientError>
```
**功能**：解析程序执行日志中的事件数据

**处理流程**：
1. 从交易元数据中提取日志信息
2. 创建执行上下文栈(`Execution`结构)
3. 逐行处理日志：
   - 识别程序调用边界(`invoke`/`success`)
   - 解析程序自定义事件(SwapEvent/LpChangeEvent)

#### `Execution`结构体
```rust
struct Execution {
    stack: Vec<String>,  // 调用栈
}
```
- 维护程序调用层级关系
- 提供`push`/`pop`方法管理调用栈

#### `handle_program_log`
```rust
fn handle_program_log(...) -> Result<(Option<String>, bool), ClientError>
```
**功能**：处理单条程序日志

**处理逻辑**：
1. 过滤系统日志前缀
2. Base64解码事件数据
3. 通过discriminator识别事件类型
4. 使用Borsh反序列化事件数据

### 2. 指令解析模块

#### `parse_program_instruction`
```rust
pub fn parse_program_instruction(
    self_program_str: &str,
    encoded_transaction: EncodedTransaction,
    meta: Option<UiTransactionStatusMeta>
) -> Result<(), ClientError>
```
**功能**：解析交易中的指令数据

**处理流程**：
1. 解码交易原始消息
2. 合并地址查找表数据
3. 定位当前程序的指令位置
4. 解析主指令和内部指令

#### `handle_program_instruction`
```rust
pub fn handle_program_instruction(
    instr_data: &str,
    decode_type: InstructionDecodeType
) -> Result<(), ClientError>
```
**功能**：解码并显示单条指令

**支持指令类型**：
| 指令名 | 描述 | 关键参数 |
|--------|------|----------|
| CreateAmmConfig | 创建AMM配置 | 费率参数 |
| UpdateAmmConfig | 更新AMM配置 | 参数索引/值 |
| Initialize | 初始化资金池 | 初始流动性量 |
| Deposit | 存入流动性 | LP代币数量 |
| Withdraw | 提取流动性 | 最小提取量 |
| SwapBaseInput | 指定输入交换 | 输入量/最小输出 |
| SwapBaseOutput | 指定输出交换 | 最大输入/输出量 |

### 3. 辅助功能

#### 编码支持
```rust
pub enum InstructionDecodeType {
    BaseHex,
    Base64,
    Base58,
}
```
- 支持三种常见编码格式
- 自动检测并解码指令数据

#### 可视化输出
- 使用`colorful`库着色输出
- 结构化显示指令参数
- 区分主指令和内部指令

## 使用示例

### 解析交易日志
```rust
let logs = vec![
    "Program log: Base64EncodedEventData".to_string(),
    "Program xxx invoke [2]".to_string(),
    "Program log: AnotherEvent".to_string(),
    "Program xxx success".to_string()
];

parse_program_event("xxx", Some(meta))?;
```

### 解析交易指令
```rust
let tx = get_transaction(); // 获取交易数据
parse_program_instruction("raydium_cp_swap", tx, meta)?;
```

## 输出示例

### 事件输出
```text
SwapEvent {
    amount_in: 1000000,
    amount_out: 995000,
    fee: 5000
}
```

### 指令输出
```text
instruction #1 (绿色)
Deposit {
    lp_token_amount: 5000000,
    maximum_token_0_amount: 10000000,
    maximum_token_1_amount: 20000000
}
```

## 设计要点

1. **调用栈管理**：正确处理跨程序调用(CPI)的日志边界
2. **错误恢复**：跳过无法解析的日志而不中断流程
3. **扩展性**：通过discriminator机制轻松添加新事件类型
4. **可视化**：彩色输出增强可读性

该工具非常适合用于：
- 链上交易分析
- 智能合约调试
- 交易监控系统集成