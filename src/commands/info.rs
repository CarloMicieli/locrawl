use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let name = "locrawl".cyan();
    let version = env!("CARGO_PKG_VERSION");
    let summary =
        "A CLI tool for retrieving railway model data from manufacturer websites and webshops.";

    println!("{} v{}", name, version);
    println!("{}", summary);

    Ok(())
}
