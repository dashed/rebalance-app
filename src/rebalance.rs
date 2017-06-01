// rust imports

use std::cmp::Ordering;
use std::io::Write;

// 3rd-party imports

use num::BigRational;
use num::{Zero, One};
use num::traits::cast::FromPrimitive;
use num::{ToPrimitive, Signed};

use tabwriter::TabWriter;

pub struct Asset {
    name: String,
    value: BigRational,
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

    let total: BigRational = assets.iter()
        .fold(amount_to_contribute.clone(),
              |total, ref asset| total + &asset.value);

    for asset in assets.iter_mut() {

        let target_value = &total * &asset.target_allocation_percent;

        let deviation = (&asset.value / &target_value) - BigRational::one();

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

    let (__k, index_to_stop): (BigRational, usize) = {

        // TODO: wtf is this
        let mut __h: BigRational = BigRational::zero();
        let mut amount_left_to_contribute: BigRational = amount_to_contribute.clone();
        // TODO: wtf is this
        let mut __k: BigRational = BigRational::zero();
        let mut index_to_stop = 0;

        for (index, asset) in assets.iter().enumerate() {
            let asset: &Asset = asset;
            index_to_stop = index;

            // TODO: remove
            // if amount_left_to_contribute.abs() <= BigRational::zero() {
            //     break;
            // }

            __k = asset.deviation.as_ref().unwrap().clone();

            let target_value = asset.target_value.as_ref().unwrap();
            __h = &__h + target_value;

            let next_least_deviation = if index >= (assets.len() - 1) {
                BigRational::zero()
            } else {
                assets[index + 1].deviation.as_ref().unwrap().clone()
            };

            // TODO: todo-note
            // println!("delta: {}", to_f64(&(&next_least_deviation - &__k)));

            let __t: BigRational = &__h * (&next_least_deviation - &__k);

            // TODO: todo-note
            // println!("__t: {}", to_f64(&__t));

            if __t.abs() <= amount_left_to_contribute.abs() {
                amount_left_to_contribute = amount_left_to_contribute - __t;
                __k = next_least_deviation;
            } else {

                __k = __k + (amount_left_to_contribute / &__h);

                index_to_stop += 1;
                break;
                // TODO: remove
                // amount_left_to_contribute = BigRational::zero();
            }

        }

        (__k, index_to_stop)
    };

    for (index, asset) in assets.iter_mut().enumerate() {

        if index >= index_to_stop {
            break;
        }

        let target_value = asset.target_value.as_ref().unwrap();

        // TODO: todo-note
        // println!("{}: target value: {}", asset.name, to_f64(&target_value));

        let deviation = asset.deviation.as_ref().unwrap();

        let delta = target_value * (&__k - deviation);

        // TODO: todo-note
        // println!("{}: delta: {}", asset.name, to_f64(&delta));

        asset.delta = Some(delta);
    }

    assets

}

fn to_f64(fraction: &BigRational) -> f64 {

    let numerator = fraction.numer();
    let denominator = fraction.denom();

    numerator.to_f64().unwrap() / denominator.to_f64().unwrap()
}

pub fn to_string(balanced_portfolio: Vec<Asset>) -> String {

    let mut buf = "Asset name\tAsset value\t Target allocation\tTarget value\tAmount to buy/sell".to_string();

    for asset in balanced_portfolio {

        let delta = match asset.delta {
            Some(delta) => {
                to_f64(&delta)
            }
            None => {
                0.0
            }
        };

        let target_allocation_percent = to_f64(&asset.target_allocation_percent);

        let target_allocation_percent = if target_allocation_percent <= 1.0 {
            target_allocation_percent * 100.0
        } else {
            target_allocation_percent
        };

        let line = format!("{}\t{}\t{} %\t{}\t{}",
                           asset.name,
                           format_price(to_f64(&asset.value)),
                           target_allocation_percent,
                           format_price(to_f64(&asset.target_value.unwrap())),
                           format_price(delta));

        buf = format!("{}\n{}", buf, line);
    }

    let mut tw = TabWriter::new(vec![]);

    tw.write_all(buf.as_bytes()).unwrap();
    tw.flush().unwrap();

    String::from_utf8(tw.into_inner().unwrap()).unwrap()
}

fn format_price(price: f64) -> String {
    format!("{:.*}", 2, price)
}
