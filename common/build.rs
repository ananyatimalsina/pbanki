fn main() {
    let embed_kind = if cfg!(feature = "software-renderer") {
        slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer
    } else {
        slint_build::EmbedResourcesKind::EmbedFiles
    };

    let config = slint_build::CompilerConfiguration::new().embed_resources(embed_kind);
    slint_build::compile_with_config("../ui/main.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();
}
