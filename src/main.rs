extern crate csv;
extern crate num;
extern crate tabwriter;

mod rebalance;

// rust imports

use std::collections::HashMap;
use std::env::args;

// local imports

use rebalance::{Asset, lazy_rebalance, to_string};

fn main() {

    let contribution_amount = {
        let mut args = args();
        args.next();
        let first_arg: String = args.next().expect("No contribution amount entered.");
        first_arg.parse::<f64>().unwrap()
    };

    println!("Contributing: {}\n",
             format!("{:.*}", 2, contribution_amount));

    let target_map = create_target_map();

    let portfolio = create_portfolio(target_map);

    let balanced_portfolio = lazy_rebalance(contribution_amount, portfolio);

    println!("{}", to_string(balanced_portfolio));
}

struct Percent(f64);

fn create_target_map() -> HashMap<String, Percent> {

    let mut reader =
        csv::ReaderBuilder::new().has_headers(false).from_path("assets/targets.csv").unwrap();

    let mut target_map = HashMap::new();

    for result in reader.records() {

        let record = result.unwrap();

        let asset_name = record.get(0).unwrap().trim().to_string();
        let allocation: Percent = {

            let column = record.get(1).unwrap().trim();

            let allocation = column.parse::<f64>().unwrap();

            if allocation <= 0.0 {
                continue;
            }

            Percent(allocation)
        };

        target_map.insert(asset_name, allocation);

    }

    target_map
}

fn create_portfolio(target_map: HashMap<String, Percent>) -> Vec<Asset> {

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("assets/fundaccountdetails.csv")
        .unwrap();

    let mut portfolio_map: HashMap<String, Asset> = HashMap::new();

    for result in reader.records() {

        let record = result.unwrap();

        let asset_name = record.get(0).unwrap().trim().to_string();

        let value = {
            let value: String = record.get(3)
                .unwrap()
                .trim()
                .chars()
                .skip(1)
                .collect();

            value.parse::<f64>().unwrap()
        };

        match target_map.get(&asset_name) {
            None => {}
            Some(&Percent(target_allocation_percent)) => {

                let target_allocation_percent =
                    adjust_target_allocation_percent(target_allocation_percent,
                                                     target_map.len() > 1);

                let asset = Asset::new(asset_name.clone(), target_allocation_percent, value);

                portfolio_map.insert(asset_name, asset);
            }
        }

    }

    for asset_name in target_map.keys() {

        if portfolio_map.contains_key(asset_name) {
            continue;
        }

        let &Percent(target_allocation_percent) = target_map.get(asset_name).unwrap();

        let target_allocation_percent = adjust_target_allocation_percent(target_allocation_percent,
                                                                         target_map.len() > 1);

        let asset = Asset::new(asset_name.clone(), target_allocation_percent, 0.0);

        portfolio_map.insert(asset_name.to_string(), asset);
    }

    let mut portfolio = vec![];

    for (_asset_name, asset) in portfolio_map {

        portfolio.push(asset);
    }

    portfolio
}


fn adjust_target_allocation_percent(target_allocation_percent: f64,
                                    has_more_than_one_target: bool)
                                    -> f64 {

    if target_allocation_percent >= 1.0 && has_more_than_one_target {
        target_allocation_percent / 100.0
    } else {
        target_allocation_percent
    }

}
