extern crate csv;

// rust imports

use std::collections::HashMap;

struct Percent(f64);
#[derive(Debug)]
struct Value(f64);

struct Asset {
    value: Value,
    target_allocation_percent: Percent
}


fn main() {

    // let contents = read_file_to_string("assets/fundaccountdetails.csv");
    // let contents = read_file_to_string("assets/targets.csv");
    let target_map = create_target_map();

    let portfolio_map = create_portfolio_map(target_map);

    println!("Hello, world!");
}

fn create_target_map() -> HashMap<String, Percent> {

    let mut reader = csv::Reader::from_path("assets/targets.csv").unwrap();

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

fn create_portfolio_map(target_map: HashMap<String, Percent>) -> HashMap<String, Asset> {

    let mut reader = csv::Reader::from_path("assets/fundaccountdetails.csv").unwrap();

    let mut portfolio_map = HashMap::new();

    for result in reader.records() {

        let record = result.unwrap();

        let asset_name = record.get(0).unwrap().trim().to_string();

        let value = {
            let value: String = record
                .get(3)
                .unwrap()
                .trim()
                .chars()
                .skip(1)
                .collect();

            Value(value.parse::<f64>().unwrap())
        };

        match target_map.get(&asset_name) {
            None => {},
            Some(&Percent(target_allocation_percent)) => {

                let asset = Asset {
                    value: value,
                    target_allocation_percent: Percent(target_allocation_percent)
                };

                portfolio_map.insert(asset_name, asset);
            }
        }

    }

    portfolio_map
}
