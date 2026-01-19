//! MacPak CLI binary entry point

fn main() -> anyhow::Result<()> {
    macpak::cli::run_cli()
}
