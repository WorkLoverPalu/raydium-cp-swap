# Solana 交易工具库文档

## 功能概述

该代码实现了一个 Solana 交易辅助工具库，提供以下核心功能：
- 交易模拟执行
- 交易发送与确认
- 账户数据查询
- 批量账户查询

## API 参考

### 1. 交易模拟执行

#### `simulate_transaction`

```rust
pub fn simulate_transaction(
    client: &RpcClient,               // RPC客户端实例
    transaction: &Transaction,        // 待模拟交易
    sig_verify: bool,                 // 是否验证签名
    cfg: CommitmentConfig             // 确认级别配置
) -> RpcResult<RpcSimulateTransactionResult>
```

**功能**：在不上链的情况下模拟执行交易

**参数说明**：
- `sig_verify`: 
  - `true`: 验证交易签名有效性
  - `false`: 跳过签名验证
- `cfg`: 确认级别枚举值，常用选项：
  - `processed`: 节点已接收
  - `confirmed`: 集群已确认
  - `finalized`: 不可逆确认

**返回值**：
包含模拟执行结果的RPC响应，其中：
- `logs`: 程序执行日志
- `accounts`: 账户状态变化
- `units_consumed`: 计算单元消耗量

**使用示例**：
```rust
let result = simulate_transaction(
    &rpc_client,
    &txn,
    true, 
    CommitmentConfig::confirmed()
)?;
```

### 2. 交易发送

#### `send_txn`

```rust
pub fn send_txn(
    client: &RpcClient,       // RPC客户端实例
    txn: &Transaction,        // 待发送交易
    wait_confirm: bool        // 是否等待确认
) -> Result<Signature>        // 返回交易签名
```

**功能**：发送交易并可选等待确认

**参数说明**：
- `wait_confirm`:
  - `true`: 等待交易确认（使用confirmed级别）
  - `false`: 异步发送（使用processed级别）

**配置特性**：
- 启用进度条显示(`with_spinner`)
- 跳过预执行检查(`skip_preflight=true`)

**使用示例**：
```rust
// 同步发送
let sig = send_txn(&rpc_client, &txn, true)?;

// 异步发送 
let sig = send_txn(&rpc_client, &txn, false)?;
```

### 3. 账户数据查询

#### `get_token_account`

```rust
pub fn get_token_account<T: TokenPack>(
    client: &RpcClient,   // RPC客户端实例
    addr: &Pubkey         // 账户地址
) -> Result<T>            // 返回解析后的账户数据
```

**功能**：查询并解析Token账户数据

**泛型约束**：
`T` 需实现 `TokenPack` trait，典型类型：
- `spl_token::state::Account`
- `spl_token::state::Mint`

**错误处理**：
- 账户不存在时返回自定义错误
- 数据解析失败时返回标准错误

**使用示例**：
```rust
let token_account: spl_token::state::Account = 
    get_token_account(&rpc_client, &token_account_address)?;
```

#### `get_multiple_accounts`

```rust
pub fn get_multiple_accounts(
    client: &RpcClient,       // RPC客户端实例
    pubkeys: &[Pubkey]        // 账户地址列表
) -> Result<Vec<Option<Account>>>  // 返回账户数据列表
```

**功能**：批量查询账户原始数据

**返回说明**：
- `Vec`顺序与输入地址顺序一致
- `Option`表示账户是否存在

**性能建议**：
- 单次请求最多支持100个地址
- 大数据集建议分批次查询

## 设计特点

1. **错误处理统一化**
   - 使用`anyhow`提供上下文错误信息
   - 自定义账户不存在错误

2. **配置灵活性**
   - 支持不同确认级别
   - 可调式签名验证

3. **性能优化**
   - 批量查询接口
   - 异步发送选项

4. **类型安全**
   - 泛型约束确保数据正确解析
   - 明确的返回类型签名

## 典型使用场景

### 交易预检流程
```rust
// 1. 构建交易
let txn = build_transaction(...);

// 2. 模拟执行
let sim_result = simulate_transaction(
    &rpc_client,
    &txn,
    true,
    CommitmentConfig::confirmed()
)?;

// 3. 检查执行结果
if let Some(err) = sim_result.value.err {
    eprintln!("Simulation failed: {}", err);
    return Err(anyhow!("Transaction would fail"));
}

// 4. 正式发送
let sig = send_txn(&rpc_client, &txn, true)?;
```

### 批量检查Token余额
```rust
let addresses = vec![addr1, addr2, addr3];
let accounts = get_multiple_accounts(&rpc_client, &addresses)?;

for (i, account) in accounts.iter().enumerate() {
    if let Some(acc) = account {
        let token_acc: spl_token::state::Account = 
            spl_token::state::Account::unpack(&acc.data)?;
        println!("Account {} balance: {}", i, token_acc.amount);
    }
}
```