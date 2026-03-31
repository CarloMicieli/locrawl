# 🚂 locrawl

**The smart way to retrieve railway model data from the web.**

[](https://www.google.com/search?q=https://github.com/yourusername/locrawl/actions)
[](https://opensource.org/licenses/apache2.0)

`locrawl` is a high-performance CLI tool built in Rust to help railway enthusiasts and researchers aggregate data on rolling stock, track plans, and manufacturer catalogs. Stop manual scraping and start modeling.

## ✨ Features

* **Lightning Fast:** Built with Rust for memory safety and speed.
* **Smart Parsing:** Specifically tuned to recognize railway-specific metadata (scales, gauges, eras).
* **Multi-Format Export:** Save your data in JSON, CSV, or directly to a local database.
* **Clap-Powered:** A robust command-line interface with intuitive help and completions.

## 🚀 Installation

### From Source

Ensure you have the [Rust toolchain](https://rustup.rs/) installed:

```bash
git clone https://github.com/CarloMicieli/locrawl.git
cd locrawl
cargo install --path .
```

## 🛠 Usage

`locrawl` uses a simple command structure to navigate data sources.

### Basic Fetch

Retrieve data for a specific locomotive manufacturer:

```bash
locrawl fetch --source marklin --type diesel
```

### Search by Scale

Filter the internet for specific modeling scales (e.g., H0, N, O):

```bash
locrawl search "Class 66" --scale H0
```

### Exporting Data

```bash
locrawl fetch --all --output results.json
```

## 📖 Command Reference

| Command | Description |
| :--- | :--- |
| `fetch` | Pulls data from a predefined list of railway databases. |
| `search` | Queries various hobbyist forums and manufacturer sites. |
| `config` | Manage your API keys or local storage paths. |
| `update` | Refreshes the local cache of railway manufacturer IDs. |

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

-----

**Built with 🦀 by Carlo Micieli**

Would you like me to generate the `Cargo.toml` dependencies or the basic `main.rs` structure to match this README?