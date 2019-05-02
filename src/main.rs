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
        .version("1.2.0")
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
            Arg::with_name("portfolio_value_index")
                .short("i")
                .long("portfolio_value_index")
                .value_name("INDEX")
                .help("Sets CSV index of the portfolio value")
                .required(false)
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

    let path_to_portfolio = matches.value_of("portfolio").unwrap();

    let portfolio_value_index = matches
        .value_of("portfolio_value_index")
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(1);

    let contribution_amount: f64 = matches
        .value_of("contribution")
        .map(|x| x.parse::<f64>().unwrap())
        .unwrap();

    println!(
        "Contributing: {}\n",
        format!("{:.*}", 2, contribution_amount)
    );

    let target_map = create_target_map(path_to_targets);

    let portfolio = create_portfolio(path_to_portfolio, portfolio_value_index, target_map);

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

fn create_portfolio(
    path_to_portfolio: &str,
    portfolio_value_index: usize,
    target_map: HashMap<String, Percent>,
) -> Vec<Asset> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path_to_portfolio)
        .unwrap();

    let mut portfolio_map: HashMap<String, Asset> = HashMap::new();

    for result in reader.records() {
        let record = result.unwrap();

        let asset_name = record.get(0).unwrap().trim().to_string();

        let value = {
            let value: String = record
                .get(portfolio_value_index)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let path_to_targets = "example/targets.csv";
        let path_to_portfolio = "example/portfolio.csv";
        let contribution_amount = 10000.00;
        let portfolio_value_index = 1;

        let target_map = create_target_map(path_to_targets);

        let portfolio = create_portfolio(path_to_portfolio, portfolio_value_index, target_map);

        let balanced_portfolio = lazy_rebalance(contribution_amount, portfolio);

        let expected = r###"
Asset name               Asset value  Holdings %  New holdings %  Target allocation %  Target value  $ to buy/sell
TIPS fund                6500.00      6.500       9.935           10.000               11000.00      4428.57
Bond fund                16500.00     16.500      19.870          20.000               22000.00      5357.14
Domestic Stock ETF       43500.00     43.500      39.740          40.000               44000.00      214.29
International Stock ETF  33500.00     33.500      30.455          30.000               33000.00      0.00
Total                    100000.00    100.000     100.000         100.000              110000.00     10000.00
        "###.trim();

        assert_eq!(to_string(&balanced_portfolio), expected);
    }

}
