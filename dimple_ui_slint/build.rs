fn main() {
    // https://slint.dev/releases/1.3.2/docs/slint/src/advanced/style
    let config =
        slint_build::CompilerConfiguration::new()
        .with_style("material".into());
    slint_build::compile_with_config("ui/app_window.slint", config).unwrap();

    // println!("cargo:rustc-link-lib=framework=OpenGL");
    // println!("cargo:rustc-link-lib=framework=QtGui");

    // you are still responsible for 
    // (1) arranging for the compiled binary to link against those frameworks 
    // (e.g. by emitting lines like cargo:rustc-link-lib=framework=SDL2 from 
    // your build.rs script), and 
    // (2) embedding the correct rpath in your binary (e.g. by running 
    // install_name_tool -add_rpath "@executable_path/../Frameworks" 
    // path/to/binary after compiling).
    // osx_minimum_system_version: A version string indicating the minimum 
    // Mac OS X version that the bundled app supports (e.g. "10.11"). 
    // If you are using this config field, you may also want have your 
    // build.rs script emit cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11 
    // (or whatever version number you want) to ensure that the compiled 
    // binary has the same minimum version.
}
