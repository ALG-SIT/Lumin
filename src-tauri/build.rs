use std::{env, fs, path::PathBuf};

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=LUMIN_TEST_MANIFEST");

    // Tauri's default `common-controls-v6` feature links TaskDialogIndirect
    // from comctl32 v6. Cargo test executables do not receive the app manifest
    // generated for the packaged app, so embed the activation manifest directly
    // into this package's Windows MSVC test binaries.
    if env::var("LUMIN_TEST_MANIFEST").as_deref() == Ok("true")
        && env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
        let manifest = out_dir.join("windows-test-manifest.xml");
        fs::write(
            &manifest,
            r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
</assembly>
"#,
        )
        .expect("write Windows test manifest");

        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
}
