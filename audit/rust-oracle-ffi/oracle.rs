use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

pub const MPS: u64 = 10_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleError {
    DivisionByZero,
}

pub fn q96() -> BigUint {
    BigUint::one() << 96_u32
}

pub fn rat_from_biguint(value: &BigUint) -> BigRational {
    BigRational::from_integer(BigInt::from_biguint(Sign::Plus, value.clone()))
}

pub fn rat_from_u64(value: u64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

pub fn floor_to_biguint(value: &BigRational) -> BigUint {
    assert!(
        !value.is_negative(),
        "negative oracle value cannot project to uint"
    );
    (value.numer() / value.denom())
        .to_biguint()
        .expect("non-negative")
}

pub fn ceil_to_biguint(value: &BigRational) -> BigUint {
    assert!(
        !value.is_negative(),
        "negative oracle value cannot project to uint"
    );
    let numerator = value.numer().clone();
    let denominator = value.denom().clone();
    let quotient = &numerator / &denominator;
    let remainder = numerator % denominator;
    let result = if remainder.is_zero() {
        quotient
    } else {
        quotient + BigInt::one()
    };
    result.to_biguint().expect("non-negative")
}

pub fn ceil_div(numerator: BigUint, denominator: BigUint) -> Result<BigUint, OracleError> {
    if denominator.is_zero() {
        return Err(OracleError::DivisionByZero);
    }
    let (quotient, remainder) = numerator.div_rem(&denominator);
    Ok(if remainder.is_zero() {
        quotient
    } else {
        quotient + BigUint::one()
    })
}

pub fn required_demand_exact(
    remaining_supply_q96x7: &BigUint,
    price_q96: &BigUint,
    remaining_mps: u64,
) -> Result<BigRational, OracleError> {
    if remaining_mps == 0 {
        return Err(OracleError::DivisionByZero);
    }
    Ok(
        rat_from_biguint(remaining_supply_q96x7) * rat_from_biguint(price_q96)
            / (rat_from_biguint(&q96()) * rat_from_u64(remaining_mps)),
    )
}

pub fn required_demand_ceiling(
    remaining_supply_q96x7: &BigUint,
    price_q96: &BigUint,
    remaining_mps: u64,
) -> Result<BigUint, OracleError> {
    Ok(ceil_to_biguint(&required_demand_exact(
        remaining_supply_q96x7,
        price_q96,
        remaining_mps,
    )?))
}

pub fn can_clear_ideal(
    demand_q96: &BigUint,
    remaining_supply_q96x7: &BigUint,
    price_q96: &BigUint,
    remaining_mps: u64,
) -> Result<bool, OracleError> {
    Ok(rat_from_biguint(demand_q96)
        >= required_demand_exact(remaining_supply_q96x7, price_q96, remaining_mps)?)
}

pub fn price_ceiling_exact(
    demand_q96: &BigUint,
    remaining_supply_q96x7: &BigUint,
    remaining_mps: u64,
) -> Result<BigRational, OracleError> {
    if remaining_supply_q96x7.is_zero() {
        return Err(OracleError::DivisionByZero);
    }
    Ok(
        rat_from_biguint(demand_q96) * rat_from_u64(remaining_mps) * rat_from_biguint(&q96())
            / rat_from_biguint(remaining_supply_q96x7),
    )
}

pub fn price_ceiling(
    demand_q96: &BigUint,
    remaining_supply_q96x7: &BigUint,
    remaining_mps: u64,
) -> Result<BigUint, OracleError> {
    Ok(ceil_to_biguint(&price_ceiling_exact(
        demand_q96,
        remaining_supply_q96x7,
        remaining_mps,
    )?))
}

pub fn currency_raised_at_clearing_tick_exact(
    remaining_supply_q96x7: &BigUint,
    demand_at_price_q96: &BigUint,
    demand_above_price_q96: &BigUint,
    price_q96: &BigUint,
    delta_mps: u64,
    remaining_mps: u64,
) -> Result<BigRational, OracleError> {
    if remaining_mps == 0 {
        return Err(OracleError::DivisionByZero);
    }

    let total_currency = rat_from_biguint(remaining_supply_q96x7)
        * rat_from_biguint(price_q96)
        * rat_from_u64(delta_mps)
        / (rat_from_biguint(&q96()) * rat_from_u64(remaining_mps));
    let currency_above = rat_from_biguint(demand_above_price_q96) * rat_from_u64(delta_mps);
    let max_at_price = rat_from_biguint(demand_at_price_q96) * rat_from_u64(delta_mps);
    let complement = if total_currency > currency_above {
        total_currency - currency_above
    } else {
        BigRational::zero()
    };

    Ok(if complement < max_at_price {
        complement
    } else {
        max_at_price
    })
}

pub fn currency_raised_at_clearing_tick_ceiling(
    remaining_supply_q96x7: &BigUint,
    demand_at_price_q96: &BigUint,
    demand_above_price_q96: &BigUint,
    price_q96: &BigUint,
    delta_mps: u64,
    remaining_mps: u64,
) -> Result<BigUint, OracleError> {
    Ok(ceil_to_biguint(&currency_raised_at_clearing_tick_exact(
        remaining_supply_q96x7,
        demand_at_price_q96,
        demand_above_price_q96,
        price_q96,
        delta_mps,
        remaining_mps,
    )?))
}

pub fn public_view_floor(value_q96x7: &BigUint) -> BigUint {
    value_q96x7 / (q96() * BigUint::from(MPS))
}

pub fn full_fill_currency_ceiling(
    amount_q96: &BigUint,
    cumulative_mps_delta: u64,
    mps_remaining: u64,
) -> Result<BigUint, OracleError> {
    ceil_div(
        amount_q96 * BigUint::from(cumulative_mps_delta),
        BigUint::from(mps_remaining),
    )
}

pub fn full_fill_tokens_floor(
    amount_q96: &BigUint,
    cumulative_mps_per_price_delta: &BigUint,
    mps_remaining: u64,
) -> Result<BigUint, OracleError> {
    let denominator = (BigUint::one() << 192_u32) * BigUint::from(mps_remaining);
    if denominator.is_zero() {
        return Err(OracleError::DivisionByZero);
    }
    Ok(amount_q96 * cumulative_mps_per_price_delta / denominator)
}

pub fn partial_fill_currency_ceiling(
    amount_q96: &BigUint,
    tick_demand_q96: &BigUint,
    currency_raised_at_clearing_q96x7: &BigUint,
    mps_remaining: u64,
) -> Result<BigUint, OracleError> {
    ceil_div(
        amount_q96 * currency_raised_at_clearing_q96x7,
        tick_demand_q96 * BigUint::from(mps_remaining),
    )
}

pub fn partial_fill_tokens_floor(
    amount_q96: &BigUint,
    max_price_q96: &BigUint,
    tick_demand_q96: &BigUint,
    currency_raised_at_clearing_q96x7: &BigUint,
    mps_remaining: u64,
) -> Result<BigUint, OracleError> {
    let denominator = tick_demand_q96 * BigUint::from(mps_remaining);
    if denominator.is_zero() || max_price_q96.is_zero() {
        return Err(OracleError::DivisionByZero);
    }
    let currency_floor = amount_q96 * currency_raised_at_clearing_q96x7 / denominator;
    Ok(currency_floor / max_price_q96)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn bu(value: u128) -> BigUint {
        BigUint::from(value)
    }

    #[test]
    fn required_demand_rounds_up_from_exact_rational() {
        let supply = q96() * BigUint::from(21_u8);
        let exact = required_demand_exact(&supply, &BigUint::from(5_u8), 2).unwrap();
        assert_eq!(floor_to_biguint(&exact), BigUint::from(52_u8));
        assert_eq!(
            required_demand_ceiling(&supply, &BigUint::from(5_u8), 2).unwrap(),
            BigUint::from(53_u8)
        );
    }

    #[test]
    fn clearability_matches_exact_threshold() {
        let supply = BigUint::from(21_u8);
        let price = BigUint::from(5_u8);
        let required = required_demand_ceiling(&supply, &price, 2).unwrap();
        assert!(
            !can_clear_ideal(&(required.clone() - BigUint::one()), &supply, &price, 2).unwrap()
        );
        assert!(can_clear_ideal(&required, &supply, &price, 2).unwrap());
    }

    #[test]
    fn public_view_projection_floors() {
        let scale = q96() * BigUint::from(MPS);
        assert_eq!(
            public_view_floor(&(scale.clone() * BigUint::from(7_u8) - BigUint::one())),
            BigUint::from(6_u8)
        );
    }

    #[test]
    fn bid_accounting_projection_directions_are_explicit() {
        assert_eq!(
            full_fill_currency_ceiling(&BigUint::from(101_u8), 3, 7).unwrap(),
            BigUint::from(44_u8)
        );
        let c_mps_per_price =
            (BigUint::one() << 192_u32) * BigUint::from(11_u8) + BigUint::from(5_u8);
        assert_eq!(
            full_fill_tokens_floor(&BigUint::from(101_u8), &c_mps_per_price, 7).unwrap(),
            BigUint::from(158_u8)
        );
    }

    proptest! {
        #[test]
        fn required_demand_is_monotone_in_price(
            supply in 1_u128..=u128::MAX / 1_000_000,
            price_a in 1_u128..=u128::MAX / 1_000_000,
            price_b in 1_u128..=u128::MAX / 1_000_000,
            remaining_mps in 1_u64..=MPS,
        ) {
            let low = price_a.min(price_b);
            let high = price_a.max(price_b);
            let required_low = required_demand_ceiling(&bu(supply), &bu(low), remaining_mps).unwrap();
            let required_high = required_demand_ceiling(&bu(supply), &bu(high), remaining_mps).unwrap();
            prop_assert!(required_low <= required_high);
        }

        #[test]
        fn can_clear_turns_true_at_required_demand(
            supply in 1_u128..=u128::MAX / 1_000_000,
            price in 1_u128..=u128::MAX / 1_000_000,
            remaining_mps in 1_u64..=MPS,
        ) {
            let required = required_demand_ceiling(&bu(supply), &bu(price), remaining_mps).unwrap();
            if required > BigUint::zero() {
                prop_assert!(!can_clear_ideal(&(required.clone() - BigUint::one()), &bu(supply), &bu(price), remaining_mps).unwrap());
            }
            prop_assert!(can_clear_ideal(&required, &bu(supply), &bu(price), remaining_mps).unwrap());
        }
    }
}
