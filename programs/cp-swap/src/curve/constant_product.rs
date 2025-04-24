//! The Uniswap invariantConstantProductCurve::

use crate::{
    curve::calculator::{RoundDirection, TradingTokenResult},
    utils::CheckedCeilDiv,
};

/// ConstantProductCurve 结构实现 CurveCalculator
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstantProductCurve;

impl ConstantProductCurve {
    /// 恒定的乘积交换确保 x *y = 恒定
    /// 恒定的产品交换计算，从其类中提取以供重用。
    ///
    /// 这保证适用于所有值，例如：
    ///  -1 <= swap_source_amount *swap_destination_amount <= u128::MAX
    ///  -1 <= source_amount <= u64::MAX
    pub fn swap_base_input_without_fees(
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> u128 {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_y = (delta_x * y) / (x + delta_x)
        let numerator = source_amount.checked_mul(swap_destination_amount).unwrap();
        let denominator = swap_source_amount.checked_add(source_amount).unwrap();
        let destinsation_amount_swapped = numerator.checked_div(denominator).unwrap();
        destinsation_amount_swapped
    }

    pub fn swap_base_output_without_fees(
        destinsation_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
    ) -> u128 {
        // (x + delta_x) * (y - delta_y) = x * y
        // delta_x = (x * delta_y) / (y - delta_y)
        let numerator = swap_source_amount.checked_mul(destinsation_amount).unwrap();
        let denominator = swap_destination_amount
            .checked_sub(destinsation_amount)
            .unwrap();
        let (source_amount_swapped, _) = numerator.checked_ceil_div(denominator).unwrap();
        source_amount_swapped
    }

    /// 获取给定数量的池代币的交易代币数量，
    /// 提供总交易代币和矿池代币供应量。
    ///
    /// 恒定乘积实现是一个简单的比率计算，用于计算如何
    /// 许多交易代币对应一定数量的池代币
    pub fn lp_tokens_to_trading_tokens(
        lp_token_amount: u128,
        lp_token_supply: u128,
        swap_token_0_amount: u128,
        swap_token_1_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        let mut token_0_amount = lp_token_amount
            .checked_mul(swap_token_0_amount)?
            .checked_div(lp_token_supply)?;
        let mut token_1_amount = lp_token_amount
            .checked_mul(swap_token_1_amount)?
            .checked_div(lp_token_supply)?;
        let (token_0_amount, token_1_amount) = match round_direction {
            RoundDirection::Floor => (token_0_amount, token_1_amount),
            RoundDirection::Ceiling => {
                let token_0_remainder = lp_token_amount
                    .checked_mul(swap_token_0_amount)?
                    .checked_rem(lp_token_supply)?;
                // 同时检查代币 A 和 B 的数量是否为 0，以避免拿走太多
                // 对于少量的池代币。  例如，如果有人问
                // 对于 1 个池代币，其价值 0.01 代币 A，我们避免
                // 获取 1 个代币 A 并返回 0 的上限，因为它是
                // 稍后在处理中被拒绝。
                if token_0_remainder > 0 && token_0_amount > 0 {
                    token_0_amount += 1;
                }
                let token_1_remainder = lp_token_amount
                    .checked_mul(swap_token_1_amount)?
                    .checked_rem(lp_token_supply)?;
                if token_1_remainder > 0 && token_1_amount > 0 {
                    token_1_amount += 1;
                }
                (token_0_amount, token_1_amount)
            }
        };
        Some(TradingTokenResult {
            token_0_amount,
            token_1_amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::curve::calculator::{
            test::{
                check_curve_value_from_swap, check_pool_value_from_deposit,
                check_pool_value_from_withdraw, total_and_intermediate,
            },
            RoundDirection, TradeDirection,
        },
        proptest::prelude::*,
    };

    fn check_pool_token_rate(
        token_a: u128,
        token_b: u128,
        deposit: u128,
        supply: u128,
        expected_a: u128,
        expected_b: u128,
    ) {
        let results = ConstantProductCurve::lp_tokens_to_trading_tokens(
            deposit,
            supply,
            token_a,
            token_b,
            RoundDirection::Ceiling,
        )
        .unwrap();
        assert_eq!(results.token_0_amount, expected_a);
        assert_eq!(results.token_1_amount, expected_b);
    }

    #[test]
    fn trading_token_conversion() {
        check_pool_token_rate(2, 49, 5, 10, 1, 25);
        check_pool_token_rate(100, 202, 5, 101, 5, 10);
        check_pool_token_rate(5, 501, 2, 10, 1, 101);
    }

    #[test]
    fn fail_trading_token_conversion() {
        let results = ConstantProductCurve::lp_tokens_to_trading_tokens(
            5,
            10,
            u128::MAX,
            0,
            RoundDirection::Floor,
        );
        assert!(results.is_none());
        let results = ConstantProductCurve::lp_tokens_to_trading_tokens(
            5,
            10,
            0,
            u128::MAX,
            RoundDirection::Floor,
        );
        assert!(results.is_none());
    }

    fn test_truncation(
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        expected_source_amount_swapped: u128,
        expected_destination_amount_swapped: u128,
    ) {
        let invariant = swap_source_amount * swap_destination_amount;
        let destination_amount_swapped = ConstantProductCurve::swap_base_input_without_fees(
            source_amount,
            swap_source_amount,
            swap_destination_amount,
        );
        assert_eq!(source_amount, expected_source_amount_swapped);
        assert_eq!(
            destination_amount_swapped,
            expected_destination_amount_swapped
        );
        let new_invariant = (swap_source_amount + source_amount)
            * (swap_destination_amount - destination_amount_swapped);
        assert!(new_invariant >= invariant);
    }

    #[test]
    fn constant_product_swap_rounding() {
        let tests: &[(u128, u128, u128, u128, u128)] = &[
            // spot: 10 * 70b / ~4m = 174,999.99
            (10, 4_000_000, 70_000_000_000, 10, 174_999),
            // spot: 20 * 1 / 3.000 = 6.6667 (source can be 18 to get 6 dest.)
            (20, 30_000 - 20, 10_000, 20, 6),
            // spot: 19 * 1 / 2.999 = 6.3334 (source can be 18 to get 6 dest.)
            (19, 30_000 - 20, 10_000, 19, 6),
            // spot: 18 * 1 / 2.999 = 6.0001
            (18, 30_000 - 20, 10_000, 18, 6),
            // spot: 10 * 3 / 2.0010 = 14.99
            (10, 20_000, 30_000, 10, 14),
            // spot: 10 * 3 / 2.0001 = 14.999
            (10, 20_000 - 9, 30_000, 10, 14),
            // spot: 10 * 3 / 2.0000 = 15
            (10, 20_000 - 10, 30_000, 10, 15),
            // spot: 100 * 3 / 6.001 = 49.99 (source can be 99 to get 49 dest.)
            (100, 60_000, 30_000, 100, 49),
            // spot: 99 * 3 / 6.001 = 49.49
            (99, 60_000, 30_000, 99, 49),
            // spot: 98 * 3 / 6.001 = 48.99 (source can be 97 to get 48 dest.)
            (98, 60_000, 30_000, 98, 48),
        ];
        for (
            source_amount,
            swap_source_amount,
            swap_destination_amount,
            expected_source_amount,
            expected_destination_amount,
        ) in tests.iter()
        {
            test_truncation(
                *source_amount,
                *swap_source_amount,
                *swap_destination_amount,
                *expected_source_amount,
                *expected_destination_amount,
            );
        }
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_swap(
            source_token_amount in 1..u64::MAX,
            swap_source_amount in 1..u64::MAX,
            swap_destination_amount in 1..u64::MAX,
        ) {
            check_curve_value_from_swap(
                source_token_amount as u128,
                swap_source_amount as u128,
                swap_destination_amount as u128,
                TradeDirection::ZeroForOne
            );
        }
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_deposit(
            pool_token_amount in 1..u64::MAX,
            pool_token_supply in 1..u64::MAX,
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u64::MAX,
        ) {
            let pool_token_amount = pool_token_amount as u128;
            let pool_token_supply = pool_token_supply as u128;
            let swap_token_a_amount = swap_token_a_amount as u128;
            let swap_token_b_amount = swap_token_b_amount as u128;
            // 确保我们将为每个人至少获得一个交易代币
            // 侧，否则计算失败
            prop_assume!(pool_token_amount * swap_token_a_amount / pool_token_supply >= 1);
            prop_assume!(pool_token_amount * swap_token_b_amount / pool_token_supply >= 1);
            check_pool_value_from_deposit(
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
            );
        }
    }

    proptest! {
        #[test]
        fn curve_value_does_not_decrease_from_withdraw(
            (pool_token_supply, pool_token_amount) in total_and_intermediate(u64::MAX),
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u64::MAX,
        ) {
            let pool_token_amount = pool_token_amount as u128;
            let pool_token_supply = pool_token_supply as u128;
            let swap_token_a_amount = swap_token_a_amount as u128;
            let swap_token_b_amount = swap_token_b_amount as u128;
            // 确保我们将为每个人至少获得一个交易代币
            // 侧，否则计算失败
            prop_assume!(pool_token_amount * swap_token_a_amount / pool_token_supply >= 1);
            prop_assume!(pool_token_amount * swap_token_b_amount / pool_token_supply >= 1);
            check_pool_value_from_withdraw(
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
            );
        }
    }
}
