// rust imports

use std::cmp::Ordering;
use std::io::Write;

// 3rd-party imports

use num::traits::cast::FromPrimitive;
use num::BigRational;
use num::{One, Zero};
use num::{Signed, ToPrimitive};

use chrono::prelude::*;

use tabwriter::TabWriter;

pub struct Asset {
    name: String,
    value: BigRational,
    actual_allocation: BigRational,
    target_allocation_percent: BigRational,

    target_value: Option<BigRational>,
    deviation: Option<BigRational>,
    delta: Option<BigRational>,
}

impl Asset {
    pub fn new(name: String, target_percent: f64, value: f64) -> Self {
        assert!(target_percent <= 1.0);
        assert!(target_percent >= 0.0);

        Asset {
            name: name,

            value: BigRational::from_f64(value).unwrap(),
            actual_allocation: BigRational::zero(),
            target_allocation_percent: BigRational::from_f64(target_percent).unwrap(),

            target_value: None,
            deviation: None,
            delta: None,
        }
    }
}

fn comparator(left: &Asset, right: &Asset) -> Ordering {
    if left.deviation < right.deviation {
        return Ordering::Less;
    }

    if left.deviation > right.deviation {
        return Ordering::Greater;
    }

    Ordering::Equal
}

pub fn lazy_rebalance(amount_to_contribute: f64, mut assets: Vec<Asset>) -> Vec<Asset> {
    let amount_to_contribute = BigRational::from_f64(amount_to_contribute).unwrap();

    let portfolio_total: BigRational = assets
        .iter()
        .fold(BigRational::zero(), |total, ref asset| total + &asset.value);

    let total: BigRational = &portfolio_total + &amount_to_contribute;

    for asset in assets.iter_mut() {
        let target_value = &total * &asset.target_allocation_percent;

        // equivalent to: (value - target_value) / target_value
        // see: https://en.wikipedia.org/wiki/Approximation_error#Formal_Definition
        // 
        // This will be negative for underweighted assets and positive for overweighted assets.
        let deviation = (&asset.value / &target_value) - BigRational::one();

        asset.actual_allocation = if portfolio_total <= BigRational::zero() {
            BigRational::zero()
        } else {
            &asset.value / &portfolio_total
        };

        asset.target_value = Some(target_value);
        asset.deviation = Some(deviation);
    }

    assets.sort_by(|left, right| {
        let result = comparator(left, right);

        if amount_to_contribute < BigRational::zero() {
            result.reverse()
        } else {
            result
        }
    });

    let (largest_least_deviation, index_to_stop): (BigRational, usize) = {
        // since deviations are approx. errors, original author wanted to 'minimize' approx. errors
        // of assets with 'lowest' approx. errors. in other words, assets with lowest approx. error
        // gets first dibs of the contribution first.

        let mut cumulative_target_value: BigRational = BigRational::zero();

        let mut amount_left_to_contribute: BigRational = amount_to_contribute.clone();
        // TODO: wtf is this
        // Idea:
        // Add money to the asset(s) with least fractional deviation until they are tied with the asset(s) with the 
        // next-least fractional deviation.
        // 
        // When contributing to assets with lowest fractional deviation to the largest fractional deviation, we may be
        // unable to contribute to all the assets in the portfolio.
        // 
        // The last asset we contribute to will be compared to `largest_least_deviation`.
        let mut largest_least_deviation: BigRational = BigRational::zero();

        let mut last_known_index = None;

        for (index, asset) in assets.iter().enumerate() {
            if amount_left_to_contribute.abs() <= BigRational::zero() {
                break;
            }

            let asset: &Asset = asset;
            last_known_index = Some(index);

            largest_least_deviation = asset.deviation.as_ref().unwrap().clone();

            let target_value = asset.target_value.as_ref().unwrap();
            cumulative_target_value = &cumulative_target_value + target_value;

            let next_least_deviation = if index >= (assets.len() - 1) {
                BigRational::zero()
            } else {
                assets[index + 1].deviation.as_ref().unwrap().clone()
            };

            // TODO: todo-note
            // println!("delta: {}", to_f64(&(&next_least_deviation - &largest_least_deviation)));

            // For the first asset, this is the amount contributed.
            let contribution: BigRational = &cumulative_target_value * (&next_least_deviation - &largest_least_deviation);

            // TODO: todo-note
            // println!("contribution: {}", to_f64(&contribution));

            if contribution.abs() <= amount_left_to_contribute.abs() {
                amount_left_to_contribute = amount_left_to_contribute - contribution;
                largest_least_deviation = next_least_deviation;
            } else {
                largest_least_deviation = largest_least_deviation + (amount_left_to_contribute / &cumulative_target_value);

                // TODO: remove
                // amount_left_to_contribute = BigRational::zero();

                break;
            }
        }

        match last_known_index {
            Some(last_known_index) => {
                // We contribute to all assets before index_to_stop.
                let index_to_stop = last_known_index + 1;
                (largest_least_deviation, index_to_stop)
            }
            None => (largest_least_deviation, 0),
        }
    };

    // Calculate amount to contribute for each eligible asset.
    // Eligible assets are those before `index_to_stop`.
    for (index, asset) in assets.iter_mut().enumerate() {
        if index >= index_to_stop {
            break;
        }

        let target_value = asset.target_value.as_ref().unwrap();

        let deviation = asset.deviation.as_ref().unwrap();


        /****
         
        Definition:
        deviation = asset_market_value / target_asset_market_value - 1
        
        Goal: (asset_market_value + delta) / target_asset_market_value - 1 = largest_least_deviation
        
        
        Solve for delta from above:
        (asset_market_value + delta) / target_asset_market_value = largest_least_deviation + 1
        asset_market_value + delta = (largest_least_deviation + 1) * target_asset_market_value
        
        (1) delta = (largest_least_deviation + 1) * target_asset_market_value - asset_market_value
        
        
        Assume:
        delta = target_asset_market_value * (largest_least_deviation - deviation)
        
        Manipulate algebra to see it's equivalent to (1):
        delta = target_asset_market_value * (largest_least_deviation - [asset_market_value / target_asset_market_value - 1])
        delta = target_asset_market_value * (largest_least_deviation + [1 - asset_market_value / target_asset_market_value])
        delta = (largest_least_deviation + 1) * target_asset_market_value - asset_market_value

        ****/
        let delta = target_value * (&largest_least_deviation - deviation);

        asset.delta = Some(delta);
    }

    assets
}

fn to_f64(fraction: &BigRational) -> f64 {
    let numerator = fraction.numer();
    let denominator = fraction.denom();

    numerator.to_f64().unwrap() / denominator.to_f64().unwrap()
}

// pub fn to_debug_string(balanced_portfolio: &Vec<Asset>) -> String {
//     let mut buf = "ASSET NAME\tTARGET\tVALUE".to_string();

//     for asset in balanced_portfolio {
//         // let delta = match asset.delta {
//         //     Some(ref delta) => delta.clone(),
//         //     None => BigRational::zero(),
//         // };

//         let target_allocation_percent = to_f64(&asset.target_allocation_percent);

//         let target_allocation_percent = if target_allocation_percent <= 1.0 {
//             target_allocation_percent * 100.0
//         } else {
//             target_allocation_percent
//         };

//         // let actual_allocation = to_f64(&asset.actual_allocation);

//         // let target_value = &(asset.target_value.clone()).unwrap();

//         // let final_portion =
//         //     (&asset.value + &delta) * &asset.target_allocation_percent / target_value;

//         let line = format!(
//             "{}\t{}%\t{}",
//             asset.name,
//             format_f64(target_allocation_percent, 3),
//             format_f64(to_f64(&asset.value), 2)
//         );

//         buf = format!("{}\n{}", buf, line);
//     }

//     for asset in balanced_portfolio {
//         // let delta = match asset.delta {
//         //     Some(ref delta) => delta.clone(),
//         //     None => BigRational::zero(),
//         // };

//         let target_allocation_percent = to_f64(&asset.target_allocation_percent);

//         let target_allocation_percent = if target_allocation_percent <= 1.0 {
//             target_allocation_percent * 100.0
//         } else {
//             target_allocation_percent
//         };

//         // let actual_allocation = to_f64(&asset.actual_allocation);

//         // let target_value = &(asset.target_value.clone()).unwrap();

//         // let final_portion =
//         //     (&asset.value + &delta) * &asset.target_allocation_percent / target_value;

//         let line = format!(
//             "list.push(new Asset(1, `{}`, {}/100, {}));",
//             asset.name,
//             format_f64(target_allocation_percent, 3),
//             format_f64(to_f64(&asset.value), 2)
//         );

//         buf = format!("{}\n{}", buf, line);
//     }

//     let mut tw = TabWriter::new(vec![]);

//     tw.write_all(buf.as_bytes()).unwrap();
//     tw.flush().unwrap();

//     String::from_utf8(tw.into_inner().unwrap()).unwrap()
// }

pub fn to_ledger_string(
    balanced_portfolio: &Vec<Asset>,
    dest_account_name: &str,
    source_account_name: &str,
) -> String {
    let mut buf: String = "".to_string();

    for asset in balanced_portfolio {
        let delta = match asset.delta {
            Some(ref delta) => delta.clone(),
            None => BigRational::zero(),
        };

        if delta == BigRational::zero() {
            continue;
        }

        let date_time_now = Local::now().format("%Y-%m-%d").to_string();

        let amount_to_contribute = format_f64(to_f64(&delta), 2);
        let amount_to_withdraw = format_f64(-to_f64(&delta), 2);

        let line: String = if delta <= BigRational::zero() {
            format!(
                r#"
{date} * Withdrawal from {account_name}
    {dest_account_name:76}{amount_to_contribute} CAD
    {source_account_name:76}{amount_to_withdraw} CAD
    "#,
                date = date_time_now,
                account_name = asset.name,
                dest_account_name = dest_account_name,
                amount_to_contribute = amount_to_contribute,
                source_account_name = source_account_name,
                amount_to_withdraw = amount_to_withdraw
            )
            .trim()
            .to_string()
        } else {
            format!(
                r#"
{date} * Contribution to {account_name}
    {dest_account_name:76}{amount_to_contribute} CAD
    {source_account_name:76}{amount_to_withdraw} CAD
    "#,
                date = date_time_now,
                account_name = asset.name,
                dest_account_name = dest_account_name,
                amount_to_contribute = amount_to_contribute,
                source_account_name = source_account_name,
                amount_to_withdraw = amount_to_withdraw
            )
            .trim()
            .to_string()
        };

        buf = format!("{}\n{}\n", buf, line);
    }

    return buf.to_string();
}

pub fn to_string(balanced_portfolio: &Vec<Asset>) -> String {
    let mut buf = "Asset name\tAsset value\tHoldings %\tNew holdings %\tTarget allocation \
                   %\tTarget value\t$ to buy/sell"
        .to_string();

    let mut total_asset_value = BigRational::zero();
    let mut total_current_holdings = BigRational::zero();
    let mut total_new_holdings = BigRational::zero();
    let mut total_target_allocation = BigRational::zero();
    let mut total_target_value = BigRational::zero();
    let mut total_contribution = 0.0;

    for asset in balanced_portfolio {
        let delta = match asset.delta {
            Some(ref delta) => delta.clone(),
            None => BigRational::zero(),
        };

        let target_allocation_percent =
            if asset.target_allocation_percent <= BigRational::from_f64(1.0).unwrap() {
                asset.target_allocation_percent.clone() * BigRational::from_f64(100.00).unwrap()
            } else {
                asset.target_allocation_percent.clone()
            };

        let actual_allocation = &asset.actual_allocation * BigRational::from_f64(100.00).unwrap();

        let target_value = &(asset.target_value.clone()).unwrap();

        let final_portion =
            (&asset.value + &delta) * &asset.target_allocation_percent / target_value;

        let final_portion = &final_portion * BigRational::from_f64(100.00).unwrap();

        // totals

        total_asset_value = total_asset_value + &asset.value;
        total_current_holdings = total_current_holdings + &actual_allocation;
        total_new_holdings = total_new_holdings + &final_portion;
        total_target_allocation = total_target_allocation + &target_allocation_percent;
        total_target_value = total_target_value + target_value;
        let actual_delta = (to_f64(&delta) * 100.0).round() / 100.0;
        total_contribution = total_contribution + actual_delta;

        // generate line

        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            asset.name,
            format_f64(to_f64(&asset.value), 2),
            format_f64(to_f64(&actual_allocation), 3),
            format_f64(to_f64(&final_portion), 3),
            format_f64(to_f64(&target_allocation_percent), 3),
            format_f64(to_f64(&target_value), 2),
            format_f64(actual_delta, 2)
        );

        buf = format!("{}\n{}", buf, line);
    }

    let total_line = format!(
        "Total\t{}\t{}\t{}\t{}\t{}\t{}",
        format_f64(to_f64(&total_asset_value), 2),
        format_f64(to_f64(&total_current_holdings), 3),
        format_f64(to_f64(&total_new_holdings), 3),
        format_f64(to_f64(&total_target_allocation), 3),
        format_f64(to_f64(&total_target_value), 2),
        format_f64(total_contribution, 2)
    );

    buf = format!("{}\n{}", buf, total_line);

    let mut tw = TabWriter::new(vec![]);

    tw.write_all(buf.as_bytes()).unwrap();
    tw.flush().unwrap();

    String::from_utf8(tw.into_inner().unwrap()).unwrap()
}

fn format_f64(price: f64, dec_places: usize) -> String {
    format!("{:.*}", dec_places, price)
}
