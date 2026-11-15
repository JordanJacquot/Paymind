//! Construction de Mindset.exe (le programme de Paymind) par tauri-build : des fichiers lus, et seulement les logos
//! de la liste toute prête recopiés dans les pages.
//!
//! - le manifeste des commandes de l'appli (docs\regles-tauri.md du dépôt Mindset, règle 2) : sans lui, Tauri
//!   laisserait toute page locale appeler toutes les commandes ; avec lui, chaque commande demande sa permission
//!   `allow-<commande>`, donnée fenêtre par fenêtre dans capabilities\ ;
//! - l'icône de Paymind (icons\Paymind.ico, faite par outils\icone.ps1 : une carte bancaire verte) ;
//! - la fiche de version : tauri-build écrit productName (« Mindset », tauri.conf.json) comme description de l'exe,
//!   ce qu'affiche le Gestionnaire des tâches (contrat-appli, partie 4), et la version du Cargo.toml ;
//! - le runtime C rangé dans l'exe (réglage par défaut de tauri-build) : aucune VCRUNTIME140.dll à installer ;
//! - les logos de la liste toute prête, recopiés de catalogue\logos dans ui\logos (rangés dans l'exe avec les pages).

use std::path::Path;

/// Les logos de la liste toute prête (catalogue\logos du dépôt, la seule source) recopiés dans ui\logos, que
/// tauri-build range ensuite dans l'exe avec les pages : rien n'est jamais téléchargé. Un fichier n'est réécrit que
/// s'il a changé ; un fichier de ui\logos qui n'est plus dans catalogue\logos est retiré. ui\logos n'est pas suivi
/// par git (rust\.gitignore).
fn recopier_logos() {
    let source = Path::new("../../catalogue/logos");
    let cible = Path::new("ui/logos");
    println!("cargo:rerun-if-changed=../../catalogue/logos");
    std::fs::create_dir_all(cible).expect("ui\\logos");
    let mut gardes = Vec::new();
    for e in std::fs::read_dir(source).expect("catalogue\\logos introuvable") {
        let e = e.expect("catalogue\\logos");
        if !e.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        let nom = e.file_name();
        let octets = std::fs::read(e.path()).expect("logo illisible");
        let dest = cible.join(&nom);
        if std::fs::read(&dest).ok().as_deref() != Some(octets.as_slice()) {
            std::fs::write(&dest, &octets).expect("logo non recopié");
        }
        gardes.push(nom);
    }
    for e in std::fs::read_dir(cible).expect("ui\\logos").flatten() {
        if !gardes.contains(&e.file_name()) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=icons/Paymind.ico");
    recopier_logos();
    let windows = tauri_build::WindowsAttributes::new().window_icon_path("icons/Paymind.ico");
    // Les commandes de src\commandes.rs, toutes ici (verifier-tauri.ps1 compare les deux listes)
    let attributs = tauri_build::Attributes::new()
        .windows_attributes(windows)
        .app_manifest(tauri_build::AppManifest::new().commands(&["suivre_page", "noter_visibilite", "action"]));
    tauri_build::try_build(attributs).expect("construction de Paymind (tauri-build)");
    // Les exe des tests (tests\) : tauri-build ne donne son manifeste qu'au programme. Sans les contrôles modernes de
    // Windows (comctl32 version 6, option « common-controls-v6 »), ils ne démarrent pas (STATUS_ENTRYPOINT_NOT_FOUND).
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' \
             version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
