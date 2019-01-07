extern crate clap;
extern crate csv;
extern crate num;
extern crate tabwriter;

mod rebalance;

// rust imports

use std::collections::HashMap;

// 3rd-party imports

use clap::{App, AppSettings, Arg};

// local imports

use rebalance::{lazy_rebalance, to_string, Asset};

// app

fn main() {
    let matches = App::new("rebalance-app")
        .version("1.0")
        .author("Alberto Leal (github.com/dashed) <mailforalberto@gmail.com>")
        .about("Optimal lazy portfolio rebalancing calculator")
        .setting(AppSettings::AllowNegativeNumbers)
        .arg(
            Arg::with_name("targets")
                .short("t")
                .long("targets")
                .value_name("FILE")
                .help("Sets a targets file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portfolio")
                .short("p")
                .long("portfolio")
                .value_name("FILE")
                .help("Sets a portfolio file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("contribution")
                .help("Sets the contribution amount")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path_to_targets = matches.value_of("targets").unwrap();
    println!("Value for targets: {}", path_to_targets);

    let path_to_portfolio = matches.value_of("portfolio").unwrap();
    println!("Value for portfolio: {}", path_to_portfolio);

    let contribution_amount: f64 = matches
        .value_of("contribution")
        .map(|x| x.parse::<f64>().unwrap())
        .unwrap();

    println!(
        "Contributing: {}\n",
        format!("{:.*}", 2, contribution_amount)
    );

    let target_map = create_target_map(path_to_targets);

    let portfolio = create_portfolio(path_to_portfolio, target_map);

    let balanced_portfolio = lazy_rebalance(contribution_amount, portfolio);

    println!("{}", to_string(&balanced_portfolio));
}

struct Percent(f64);

fn create_target_map(path_to_targets: &str) -> HashMap<String, Percent> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path_to_targets)
        .unwrap();

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

fn create_portfolio(path_to_portfolio: &str, target_map: HashMap<String, Percent>) -> Vec<Asset> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path_to_portfolio)
        .unwrap();

    let mut portfolio_map: HashMap<String, Asset> = HashMap::new();

    for result in reader.records() {
        let record = result.unwrap();

        let asset_name = record.get(0).unwrap().trim().to_string();

        let value = {
            let value: String = record.get(1).unwrap().trim().chars().skip(1).collect();

            value.parse::<f64>().unwrap()
        };

        match target_map.get(&asset_name) {
            None => {}
            Some(&Percent(target_allocation_percent)) => {
                let target_allocation_percent =
                    adjust_target_allocation_percent(target_allocation_percent);

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

        let target_allocation_percent = adjust_target_allocation_percent(target_allocation_percent);

        let asset = Asset::new(asset_name.clone(), target_allocation_percent, 0.0);

        portfolio_map.insert(asset_name.to_string(), asset);
    }

    let mut portfolio = vec![];

    for (_asset_name, asset) in portfolio_map {
        portfolio.push(asset);
    }

    portfolio
}

fn adjust_target_allocation_percent(target_allocation_percent: f64) -> f64 {
    target_allocation_percent / 100.0
}
