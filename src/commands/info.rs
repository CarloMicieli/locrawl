use log::info;

pub fn run() -> anyhow::Result<()> {
    let name = "locrawl";
    let version = env!("CARGO_PKG_VERSION");
    let summary =
        "A CLI tool for retrieving railway model data from manufacturer websites and webshops.";

    info!("{} v{}", name, version);
    info!("{}", summary);

    Ok(())
}
