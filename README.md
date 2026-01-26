<div align="center">
  <a href="https://www.normalfinance.io/">
    <img src="https://cdn.normalapi.com/logo/logo-full.png" alt="Normal logo" width="340"/>
  </a>
</div>

<div>
  <a href="https://discord.gg/hayb9pafjZ"><img src="https://img.shields.io/discord/928701482319101952"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/releases"><img src="https://img.shields.io/github/release-pre/normalfinance/stellar-v1.svg"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/pulse"><img src="https://img.shields.io/github/contributors/normalfinance/stellar-v1.svg"/></a>
  <a href="https://opensource.org/license/apache-2-0"><img src="https://img.shields.io/github/license/normalfinance/stellar-v1"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/pulse"><img src="https://img.shields.io/github/last-commit/normalfinance/stellar-v1.svg"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/pulls"><img src="https://img.shields.io/github/issues-pr/normalfinance/stellar-v1.svg"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/issues"><img src="https://img.shields.io/github/issues/normalfinance/stellar-v1.svg"/></a>
  <a href="https://github.com/normalfinance/stellar-v1/issues"><img src="https://img.shields.io/github/issues-closed/normalfinance/stellar-v1.svg"/></a>
</div>

# ✨ Normal Stellar v1 🦄

Normal is a synthetic asset protocol enabling investors to trade any global asset entirely on-chain.

## How Does it Work?

Normal Tokens (our synthetic assets) are USDC-backed assets that track the value of a target asset (i.e. Bitcoin). We took heavy inspiration

https://medium.com/uma-project/introducing-umas-long-short-pair-lsp-financial-primitive-84596803864f

## Features

- Mint and redeem Normal token pairs with USDC to earn LP yield
- Earn additional yield by providing Normal token pairs as liquidity to the Treasury
- Trade long and short exposure to Bitcoin, Ethereum, Solana, Ripple, and Cardano without leaving Stellar

## Upcoming Features

- [ ] Support for Top 30 cryptocurrencies
- [ ] Support for other asset classes such as commodities, stocks, and bonds
- [ ] Custom index funds for automated diversification and portfolio management

## Smart Contracts

- **long_short_pair** - A DeFi primitive for financial derivatives using long and short tokens
- **long_short_pair_factory** - Factory for creating and interacting with Long Short Pair contracts
- **long_short_pair_calculator** - Price logic to create linear Long Short Pairs
- **normal_oracle** - A standardized oracle supporting multiple providers and risk measures
- **treasury** - A primary market for Long Short Pair tokens with accurate price enforcement and LP protections

## Modules

- **access_control** - Handles permissioned access to contracts using role-based access control (RBAC)
- **oracle** - Handles oracle types and utilities
- **token_pair** - Handles long short pair token utilities
- **types** - Handles contract types
- **upgrade** - Handles contract upgrades
- **utils** - Handles shared types, utils, constants, errors, macros, and more

## Built With

- [Rust](https://www.rust-lang.org/)
- [Soroban](https://soroban.stellar.org/)
- [Rust Soroban SDK](https://github.com/stellar/rs-soroban-sdk)

## Getting Started

### Prerequisites

- [Task](https://taskfile.dev/) as task runner
- installed latest Rust version
- [soroban cli](https://github.com/stellar/soroban-tools)

### Development setup

#### Clone project

`git clone git@github.com:normalfinance/stellar-v1.git`

#### Build contracts

`task build`

#### Run tests

`task test`

#### (Optionally) Deploy & invoke contracts via soroban-cli

check the Soroban documentation: https://soroban.stellar.org/docs/reference/rpc

## Authors

- [@jblewnormal](https://github.com/jblewnormal)
- [@jaymalve](ttps://github.com/jaymalve)

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Audits

You can find audits of this codebase in the `/audits` directory. These are our most recent audits:

- **Summer 2025 x Halborn** - (https://github.com/normalfinance/stellar-v1/audits/)

## Contact

- 📧 Email: [hello@normalfinance.io](mailto:hello@normalfinance.io)
- ✈️ Telegram: [@normalfinance](https://t.me/normalfinance)
- 🐣 Twitter: [@normalfi](https://twitter.com/normalfi)
- 🥷🏼 GitHub: [@normalfinance](https://github.com/normalfinance)
- 👾 Discord: [@Normal](https://discord.gg/cB3DVSddQR)
- 📚 Docs: [@normalfinance](https://docs.normalfinance.io/)
- 🤓 Blog: [@normalfinance](https://blog.normalfinance.io/)

## License

[Apache-2.0](https://choosealicense.com/licenses/apache-2.0/)
