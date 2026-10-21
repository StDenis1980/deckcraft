//! Windows only: embed the app icon and version info (VERSIONINFO) into `deckcraft.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `DECKCRAFT_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/deckcraft.ico");
    println!("cargo:rerun-if-env-changed=DECKCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=DECKCRAFT_BRAND_NAME");
    println!("cargo:rerun-if-env-changed=DECKCRAFT_BRAND_PUBLISHER");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // A rebranded build (crates/brand) carries its own name and publisher.
    let brand = |var: &str| std::env::var(var).ok().filter(|v| !v.is_empty());
    let name = brand("DECKCRAFT_BRAND_NAME").unwrap_or_else(|| "DeckCraft".into());
    let company = brand("DECKCRAFT_BRAND_PUBLISHER").unwrap_or_else(|| "Learning Machines LLC".into());
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/deckcraft.ico")
        .set("ProductName", &name)
        .set("FileDescription", &format!("{name} presentations"))
        .set("CompanyName", &company)
        .set("LegalCopyright", "Copyright (c) the DeckCraft authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "deckcraft.exe")
        .set("InternalName", "deckcraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("DECKCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=deckcraft.exe built without icon/version resources: {e}");
    }
}
