use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        let icon_path = Path::new("assets/icon.ico");
        if icon_path.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            res.set("ProductName", "llmLab");
            res.set("FileDescription", "llmLab — Local AI TUI Runtime");
            res.set("OriginalFilename", "llmLab.exe");
            res.set("LegalCopyright", "Copyright (c) 2026 llmLab");
            if let Err(err) = res.compile() {
                panic!("Failed to embed Windows icon resource: {err}");
            }
        } else {
            println!("cargo:warning=assets/icon.ico not found; run scripts/make_icon.py before release build.");
        }
    }

    #[cfg(not(windows))]
    {
        let _ = Path::new("assets/icon.ico");
    }
}
