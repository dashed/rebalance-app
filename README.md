rebalance-app
=============

[![Build Status](https://travis-ci.org/dashed/rebalance-app.svg?branch=master)](https://travis-ci.org/dashed/rebalance-app)


> Optimal lazy portfolio rebalancing calculator (in Rust)

![](./screenshot.png)


## Usage

```
$ rebalance-app --help
rebalance-app 1.2.0
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

### Quickstart

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

### About


**Rationale:** Rather than rebalance your portfolio internally, add/remove money such that your asset targets % are achieved as close as possible.

See this article on rebalancing: https://www.bogleheads.org/wiki/Rebalancing

Based on:

- http://optimalrebalancing.tk
- https://github.com/EDawg878/Portfolio-Rebalancer


**How it works:**

Source: http://optimalrebalancing.tk/explanation.html

*Step 1: Calculate the fractional deviations*

Define the fractional deviation `f` of an asset to be `a/t − 1`, where `t` is the asset's target allocation and `a` is its actual portion of the portfolio. Calculate `f` for each asset. `f` will be negative for underweighted assets and positive for overweighted assets. Note that a denotes the portion relative to the final total portfolio value; this is obtained by adding the contribution amount to the original total portfolio value.

*Step 2: Add money to asset(s) with lowest fractional deviation*

Add money to the asset(s) with least `f` until they are tied with the asset(s) with the next-least `f`. The money added to each asset must be proportional to that asset's target allocation so that the minimum `f`'s increase in synchrony. Repeat this until the contribution amount is exhausted. If the assets are pre-sorted according to `f`, this process can be implemented such that its running time increases linearly with the number of assets.

Chores
======

- `cargo fmt`

License
=======

GPL-3.0+.
