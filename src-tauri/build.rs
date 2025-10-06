use std::{env, path::Path};

fn main() {
    println!("cargo:rustc-env=SQLX_SQLITE_HAS_CODEC=1");

    if cfg!(target_os = "windows") {
        let lib_dir = env::var("OPENSSL_LIB_DIR").ok().or_else(|| {
            let default = r"C:\Program Files\OpenSSL-Win64\lib\VC\x64\MD";
            if Path::new(default).exists() {
                Some(default.to_string())
            } else {
                None
            }
        });

        if let Some(path) = lib_dir {
            println!("cargo:rustc-link-search=native={path}");
        }
    }

    tauri_build::build();
}
