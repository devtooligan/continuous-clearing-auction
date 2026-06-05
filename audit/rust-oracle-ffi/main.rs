use cca_oracle::oracle::{
    can_clear_ideal, currency_raised_at_clearing_tick_ceiling, full_fill_currency_ceiling,
    full_fill_tokens_floor, partial_fill_currency_ceiling, partial_fill_tokens_floor,
    price_ceiling, public_view_floor, required_demand_ceiling, OracleError,
};
use num_bigint::BigUint;
use num_traits::Num;
use std::{env, process};

fn parse_u64(input: &str) -> u64 {
    input.parse::<u64>().unwrap_or_else(|err| {
        eprintln!("invalid u64 `{input}`: {err}");
        process::exit(2);
    })
}

fn parse_biguint(input: &str) -> BigUint {
    if let Some(hex) = input.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(input, 10)
    }
    .unwrap_or_else(|err| {
        eprintln!("invalid integer `{input}`: {err}");
        process::exit(2);
    })
}

fn encode_uint256(value: &BigUint) -> String {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        eprintln!("result does not fit uint256");
        process::exit(3);
    }

    let mut padded = [0_u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    let mut out = String::with_capacity(66);
    out.push_str("0x");
    for byte in padded {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn expect_arg_count(args: &[String], expected: usize, usage: &str) {
    if args.len() != expected {
        eprintln!("{usage}");
        process::exit(2);
    }
}

fn unwrap_or_exit(result: Result<BigUint, OracleError>) -> BigUint {
    result.unwrap_or_else(|err| {
        eprintln!("oracle error: {err:?}");
        process::exit(3);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: cca-oracle <command> [args...]");
        process::exit(2);
    }

    let result = match args[1].as_str() {
        "required-demand-ceil" => {
            expect_arg_count(
                &args,
                5,
                "usage: cca-oracle required-demand-ceil <remainingSupplyQ96X7> <priceQ96> <remainingMps>",
            );
            unwrap_or_exit(required_demand_ceiling(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                parse_u64(&args[4]),
            ))
        }
        "can-clear-ideal" => {
            expect_arg_count(
                &args,
                6,
                "usage: cca-oracle can-clear-ideal <demandQ96> <remainingSupplyQ96X7> <priceQ96> <remainingMps>",
            );
            let can_clear = can_clear_ideal(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                &parse_biguint(&args[4]),
                parse_u64(&args[5]),
            )
            .unwrap_or_else(|err| {
                eprintln!("oracle error: {err:?}");
                process::exit(3);
            });
            BigUint::from(can_clear as u8)
        }
        "price-ceil" => {
            expect_arg_count(
                &args,
                5,
                "usage: cca-oracle price-ceil <demandQ96> <remainingSupplyQ96X7> <remainingMps>",
            );
            unwrap_or_exit(price_ceiling(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                parse_u64(&args[4]),
            ))
        }
        "currency-raised-clearing-tick-ceil" => {
            expect_arg_count(
                &args,
                8,
                "usage: cca-oracle currency-raised-clearing-tick-ceil <remainingSupplyQ96X7> <demandAtPriceQ96> <demandAbovePriceQ96> <priceQ96> <deltaMps> <remainingMps>",
            );
            unwrap_or_exit(currency_raised_at_clearing_tick_ceiling(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                &parse_biguint(&args[4]),
                &parse_biguint(&args[5]),
                parse_u64(&args[6]),
                parse_u64(&args[7]),
            ))
        }
        "public-view-floor" => {
            expect_arg_count(&args, 3, "usage: cca-oracle public-view-floor <valueQ96X7>");
            public_view_floor(&parse_biguint(&args[2]))
        }
        "full-fill-currency-ceil" => {
            expect_arg_count(
                &args,
                5,
                "usage: cca-oracle full-fill-currency-ceil <amountQ96> <cumulativeMpsDelta> <mpsRemaining>",
            );
            unwrap_or_exit(full_fill_currency_ceiling(
                &parse_biguint(&args[2]),
                parse_u64(&args[3]),
                parse_u64(&args[4]),
            ))
        }
        "full-fill-tokens-floor" => {
            expect_arg_count(
                &args,
                5,
                "usage: cca-oracle full-fill-tokens-floor <amountQ96> <cumulativeMpsPerPriceDelta> <mpsRemaining>",
            );
            unwrap_or_exit(full_fill_tokens_floor(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                parse_u64(&args[4]),
            ))
        }
        "partial-fill-currency-ceil" => {
            expect_arg_count(
                &args,
                6,
                "usage: cca-oracle partial-fill-currency-ceil <amountQ96> <tickDemandQ96> <currencyRaisedAtClearingQ96X7> <mpsRemaining>",
            );
            unwrap_or_exit(partial_fill_currency_ceiling(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                &parse_biguint(&args[4]),
                parse_u64(&args[5]),
            ))
        }
        "partial-fill-tokens-floor" => {
            expect_arg_count(
                &args,
                7,
                "usage: cca-oracle partial-fill-tokens-floor <amountQ96> <maxPriceQ96> <tickDemandQ96> <currencyRaisedAtClearingQ96X7> <mpsRemaining>",
            );
            unwrap_or_exit(partial_fill_tokens_floor(
                &parse_biguint(&args[2]),
                &parse_biguint(&args[3]),
                &parse_biguint(&args[4]),
                &parse_biguint(&args[5]),
                parse_u64(&args[6]),
            ))
        }
        _ => {
            eprintln!("unknown command `{}`", args[1]);
            process::exit(2);
        }
    };

    print!("{}", encode_uint256(&result));
}
