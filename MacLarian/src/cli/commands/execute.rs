//! Command execution implementations

use super::Commands;
use super::definitions::{
    Gr2Commands, LocaCommands, ModCommands, PakCommands, TextureCommands,
    VirtualTextureCommands,
};
use super::{convert, gr2, loca, mod_cmd, pak, texture, virtual_texture};

impl Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Commands::Pak { command } => command.execute(),
            Commands::Convert {
                source,
                destination,
                output_format,
                texture_format,
                quiet,
            } => convert::execute(
                source,
                destination,
                output_format.as_deref(),
                texture_format,
                *quiet,
            ),
            Commands::Gr2 { command } => command.execute(),
            Commands::VirtualTexture { command } => command.execute(),
            Commands::Mods { command } => command.execute(),
            Commands::Loca { command } => command.execute(),
            Commands::Texture { command } => command.execute(),
        }
    }
}

impl PakCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            PakCommands::Extract {
                source,
                destination,
                filter,
                file,
                quiet,
            } => pak::extract(
                source,
                destination,
                filter.as_deref(),
                file.as_deref(),
                *quiet,
            ),
            PakCommands::Create {
                source,
                destination,
                compression,
                quiet,
            } => pak::create(source, destination, compression, *quiet),
            PakCommands::List {
                source,
                detailed,
                filter,
                count,
                quiet,
            } => pak::list(source, *detailed, filter.as_deref(), *count, *quiet),
        }
    }
}

impl Gr2Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Gr2Commands::Inspect { path, output } => gr2::inspect(path, output.as_deref()),
            Gr2Commands::FromGr2 {
                source,
                destination,
                format,
                textures,
                bg3_path,
                quiet,
            } => gr2::from_gr2(
                source,
                destination,
                format,
                textures.as_deref(),
                bg3_path.as_deref(),
                *quiet,
            ),
            Gr2Commands::ToGr2 {
                source,
                destination,
                quiet,
            } => gr2::to_gr2(source, destination, *quiet),
        }
    }
}

impl VirtualTextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            VirtualTextureCommands::List {
                path,
                detailed,
                output,
            } => virtual_texture::list(path, *detailed, output.as_deref()),
            VirtualTextureCommands::Extract {
                source,
                destination,
                gtex,
                layer,
                quiet,
            } => {
                let layers: Vec<usize> = layer.iter().map(|l| l.0).collect();
                virtual_texture::extract(source, destination, gtex.as_deref(), &layers, *quiet)
            }
            VirtualTextureCommands::Create {
                source,
                destination,
                gtex,
                base,
                normal,
                physical,
                compression,
                no_embed_mip,
                quiet,
            } => virtual_texture::create(
                source,
                destination,
                gtex.as_deref(),
                base.as_deref(),
                normal.as_deref(),
                physical.as_deref(),
                compression,
                *no_embed_mip,
                *quiet,
            ),
        }
    }
}

impl LocaCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            LocaCommands::Search {
                path,
                query,
                handle,
                limit,
                quiet,
            } => loca::search(path, query, *handle, *limit, *quiet),
        }
    }
}

impl TextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            TextureCommands::Info { path } => texture::info(path),
        }
    }
}

impl ModCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            ModCommands::Validate {
                source,
                recursive,
                paks_only,
                dirs_only,
                quiet,
            } => mod_cmd::validate(source, *recursive, *paks_only, *dirs_only, *quiet),
            ModCommands::Package {
                source,
                destination,
                compress,
                quiet,
            } => mod_cmd::package(source, destination, compress.as_deref(), *quiet),
            ModCommands::Meta {
                source,
                name,
                author,
                description,
                folder,
                uuid,
                version,
            } => mod_cmd::meta(
                source,
                name,
                author,
                description,
                folder.as_deref(),
                uuid.as_deref(),
                version,
            ),
            ModCommands::Vtex {
                source,
                output,
                quiet,
            } => virtual_texture::discover(source, output.as_deref(), *quiet),
            ModCommands::Conflicts { sources, quiet } => mod_cmd::conflicts(sources, *quiet),
        }
    }
}
