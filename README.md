rebalance
=========

> Optimal lazy portfolio rebalancing calculator (in Rust)

![](./screenshot.png)

**Rationale:** Rather than rebalance your portfolio internally, add/remove money such that your asset targets % are archieved as close as possible.

Based on:

- http://optimalrebalancing.tk/
- https://github.com/EDawg878/Portfolio-Rebalancer

**NOTE:** CSV input is currently opinionated towards data exported from my investment accounts. I will eventually change this.

To Do
=====

- [ ] refactor out rebalance algorithm as a crate.
- [ ] documentation and usage
- [ ] refactor for less opinionated csv input
- [ ] CLI using clap-rs

License
=======

GPL-3.0+.
