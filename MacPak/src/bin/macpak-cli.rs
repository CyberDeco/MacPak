//! MacPak CLI binary entry point

fn main() -> anyhow::Result<()> {
    MacPak::cli::run_cli()
}
