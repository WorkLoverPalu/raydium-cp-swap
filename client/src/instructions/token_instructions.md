# Solana Token 操作工具库文档

## 功能概述

该工具库提供了一套完整的 Solana Token 操作功能，包括：
- Token 账户创建与初始化
- 关联 Token 账户(ATA)管理
- Token 转账与铸造
- SOL 包装与解包
- 账户关闭操作

支持标准 SPL Token 和 Token-2022 两种代币程序。

## 核心 API

### 1. Token 铸造账户管理

#### `create_and_init_mint_instr`

```rust
pub fn create_and_init_mint_instr(
    config: &ClientConfig,
    token_program: Pubkey,  // spl_token 或 spl_token_2022 程序ID
    mint_key: &Pubkey,      // 新铸造账户地址
    mint_authority: &Pubkey, // 铸造权限地址
    freeze_authority: Option<&Pubkey>, // 冻结权限地址(可选)
    extension_init_params: Vec<ExtensionInitializationParams>, // 扩展参数(Token-2022)
    decimals: u8           // 代币精度
) -> Result<Vec<Instruction>>
```

**功能**：创建并初始化代币铸造账户

**特点**：
- 自动计算账户空间（支持扩展）
- 支持 Token-2022 扩展功能
- 返回完整的初始化指令序列

**使用示例**：
```rust
let instrs = create_and_init_mint_instr(
    &config,
    spl_token_2022::id(),
    &mint_pubkey,
    &auth_pubkey,
    None,
    vec![ExtensionInitializationParams::TransferFeeConfig {
        transfer_fee_config_authority: Some(auth_pubkey),
        withdraw_withheld_authority: Some(auth_pubkey),
        withheld_amount: 0,
    }],
    9 // 9位小数
)?;
```

### 2. 账户创建

#### `create_account_rent_exmpt_instr`

```rust
pub fn create_account_rent_exmpt_instr(
    config: &ClientConfig,
    new_account_key: &Pubkey, // 新账户地址
    owner: Pubkey,           // 账户所有者程序ID
    data_size: usize        // 账户数据大小
) -> Result<Vec<Instruction>>
```

**功能**：创建免租金的通用账户

**典型用途**：
- 创建 PDA 账户
- 初始化程序状态账户

#### `create_ata_token_account_instr`

```rust
pub fn create_ata_token_account_instr(
    config: &ClientConfig,
    token_program: Pubkey,  // 代币程序ID
    mint: &Pubkey,          // 代币铸造地址
    owner: &Pubkey          // Token 账户所有者
) -> Result<Vec<Instruction>>
```

**功能**：创建关联 Token 账户(ATA)

**特点**：
- 幂等操作（已存在时不报错）
- 自动计算租金豁免金额

### 3. Token 账户操作

#### `create_and_init_auxiliary_token`

```rust
pub fn create_and_init_auxiliary_token(
    config: &ClientConfig,
    new_account_key: &Pubkey, // 新 Token 账户地址
    mint: &Pubkey,           // 代币铸造地址
    owner: &Pubkey           // Token 账户所有者
) -> Result<Vec<Instruction>>
```

**功能**：创建并初始化辅助 Token 账户

**特点**：
- 自动检测代币程序版本
- 支持 ImmutableOwner 扩展
- 兼容 Token-2022 的必需扩展

#### `close_token_account`

```rust
pub fn close_token_account(
    config: &ClientConfig,
    close_account: &Pubkey,  // 要关闭的账户
    destination: &Pubkey,    // 接收剩余SOL的账户
    owner: &Keypair          // 账户所有者密钥
) -> Result<Vec<Instruction>>
```

**功能**：关闭 Token 账户并回收租金

**注意事项**：
- 账户余额必须为0
- 需要所有者签名

### 4. Token 转账与铸造

#### `spl_token_transfer_instr`

```rust
pub fn spl_token_transfer_instr(
    config: &ClientConfig,
    from: &Pubkey,          // 来源 Token 账户
    to: &Pubkey,            // 目标 Token 账户
    amount: u64,            // 转账金额
    from_authority: &Keypair // 转账权限密钥
) -> Result<Vec<Instruction>>
```

**功能**：标准 Token 转账

#### `spl_token_mint_to_instr`

```rust
pub fn spl_token_mint_to_instr(
    config: &ClientConfig,
    token_program: Pubkey,  // 代币程序ID
    mint: &Pubkey,          // 代币铸造地址
    to: &Pubkey,            // 目标账户
    amount: u64,            // 铸造数量
    mint_authority: &Keypair // 铸造权限密钥
) -> Result<Vec<Instruction>>
```

**功能**：铸造代币到指定账户

**特点**：
- 支持标准 Token 和 Token-2022
- 需要铸造权限签名

### 5. SOL 包装

#### `wrap_sol_instr`

```rust
pub fn wrap_sol_instr(
    config: &ClientConfig,
    amount: u64            // 要包装的SOL数量
) -> Result<Vec<Instruction>>
```

**功能**：将 SOL 包装为 wSOL

**执行步骤**：
1. 创建 wSOL 关联账户（如不存在）
2. 转账 SOL 到 wSOL 账户
3. 同步 wSOL 余额

## 设计特点

1. **多版本支持**
   - 自动区分 spl_token 和 spl_token_2022
   - 兼容处理扩展功能

2. **完整指令序列**
   - 每个函数返回完整的操作指令集
   - 方便组合到交易中

3. **安全考虑**
   - 显式要求必要的签名密钥
   - 自动计算租金豁免

4. **易用性**
   - 统一使用 ClientConfig 配置
   - 简洁的 API 设计

## 使用示例

### 创建代币并铸造

```rust
// 1. 创建铸造账户
let mint_key = Keypair::new();
let create_instrs = create_and_init_mint_instr(
    &config,
    spl_token_2022::id(),
    &mint_key.pubkey(),
    &authority.pubkey(),
    None,
    vec![],
    6
)?;

// 2. 创建关联账户
let ata_instrs = create_ata_token_account_instr(
    &config,
    spl_token_2022::id(),
    &mint_key.pubkey(),
    &user.pubkey()
)?;

// 3. 铸造代币
let mint_to_instrs = spl_token_mint_to_instr(
    &config,
    spl_token_2022::id(),
    &mint_key.pubkey(),
    &user_ata,
    100_000_000, // 100个代币
    &authority
)?;

// 组合并发送交易
let mut tx = Transaction::new_with_payer(
    &[create_instrs, ata_instrs, mint_to_instrs].concat(),
    Some(&config.payer)
);
tx.sign(&[&config.payer, &mint_key, &authority], recent_blockhash);
let sig = send_txn(&rpc_client, &tx, true)?;
```

### 包装 SOL 使用

```rust
let wrap_instrs = wrap_sol_instr(&config, 1_000_000_000)?; // 包装1 SOL
let tx = Transaction::new_with_payer(&wrap_instrs, Some(&config.payer));
let sig = send_txn(&rpc_client, &tx, true)?;
```