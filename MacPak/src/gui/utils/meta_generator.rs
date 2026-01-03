//! Meta.lsx Generator
//!
//! Generate mod metadata files for BG3 mods.

/// Convert version components to BG3's int64 format
fn version_to_int64(major: u32, minor: u32, patch: u32, build: u32) -> i64 {
    // BG3 version format: major << 55 | minor << 47 | patch << 31 | build
    ((major as i64) << 55) | ((minor as i64) << 47) | ((patch as i64) << 31) | (build as i64)
}

/// Generate the meta.lsx XML content
pub fn generate_meta_lsx(
    mod_name: &str,
    folder: &str,
    author: &str,
    description: &str,
    uuid: &str,
    version_major: u32,
    version_minor: u32,
    version_patch: u32,
    version_build: u32,
) -> String {
    let version64 = version_to_int64(version_major, version_minor, version_patch, version_build);

    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
    <version major="4" minor="0" revision="9" build="331"/>
    <region id="Config">
        <node id="root">
            <children>
                <node id="Dependencies"/>
                <node id="ModuleInfo">
                    <!-- Generated using MacPak: https://github.com/CyberDeco/MacPak -->
                    <attribute id="Author" type="LSWString" value="{author}"/>
                    <attribute id="CharacterCreationLevelName" type="FixedString" value=""/>
                    <attribute id="Description" type="LSWString" value="{description}"/>
                    <attribute id="Folder" type="LSWString" value="{folder}"/>
                    <attribute id="LobbyLevelName" type="FixedString" value=""/>
                    <attribute id="MD5" type="LSString" value=""/>
                    <attribute id="MainMenuBackgroundVideo" type="FixedString" value=""/>
                    <attribute id="MenuLevelName" type="FixedString" value=""/>
                    <attribute id="Name" type="LSString" value="{mod_name}"/>
                    <attribute id="NumPlayers" type="uint8" value="4"/>
                    <attribute id="PhotoBooth" type="FixedString" value=""/>
                    <attribute id="StartupLevelName" type="FixedString" value=""/>
                    <attribute id="Tags" type="LSWString" value=""/>
                    <attribute id="Type" type="FixedString" value="Add-on"/>
                    <attribute id="UUID" type="FixedString" value="{uuid}"/>
                    <attribute id="Version64" type="int64" value="{version64}"/>
                    <children>
                        <node id="PublishVersion">
                            <attribute id="Version64" type="int64" value="{version64}"/>
                        </node>
                        <node id="TargetModes">
                            <children>
                                <node id="Target">
                                    <attribute id="Object" type="FixedString" value="Story"/>
                                </node>
                            </children>
                        </node>
                    </children>
                </node>
            </children>
        </node>
    </region>
</save>"#,
        author = author,
        description = description,
        folder = folder,
        mod_name = mod_name,
        uuid = uuid,
        version64 = version64,
    )
}
