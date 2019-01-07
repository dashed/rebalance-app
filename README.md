rebalance-app
=============

> Optimal lazy portfolio rebalancing calculator (in Rust)

![](./screenshot.png)


## Usage

```
$ rebalance-app --help
rebalance-app 1.0
Alberto Leal (github.com/dashed) <mailforalberto@gmail.com>
Optimal lazy portfolio rebalancing calculator

USAGE:
    rebalance-app [OPTIONS] <contribution> --portfolio <FILE> --targets <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --portfolio <FILE>                 Sets a portfolio file
    -i, --portfolio_value_index <INDEX>    Sets CSV index of the portfolio value
    -t, --targets <FILE>                   Sets a targets file

ARGS:
    <contribution>    Sets the contribution amount
```

1. Set up `targets.csv`. Example: [example/targets.csv](example/targets.csv)

2. Set up `portfolio.csv`. Usually you export this from your favourite broker. Example: [example/portfolio.csv](example/portfolio.csv)

3. Run: `rebalance-app --portfolio example/portfolio.csv --targets example/targets.csv 5000`

By default, `rebalance-app` assumes the values of your assets in your portfolio CSV file is at the 2nd column (i.e. index 1). For example:

```
Bond fund,                 roi %, ticker symbol, $16500.00
TIPS fund,                 roi %, ticker symbol, $6500.00
Domestic Stock ETF,        roi %, ticker symbol, $43500.00
International Stock ETF,   roi %, ticker symbol, $33500.00
```

You can adjust this using the `-i` flag:

```
rebalance-app -i 3 --portfolio example/portfolio.csv --targets example/targets.csv 5000
```

----------



**Rationale:** Rather than rebalance your portfolio internally, add/remove money such that your asset targets % are archieved as close as possible.

Based on:

- http://optimalrebalancing.tk/
- https://github.com/EDawg878/Portfolio-Rebalancer

**NOTE:** CSV input is currently opinionated towards data exported from my investment accounts. I will eventually change this.



Chores
======

- `cargo fmt`

License
=======

GPL-3.0+.
