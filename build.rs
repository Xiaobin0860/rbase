fn main() {
    println!("cargo:rerun-if-changed=shared.proto");
    println!("cargo:rerun-if-changed=build.rs");

    prost_build::Config::new()
        .out_dir("src")
        .type_attribute(".", "#[derive(PartialOrd)]")
        .compile_protos(&["shared.proto"], &["."])
        .expect("prost build failed!");

    std::process::Command::new("cargo")
        .args(["fmt"])
        .status()
        .expect("cargo fmt failed!");
}
