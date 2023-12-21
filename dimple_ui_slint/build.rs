fn main() {
    // https://slint.dev/releases/1.3.2/docs/slint/src/advanced/style
    let config =
        slint_build::CompilerConfiguration::new()
        .with_style("material".into());
    slint_build::compile_with_config("ui/app_window.slint", config).unwrap();
}
