
// 与 Raydium 恒定乘积(CP)自动做市商(AMM)智能合约交互的客户端实现，主要包含四个核心功能：初始化资金池、存款、取款和交换代币。


// PDA(Program Derived Address): 使用种子和程序ID派生的特殊地址，确保只有特定程序可以签名
// Token Vaults: 池中存储代币的账户
// LP Token: 流动性提供者代币，代表池中的份额
// Observation State: 用于记录价格历史数据的账户

// anchor_client 用于与Anchor程序交互的客户端库
use anchor_client::{Client, Cluster};
// 错误处理工具
use anyhow::Result;
// 基础SDK，包含指令、公钥等核心类型
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar};

// 引入Raydium程序自动生成的账户结构和指令结构
use raydium_cp_swap::accounts as raydium_cp_accounts;
use raydium_cp_swap::instruction as raydium_cp_instructions;

// 引入程序中定义的常量种子(seeds)，用于PDA派生
use raydium_cp_swap::{
    states::{AMM_CONFIG_SEED, OBSERVATION_SEED, POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
    AUTH_SEED,
};
use std::rc::Rc;

use super::super::{read_keypair_file, ClientConfig};

// 初始化资金池
pub fn initialize_pool_instr(
    config: &ClientConfig,        //客户配置
    token_0_mint: Pubkey,         // 代币0的mint地址
    token_1_mint: Pubkey,         // 代币1的mint地址
    token_0_program: Pubkey,      // 代币0的程序地址
    token_1_program: Pubkey,      // 代币1的程序地址
    user_token_0_account: Pubkey, // 用户的代币0账户地址
    user_token_1_account: Pubkey, // 用户的代币1账户地址
    create_pool_fee: Pubkey,      // 创建池的费用地址
    init_amount_0: u64,           // 初始化代币0的数量
    init_amount_1: u64,           // 初始化代币1的数量
    open_time: u64,               // 开放时间
) -> Result<Vec<Instruction>> {
    // 读取支付者的密钥对
    let payer = read_keypair_file(&config.payer_path)?;

    // 创建自定义集群
    // 这里使用的是自定义的HTTP和WebSocket URL
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());

    // 创建客户端
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_cp_program)?;

    // 计算程序派生（PDA）地址
    // 这里使用了find_program_address函数来计算PDA
    // 该函数接受一个字节数组和程序ID作为参数
    // 这里的种子是AMM_CONFIG_SEED和一个u16类型的索引
    // 该索引在这里是0
    // 该函数返回一个元组，第一个元素是计算出的地址，第二个元素是一个随机数
    let amm_config_index = 0u16;
    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[AMM_CONFIG_SEED.as_bytes(), &amm_config_index.to_be_bytes()],
        &program.id(),
    );

    // 计算其他相关的PDA（程序派生）地址：Pool 地址
    let (pool_account_key, __bump) = Pubkey::find_program_address(
        &[
            POOL_SEED.as_bytes(),
            amm_config_key.to_bytes().as_ref(),
            token_0_mint.to_bytes().as_ref(),
            token_1_mint.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    // 计算其他相关的PDA（程序派生）地址：配置账户地址
    let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());

    // 代币0地址
    let (token_0_vault, __bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_0_mint.to_bytes().as_ref(),
        ],
        &program.id(),
    );

    // 代币1地址
    let (token_1_vault, __bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_1_mint.to_bytes().as_ref(),
        ],
        &program.id(),
    );

    // 计算LP mint地址
    let (lp_mint_key, __bump) = Pubkey::find_program_address(
        &[
            POOL_LP_MINT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
        ],
        &program.id(),
    );

    // 计算观察状态地址
    // 该地址用于存储观察状态
    let (observation_key, __bump) = Pubkey::find_program_address(
        &[
            OBSERVATION_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
        ],
        &program.id(),
    );

    // 创建指令
    // 这里使用了request函数来创建一个请求
    // 该请求包含了多个账户的地址和相关参数
    // 这些账户包括创建者、池状态、代币0和代币1的mint地址、LP mint地址等
    // 还包括代币0和代币1的vault地址、创建池的费用地址等
    // 还包括观察状态地址、代币程序地址、系统程序地址等
    let instructions = program
        .request()
        .accounts(raydium_cp_accounts::Initialize {
            creator: program.payer(),   // 创建者，支付交易费用的账户
            amm_config: amm_config_key, //amm 配置账户
            authority,                  //pool 的管理权限PDA
            pool_state: pool_account_key,
            token_0_mint,
            token_1_mint,
            lp_mint: lp_mint_key,
            creator_token_0: user_token_0_account,
            creator_token_1: user_token_1_account,
            creator_lp_token: spl_associated_token_account::get_associated_token_address(
                &program.payer(),
                &lp_mint_key,
            ),
            token_0_vault,
            token_1_vault,
            create_pool_fee,
            observation_state: observation_key, //价格观测账户
            token_program: spl_token::id(),
            token_0_program,
            token_1_program,
            associated_token_program: spl_associated_token_account::id(),
            system_program: system_program::id(), //必须的系统程序
            rent: sysvar::rent::id(),             //租金系统变量
        })
        .args(raydium_cp_instructions::Initialize {
            init_amount_0,
            init_amount_1,
            open_time,
        })
        .instructions()?;
    Ok(instructions)
}

// 存款操作
pub fn deposit_instr(
    config: &ClientConfig, // 客户配置
    pool_id: Pubkey,       // 资金池ID
    token_0_mint: Pubkey,  // 代币0的mint地址
    token_1_mint: Pubkey,  // 代币1的mint地址
    token_lp_mint: Pubkey, // LP mint地址
    token_0_vault: Pubkey, //
    token_1_vault: Pubkey, //
    user_token_0_account: Pubkey,
    user_token_1_account: Pubkey,
    user_token_lp_account: Pubkey,
    lp_token_amount: u64,        // 期望获得的LP代币数量
    maximum_token_0_amount: u64, // 代币A的最大存入量
    maximum_token_1_amount: u64, // 代币B的最大存入量
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_cp_program)?;

    let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());

    let instructions = program
        .request()
        .accounts(raydium_cp_accounts::Deposit {
            owner: program.payer(),
            authority,
            pool_state: pool_id,
            owner_lp_token: user_token_lp_account,
            token_0_account: user_token_0_account,
            token_1_account: user_token_1_account,
            token_0_vault,
            token_1_vault,
            token_program: spl_token::id(),
            token_program_2022: spl_token_2022::id(),
            vault_0_mint: token_0_mint,
            vault_1_mint: token_1_mint,
            lp_mint: token_lp_mint,
        })
        .args(raydium_cp_instructions::Deposit {
            lp_token_amount,        // 期望获得的LP代币数量
            maximum_token_0_amount, // 代币A的最大存入量
            maximum_token_1_amount, // 代币B的最大存入量
        })
        .instructions()?;
    Ok(instructions)
}

// 取款操作
pub fn withdraw_instr(
    config: &ClientConfig,
    pool_id: Pubkey,
    token_0_mint: Pubkey,
    token_1_mint: Pubkey,
    token_lp_mint: Pubkey,
    token_0_vault: Pubkey,
    token_1_vault: Pubkey,
    user_token_0_account: Pubkey,
    user_token_1_account: Pubkey,
    user_token_lp_account: Pubkey,
    lp_token_amount: u64,
    minimum_token_0_amount: u64,
    minimum_token_1_amount: u64,
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_cp_program)?;

    let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());

    let instructions = program
        .request()
        .accounts(raydium_cp_accounts::Withdraw {
            owner: program.payer(),
            authority,
            pool_state: pool_id,
            owner_lp_token: user_token_lp_account,
            token_0_account: user_token_0_account,
            token_1_account: user_token_1_account,
            token_0_vault,
            token_1_vault,
            token_program: spl_token::id(),
            token_program_2022: spl_token_2022::id(), // 2022版本的token程序
            vault_0_mint: token_0_mint,
            vault_1_mint: token_1_mint,
            lp_mint: token_lp_mint,
            memo_program: spl_memo::id(),
        })
        .args(raydium_cp_instructions::Withdraw {
            lp_token_amount,
            minimum_token_0_amount,
            minimum_token_1_amount,
        })
        .instructions()?;
    Ok(instructions)
}

// 交换操作
pub fn swap_base_input_instr(
    config: &ClientConfig,
    pool_id: Pubkey,
    amm_config: Pubkey,
    observation_account: Pubkey,
    input_token_account: Pubkey,
    output_token_account: Pubkey,
    input_vault: Pubkey,
    output_vault: Pubkey,
    input_token_mint: Pubkey,
    output_token_mint: Pubkey,
    input_token_program: Pubkey,
    output_token_program: Pubkey,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_cp_program)?;

    let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());

    let instructions = program
        .request()
        .accounts(raydium_cp_accounts::Swap {
            payer: program.payer(),
            authority,
            amm_config,
            pool_state: pool_id,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state: observation_account,
        })
        .args(raydium_cp_instructions::SwapBaseInput {
            amount_in,          // 精确输入量
            minimum_amount_out, // 可接受的最小输出
        })
        .instructions()?;
    Ok(instructions)
}

// 交换操作
pub fn swap_base_output_instr(
    config: &ClientConfig,
    pool_id: Pubkey,
    amm_config: Pubkey,
    observation_account: Pubkey,
    input_token_account: Pubkey,
    output_token_account: Pubkey,
    input_vault: Pubkey,
    output_vault: Pubkey,
    input_token_mint: Pubkey,
    output_token_mint: Pubkey,
    input_token_program: Pubkey,
    output_token_program: Pubkey,
    max_amount_in: u64,
    amount_out: u64,
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_cp_program)?;

    let (authority, __bump) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program.id());

    let instructions = program
        .request()
        .accounts(raydium_cp_accounts::Swap {
            payer: program.payer(),
            authority,
            amm_config,
            pool_state: pool_id,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state: observation_account,
        })
        .args(raydium_cp_instructions::SwapBaseOutput {
            max_amount_in,
            amount_out,
        })
        .instructions()?;
    Ok(instructions)
}
