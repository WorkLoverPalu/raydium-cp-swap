//！互换计算

use crate::curve::{constant_product::ConstantProductCurve, fees::Fees};
use anchor_lang::prelude::*;
use {crate::error::ErrorCode, std::fmt::Debug};

/// 用于映射到 ErrorCode::CalculationFailure 的辅助函数
pub fn map_zero_to_none(x: u128) -> Option<u128> {
    if x == 0 {
        None
    } else {
        Some(x)
    }
}

/// 交易的方向，因为曲线可以专门处理每个
/// 不同的标记（通过添加偏移量或权重）
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TradeDirection {
    /// 输入令牌0，输出令牌1
    ZeroForOne,
    /// 输入令牌1，输出令牌0
    OneForZero,
}

/// 圆的方向。  用于池代币到交易代币的转换
/// 避免任何存款或取款损失价值。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundDirection {
    /// Floor the value, ie. 1.9 => 1, 1.1 => 1, 1.5 => 1
    Floor,
    /// Ceiling the value, ie. 1.9 => 2, 1.1 => 2, 1.5 => 2
    Ceiling,
}

impl TradeDirection {
    /// 给定一个交易方向，给出交易的相反方向，所以
    /// A 到 B 变为 B 到 A，反之亦然
    pub fn opposite(&self) -> TradeDirection {
        match self {
            TradeDirection::ZeroForOne => TradeDirection::OneForZero,
            TradeDirection::OneForZero => TradeDirection::ZeroForOne,
        }
    }
}

/// 对双方同时存入的结果进行编码
#[derive(Debug, PartialEq)]
pub struct TradingTokenResult {
    /// 代币A的数量
    pub token_0_amount: u128,
    /// 代币B的数量
    pub token_1_amount: u128,
}

/// 对从源令牌交换到目标令牌的所有结果进行编码
#[derive(Debug, PartialEq)]
pub struct SwapResult {
    /// 新的源代币数量
    pub new_swap_source_amount: u128,
    /// 新的目的地代币数量
    pub new_swap_destination_amount: u128,
    /// 交换的源代币数量（包含费用）
    pub source_amount_swapped: u128,
    /// 交换的目标代币数量
    pub destination_amount_swapped: u128,
    /// 流向池持有者的源代币数量
    pub trade_fee: u128,
    /// 进入协议的源代币数量
    pub protocol_fee: u128,
    /// 流向协议团队的源代币数量
    pub fund_fee: u128,
}

/// 包裹执行计算的特征对象的具体结构。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveCalculator {}

impl CurveCalculator {
    pub fn validate_supply(token_0_amount: u64, token_1_amount: u64) -> Result<()> {
        if token_0_amount == 0 {
            return Err(ErrorCode::EmptySupply.into());
        }
        if token_1_amount == 0 {
            return Err(ErrorCode::EmptySupply.into());
        }
        Ok(())
    }

    /// 减去费用并计算将提供多少目的地代币
    /// 给定一定数量的源代币。
    pub fn swap_base_input(
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
    ) -> Option<SwapResult> {
        // 借记费用以计算交换金额
        let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
        let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
        let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

        let source_amount_less_fees = source_amount.checked_sub(trade_fee)?;

        let destination_amount_swapped = ConstantProductCurve::swap_base_input_without_fees(
            source_amount_less_fees,
            swap_source_amount,
            swap_destination_amount,
        );

        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destination_amount_swapped)?,
            source_amount_swapped: source_amount,
            destination_amount_swapped,
            trade_fee,
            protocol_fee,
            fund_fee,
        })
    }

    pub fn swap_base_output(
        destinsation_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
    ) -> Option<SwapResult> {
        let source_amount_swapped = ConstantProductCurve::swap_base_output_without_fees(
            destinsation_amount,
            swap_source_amount,
            swap_destination_amount,
        );

        let source_amount =
            Fees::calculate_pre_fee_amount(source_amount_swapped, trade_fee_rate).unwrap();
        let trade_fee = Fees::trading_fee(source_amount, trade_fee_rate)?;
        let protocol_fee = Fees::protocol_fee(trade_fee, protocol_fee_rate)?;
        let fund_fee = Fees::fund_fee(trade_fee, fund_fee_rate)?;

        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount)?,
            new_swap_destination_amount: swap_destination_amount
                .checked_sub(destinsation_amount)?,
            source_amount_swapped: source_amount,
            destination_amount_swapped: destinsation_amount,
            trade_fee,
            protocol_fee,
            fund_fee,
        })
    }

    /// 获取给定数量的池代币的交易代币数量，
    /// 提供总交易代币和矿池代币供应量。
    pub fn lp_tokens_to_trading_tokens(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        ConstantProductCurve::lp_tokens_to_trading_tokens(
            lp_token_amount,
            lp_token_supply,
            swap_token_0_amount,
            swap_token_1_amount,
            round_direction,
        )
    }
}

/// 曲线测试助手
#[cfg(test)]
pub mod test {
    use {
        super::*, proptest::prelude::*, spl_math::precise_number::PreciseNumber,
        spl_math::uint::U256,
    };

    /// 执行转换测试时大多数曲线的 epsilon，
    /// 将单方面存款与掉期+存款进行比较。
    pub const CONVERSION_BASIS_POINTS_GUARANTEE: u128 = 50;

    /// 计算给定流动性的曲线的总归一化值
    /// 参数。
    ///
    /// 该函数的恒定乘积实现给出了平方根
    /// Uniswap 不变量。
    pub fn normalized_value(
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
    ) -> Option<PreciseNumber> {
        let swap_token_a_amount = PreciseNumber::new(swap_token_a_amount)?;
        let swap_token_b_amount = PreciseNumber::new(swap_token_b_amount)?;
        swap_token_a_amount
            .checked_mul(&swap_token_b_amount)?
            .sqrt()
    }

    /// 测试函数检查交换不会降低总价值
    /// 游泳池。
    ///
    /// 由于曲线计算使用无符号整数，因此有可能
    /// 在某个时刻被截断，意味着可能会损失价值
    /// 如果给交换器太多，则向任一方向。
    ///
    /// 该测试保证值的相对变化最多为
    /// 1 个标准化代币，并且其价值永远不会因交易而减少。
    pub fn check_curve_value_from_swap(
        source_token_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_direction: TradeDirection,
    ) {
        let destination_amount_swapped = ConstantProductCurve::swap_base_input_without_fees(
            source_token_amount,
            swap_source_amount,
            swap_destination_amount,
        );

        let (swap_token_0_amount, swap_token_1_amount) = match trade_direction {
            TradeDirection::ZeroForOne => (swap_source_amount, swap_destination_amount),
            TradeDirection::OneForZero => (swap_destination_amount, swap_source_amount),
        };
        let previous_value = swap_token_0_amount
            .checked_mul(swap_token_1_amount)
            .unwrap();

        let new_swap_source_amount = swap_source_amount.checked_add(source_token_amount).unwrap();
        let new_swap_destination_amount = swap_destination_amount
            .checked_sub(destination_amount_swapped)
            .unwrap();
        let (swap_token_0_amount, swap_token_1_amount) = match trade_direction {
            TradeDirection::ZeroForOne => (new_swap_source_amount, new_swap_destination_amount),
            TradeDirection::OneForZero => (new_swap_destination_amount, new_swap_source_amount),
        };

        let new_value = swap_token_0_amount
            .checked_mul(swap_token_1_amount)
            .unwrap();
        assert!(new_value >= previous_value);
    }

    /// 测试功能检查存款不会减少矿池的价值
    /// 代币。
    ///
    /// 由于曲线计算使用无符号整数，因此有可能
    /// 在某个时刻被截断，这意味着如果出现以下情况，则可能会损失价值
    /// 给储户的钱太多了。
    pub fn check_pool_value_from_deposit(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
    ) {
        let deposit_result = CurveCalculator::lp_tokens_to_trading_tokens(
            lp_token_amount,
            lp_token_supply,
            swap_token_0_amount,
            swap_token_1_amount,
            RoundDirection::Ceiling,
        )
        .unwrap();
        let new_swap_token_0_amount = swap_token_0_amount + deposit_result.token_0_amount;
        let new_swap_token_1_amount = swap_token_1_amount + deposit_result.token_1_amount;
        let new_lp_token_supply = lp_token_supply + lp_token_amount;

        // 以下不等式必须成立：
        // new_token_a /new_pool_token_supply >= token_a /pool_token_supply
        // 这减少到：
        // new_token_a *pool_token_supply >= token_a *new_pool_token_supply

        // 存款后这些数字可能略高于 u64，这
        // 意味着它们的乘法可以略高于u128的范围。
        // 为了便于测试，我们将它们提升到 U256。
        let lp_token_supply = U256::from(lp_token_supply);
        let new_lp_token_supply = U256::from(new_lp_token_supply);
        let swap_token_0_amount = U256::from(swap_token_0_amount);
        let new_swap_token_0_amount = U256::from(new_swap_token_0_amount);
        let swap_token_b_amount = U256::from(swap_token_1_amount);
        let new_swap_token_b_amount = U256::from(new_swap_token_1_amount);

        assert!(
            new_swap_token_0_amount * lp_token_supply >= swap_token_0_amount * new_lp_token_supply
        );
        assert!(
            new_swap_token_b_amount * lp_token_supply >= swap_token_b_amount * new_lp_token_supply
        );
    }

    /// 测试功能检查提款不会减少矿池的价值
    /// 代币。
    ///
    /// 由于曲线计算使用无符号整数，因此有可能
    /// 在某个时刻被截断，这意味着如果出现以下情况，则可能会损失价值
    /// 给储户的钱太多了。
    pub fn check_pool_value_from_withdraw(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
    ) {
        let withdraw_result = CurveCalculator::lp_tokens_to_trading_tokens(
            lp_token_amount,
            lp_token_supply,
            swap_token_0_amount,
            swap_token_1_amount,
            RoundDirection::Floor,
        )
        .unwrap();
        let new_swap_token_0_amount = swap_token_0_amount - withdraw_result.token_0_amount;
        let new_swap_token_1_amount = swap_token_1_amount - withdraw_result.token_1_amount;
        let new_pool_token_supply = lp_token_supply - lp_token_amount;

        let value = normalized_value(swap_token_0_amount, swap_token_1_amount).unwrap();
        // 因为我们可以得到池值的舍入问题，这使得看起来
        // 每个代币的价值下降了，我们将其提高了 epsilon 1
        // 覆盖所有情况
        let new_value = normalized_value(new_swap_token_0_amount, new_swap_token_1_amount).unwrap();

        // 以下不等式必须成立：
        // new_pool_value /new_pool_token_supply >= pool_value /pool_token_supply
        // 也可以写成：
        // new_pool_value *pool_token_supply >= pool_value *new_pool_token_supply

        let lp_token_supply = PreciseNumber::new(lp_token_supply).unwrap();
        let new_lp_token_supply = PreciseNumber::new(new_pool_token_supply).unwrap();
        assert!(new_value
            .checked_mul(&lp_token_supply)
            .unwrap()
            .greater_than_or_equal(&value.checked_mul(&new_lp_token_supply).unwrap()));
    }

    prop_compose! {
        pub fn total_and_intermediate(max_value: u64)(total in 1..max_value)
                        (intermediate in 1..total, total in Just(total))
                        -> (u64, u64) {
           (total, intermediate)
       }
    }
}
