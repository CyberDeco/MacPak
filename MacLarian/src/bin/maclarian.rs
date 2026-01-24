//! MacLarian CLI binary entry point

fn main() -> anyhow::Result<()> {
    maclarian::cli::run_cli()
}
