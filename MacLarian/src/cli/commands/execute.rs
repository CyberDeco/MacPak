//! Command execution implementations

use super::definitions::{
    Gr2Commands, IndexCommands, LocaCommands, ModCommands, PakCommands, SearchCommands,
    TextureCommands, VirtualTextureCommands,
};
use super::{convert, create, extract, gr2, list, loca, mod_cmd, pak, search, texture, virtual_texture};
use super::Commands;

impl Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Commands::Extract {
                source,
                destination,
                filter,
                file,
                quiet,
                bundle,
                convert_gr2,
                extract_textures,
                extract_virtual_textures,
                game_data,
                virtual_textures,
                delete_gr2,
                png,
                keep_dds,
            } => extract::execute(
                source,
                destination,
                filter.as_deref(),
                file.as_deref(),
                !*quiet,
                &extract::Gr2CliOptions {
                    bundle: *bundle,
                    convert_gr2: *convert_gr2,
                    extract_textures: *extract_textures,
                    extract_virtual_textures: *extract_virtual_textures,
                    game_data: game_data.clone(),
                    virtual_textures: virtual_textures.clone(),
                    delete_gr2: *delete_gr2,
                    convert_textures_to_png: *png,
                    keep_original_dds: *keep_dds,
                },
            ),
            Commands::Convert {
                source,
                destination,
                input_format,
                output_format,
            } => convert::execute(
                source,
                destination,
                input_format.as_deref(),
                output_format.as_deref(),
            ),
            Commands::Create { source, destination, compression } => {
                create::execute(source, destination, compression)
            }
            Commands::List {
                source,
                detailed,
                filter,
                count,
            } => list::execute(source, *detailed, filter.as_deref(), *count),
            Commands::Gr2 { command } => command.execute(),
            Commands::VirtualTexture { command } => command.execute(),
            Commands::Mod { command } => command.execute(),
            Commands::Search { command } => command.execute(),
            Commands::Index { command } => command.execute(),
            Commands::Pak { command } => command.execute(),
            Commands::Loca { command } => command.execute(),
            Commands::Texture { command } => command.execute(),
        }
    }
}

impl Gr2Commands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Gr2Commands::Inspect { path } => {
                gr2::inspect(path)
            }
            Gr2Commands::Extract { path, output } => {
                gr2::extract_json(path, output)
            }
            Gr2Commands::Decompress { path, output } => {
                gr2::decompress(path, output.as_deref())
            }
            Gr2Commands::ToGlb { path, output } => {
                gr2::convert_to_glb(path, output.as_deref())
            }
            Gr2Commands::FromGltf { path, output } => {
                gr2::convert_to_gr2(path, output.as_deref())
            }
            Gr2Commands::Bundle { path, output, game_data, virtual_textures, no_glb, no_textures, gltf, png, keep_dds } => {
                gr2::bundle(
                    path,
                    output.as_deref(),
                    game_data.as_deref(),
                    virtual_textures.as_deref(),
                    *no_glb,
                    *no_textures,
                    *gltf,
                    *png,
                    *keep_dds,
                )
            }
            Gr2Commands::ToGlbTextured { path, textures_pak, output } => {
                gr2::convert_to_glb_textured(path, textures_pak, output.as_deref())
            }
        }
    }
}

impl VirtualTextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            VirtualTextureCommands::List { path } => {
                virtual_texture::list(path)
            }
            VirtualTextureCommands::Extract { path, output, texture, layer, all_layers } => {
                let layers: Vec<usize> = layer.iter().map(|l| l.0).collect();
                virtual_texture::extract(
                    path,
                    output,
                    texture.as_deref(),
                    &layers,
                    *all_layers,
                )
            }
            VirtualTextureCommands::Create { name, base, normal, physical, output, compression, no_embed_mip } => {
                virtual_texture::create(
                    name,
                    base.as_deref(),
                    normal.as_deref(),
                    physical.as_deref(),
                    output,
                    compression.as_deref(),
                    *no_embed_mip,
                )
            }
            VirtualTextureCommands::Batch { input, output, layer, recursive } => {
                let layers: Vec<usize> = layer.iter().map(|l| l.0).collect();
                virtual_texture::batch(input, output, &layers, *recursive)
            }
            VirtualTextureCommands::GtpInfo { path, gts } => {
                virtual_texture::gtp_info(path, gts.as_deref())
            }
        }
    }
}

impl ModCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            ModCommands::Validate { source } => mod_cmd::validate(source),
            ModCommands::InfoJson {
                pak,
                extracted,
                output,
            } => mod_cmd::info_json(pak, extracted, output.as_deref()),
        }
    }
}

impl SearchCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            SearchCommands::FileName {
                pak,
                query,
                type_filter,
            } => search::search_filename(pak, query, type_filter.as_deref()),
            SearchCommands::Path {
                pak,
                query,
                type_filter,
            } => search::search_path(pak, query, type_filter.as_deref()),
            SearchCommands::Uuid { pak, uuid } => search::search_uuid(pak, uuid),
            SearchCommands::Content { pak, query, limit } => {
                search::search_content(pak, query, *limit)
            }
            SearchCommands::FromIndex {
                index_dir,
                query,
                limit,
            } => search::search_index(index_dir, query, *limit),
        }
    }
}

impl IndexCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            IndexCommands::Build {
                paks,
                output,
                fulltext,
            } => {
                // Build index from multiple PAKs
                for pak in paks {
                    search::build_index(pak, Some(output), *fulltext)?;
                }
                Ok(())
            }
            IndexCommands::Stats { index_dir } => search::index_stats(index_dir),
        }
    }
}

impl PakCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            PakCommands::Info { pak } => pak::info(pak),
            PakCommands::Find { dir } => pak::find(dir),
            PakCommands::BatchExtract { source, dest } => pak::batch_extract_cmd(source, dest),
            PakCommands::BatchCreate { source, dest } => pak::batch_create_cmd(source, dest),
        }
    }
}

impl LocaCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            LocaCommands::List { path, limit } => loca::list(path, *limit),
            LocaCommands::Get { path, handle } => loca::get(path, handle),
            LocaCommands::Search { path, query, limit } => loca::search(path, query, *limit),
            LocaCommands::Export { path, output } => loca::export_xml(path, output),
        }
    }
}

impl TextureCommands {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            TextureCommands::Info { path } => texture::info(path),
            TextureCommands::Convert { input, output, format } => {
                texture::convert(input, output, format.as_deref())
            }
            TextureCommands::BatchConvert {
                dir,
                output,
                to,
                dds_format,
            } => texture::batch_convert(dir, output, to, dds_format.as_deref()),
        }
    }
}
