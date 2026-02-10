// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Fee estimation utilities for Allfeat runtimes.
//!
//! Provides tools to compute and display fee estimates for extrinsics,
//! including weight-based fees, length fees, and pallet-specific deposits.

extern crate alloc;

use crate::currency::{AFT, MICROAFT, MILLIAFT};
use allfeat_primitives::Balance;
use alloc::{format, string::String};
use frame_support::weights::Weight;

/// Default AFT price in USD for fee reports.
pub const DEFAULT_AFT_PRICE_USD: f64 = 0.02;

/// Runtime configuration values needed for fee estimation.
/// All values are read from the actual runtime configuration.
pub struct FeeReportConfig {
    /// Base extrinsic weight from `RuntimeBlockWeights`
    pub base_weight: Weight,
    /// `TransactionByteFee` from the runtime (fee per encoded byte)
    pub byte_fee: Balance,
    /// Weight-to-fee conversion function (from the runtime's `WeightToFee` polynomial)
    pub weight_to_fee_fn: fn(Weight) -> Balance,
    /// Minimum fee multiplier (from `MinimumMultiplier`, e.g. 1/10 = 0.1)
    pub min_multiplier_num: u128,
    pub min_multiplier_den: u128,
    /// Maximum fee multiplier (congestion scenario, e.g. 10/1 = 10.0)
    pub max_multiplier_num: u128,
    pub max_multiplier_den: u128,
    /// AFT price in USD
    pub aft_price_usd: f64,
}

/// Information about an extrinsic's fee components.
pub struct ExtrinsicFeeInfo {
    pub pallet: &'static str,
    pub extrinsic: &'static str,
    pub weight: Weight,
    pub encoded_len: u32,
    pub deposit: Balance,
}

/// Computed fee estimate result.
pub struct FeeEstimate {
    pub pallet: &'static str,
    pub extrinsic: &'static str,
    pub min_fee: Balance,
    pub max_fee: Balance,
    pub deposit: Balance,
    pub total_min: Balance,
    pub total_max: Balance,
}

/// Compute the fee for a given weight using the polynomial weight-to-fee conversion.
pub fn compute_fee(
    weight: Weight,
    base_weight: Weight,
    weight_to_fee_fn: fn(Weight) -> Balance,
    byte_fee: Balance,
    multiplier_num: u128,
    multiplier_den: u128,
    encoded_len: u32,
) -> Balance {
    let base_fee = weight_to_fee_fn(base_weight);
    let weight_fee = weight_to_fee_fn(weight);
    let length_fee = byte_fee.saturating_mul(encoded_len as Balance);

    // Apply multiplier: (base_fee + weight_fee) * multiplier + length_fee
    let adjusted = base_fee
        .saturating_add(weight_fee)
        .saturating_mul(multiplier_num)
        / multiplier_den;

    adjusted.saturating_add(length_fee)
}

/// Compute fee estimates for an extrinsic across min/max multiplier scenarios.
pub fn estimate_fees(info: &ExtrinsicFeeInfo, config: &FeeReportConfig) -> FeeEstimate {
    let min_fee = compute_fee(
        info.weight,
        config.base_weight,
        config.weight_to_fee_fn,
        config.byte_fee,
        config.min_multiplier_num,
        config.min_multiplier_den,
        info.encoded_len,
    );

    let max_fee = compute_fee(
        info.weight,
        config.base_weight,
        config.weight_to_fee_fn,
        config.byte_fee,
        config.max_multiplier_num,
        config.max_multiplier_den,
        info.encoded_len,
    );

    let total_min = min_fee.saturating_add(info.deposit);
    let total_max = max_fee.saturating_add(info.deposit);

    FeeEstimate {
        pallet: info.pallet,
        extrinsic: info.extrinsic,
        min_fee,
        max_fee,
        deposit: info.deposit,
        total_min,
        total_max,
    }
}

/// Convert a Balance (in planck units, 12 decimals) to AFT as f64.
pub fn balance_to_aft(balance: Balance) -> f64 {
    balance as f64 / AFT as f64
}

/// Convert AFT amount to USD.
pub fn aft_to_usd(aft: f64, price: f64) -> f64 {
    aft * price
}

/// Format a Balance as a human-readable AFT string.
pub fn format_balance(balance: Balance) -> String {
    if balance == 0 {
        return String::from("-");
    }
    let aft = balance / AFT;
    let remainder = balance % AFT;
    if aft > 0 {
        format!("{}.{:06} AFT", aft, remainder / MICROAFT)
    } else {
        format!("0.{:06} AFT", remainder / MICROAFT)
    }
}

/// Format a Balance in human-readable unit (auto-selects uAFT, mAFT, or AFT).
fn format_balance_unit(balance: Balance) -> String {
    if balance == 0 {
        return String::from("0");
    }
    if balance < MILLIAFT {
        // Display in uAFT
        let micro = balance / MICROAFT;
        let remainder = balance % MICROAFT;
        if remainder == 0 {
            format!("{} uAFT", micro)
        } else {
            format!("{}.{:06} uAFT", micro, remainder)
        }
    } else if balance < AFT {
        // Display in mAFT
        let milli = balance / MILLIAFT;
        let remainder = balance % MILLIAFT;
        if remainder == 0 {
            format!("{} mAFT", milli)
        } else {
            format!("{}.{:03} mAFT", milli, remainder / MICROAFT)
        }
    } else {
        format_balance(balance)
    }
}

/// Format a USD value.
pub fn format_usd(usd: f64) -> String {
    if usd < 0.000001 {
        String::from("~$0")
    } else {
        format!("${:.6}", usd)
    }
}

/// Format a total (fee + deposit) as USD.
fn format_total_usd(balance: Balance, aft_price: f64) -> String {
    format_usd(aft_to_usd(balance_to_aft(balance), aft_price))
}

/// Format a multiplier from num/den as a human-readable string.
fn format_multiplier(num: u128, den: u128) -> String {
    if den == 1 {
        format!("{}.0", num)
    } else {
        format!("{}", num as f64 / den as f64)
    }
}

/// Compute greatest common divisor (used to simplify multiplier rationals).
pub fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Print a complete fee estimation report to stdout.
/// All displayed configuration values come from the `FeeReportConfig` struct,
/// which should be populated from actual runtime constants.
pub fn print_fee_report(
    runtime_name: &str,
    estimates: &[FeeEstimate],
    config: &FeeReportConfig,
) {
    let col_pallet = 18;
    let col_ext = 22;
    let col_fee = 16;
    let col_deposit = 16;
    let col_total = 14;

    let total_width =
        col_pallet + col_ext + col_fee * 2 + col_deposit + col_total * 2 + 8; // 8 for separators

    // Top border
    println!();
    println!("{}", "=".repeat(total_width));

    // Title
    let title = format!(
        "{} - Fee Estimation Report  (1 AFT = ${:.3})",
        runtime_name, config.aft_price_usd
    );
    let padding = (total_width.saturating_sub(title.len())) / 2;
    println!("{}{}", " ".repeat(padding), title);

    println!("{}", "=".repeat(total_width));

    // Header
    println!(
        "{:<pw$} {:<ew$} {:>fw$} {:>fw$} {:>dw$} {:>tw$} {:>tw$}",
        "Pallet",
        "Extrinsic",
        "Min Fee",
        "Max Fee",
        "Deposit",
        "Total Min $",
        "Total Max $",
        pw = col_pallet,
        ew = col_ext,
        fw = col_fee,
        dw = col_deposit,
        tw = col_total,
    );
    println!("{}", "-".repeat(total_width));

    let mut current_pallet = "";

    for est in estimates {
        // Print separator between pallet groups
        if !current_pallet.is_empty() && current_pallet != est.pallet {
            println!(
                "{:<pw$} {:<ew$} {:>fw$} {:>fw$} {:>dw$} {:>tw$} {:>tw$}",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                pw = col_pallet,
                ew = col_ext,
                fw = col_fee,
                dw = col_deposit,
                tw = col_total,
            );
        }
        current_pallet = est.pallet;

        println!(
            "{:<pw$} {:<ew$} {:>fw$} {:>fw$} {:>dw$} {:>tw$} {:>tw$}",
            est.pallet,
            est.extrinsic,
            format_balance(est.min_fee),
            format_balance(est.max_fee),
            format_balance(est.deposit),
            format_total_usd(est.total_min, config.aft_price_usd),
            format_total_usd(est.total_max, config.aft_price_usd),
            pw = col_pallet,
            ew = col_ext,
            fw = col_fee,
            dw = col_deposit,
            tw = col_total,
        );
    }

    println!("{}", "=".repeat(total_width));

    // Configuration summary - all values from runtime
    let min_mult = format_multiplier(config.min_multiplier_num, config.min_multiplier_den);
    let max_mult = format_multiplier(config.max_multiplier_num, config.max_multiplier_den);

    // Compute WeightFeeFactor by evaluating weight_to_fee on the base_weight
    // and extracting the effective factor
    let one_second_weight = Weight::from_parts(1_000_000_000_000, 0);
    let fee_for_one_second = (config.weight_to_fee_fn)(one_second_weight);

    println!();
    println!("Configuration (from runtime):");
    println!("  AFT/USD Rate:           ${:.3}", config.aft_price_usd);
    println!(
        "  Fee Multiplier Range:   [{} (min), {} (congestion)]",
        min_mult, max_mult
    );
    println!(
        "  TransactionByteFee:     {}/byte",
        format_balance_unit(config.byte_fee)
    );
    println!(
        "  Base Extrinsic Weight:  {} ref_time",
        config.base_weight.ref_time()
    );
    println!(
        "  Fee for 1s ref_time:    {}",
        format_balance(fee_for_one_second)
    );
    println!();
}
