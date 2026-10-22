fn main() {
    tauri_build::build();
    if matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("windows")
    ) {
        // Tauri embeds Common Controls v6 into app binaries only. Integration
        // tests also link native menus, whose TaskDialogIndirect import needs it.
        let resource = std::path::PathBuf::from(
            std::env::var_os("OUT_DIR").expect("Cargo must supply the resource directory"),
        )
        .join("resource.rc");
        embed_resource::compile_for_tests(resource, embed_resource::NONE)
            .manifest_required()
            .expect("native tests require the same Windows manifest as the app");
    }
}
