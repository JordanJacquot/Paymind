//! Paymind en Rust : la fenêtre de Paymind dans Mindset et son programme. Repris de mindcare-appli : une page qui suit
//! le thème, la vie avec Mindset, depuis l'étape 3 le portefeuille de la maquette F, depuis l'étape 4 la liste
//! toute prête, et depuis l'étape 5 la fiche modifiable.
//!
//! - [`fenetre`] : la fenêtre « principale », créée dans le code (jamais dans tauri.conf.json), cachée et sans
//!   prendre la main, titre exact « Paymind », dossier WebView2 et options du navigateur de suite-commun
//!   (docs\regles-tauri.md du dépôt Mindset, règle 7), navigation fermée, aucune nouvelle fenêtre ;
//! - [`commandes`] : les trois seules commandes que la page peut appeler (`suivre_page`, `noter_visibilite`,
//!   `action`), déclarées dans build.rs et autorisées à la seule fenêtre « principale » (capabilities\principale.json) ;
//! - [`ecran`] : ce que la page montre (l'état du kit d'écrans de la suite) ;
//! - [`portefeuille`] : l'accueil de la maquette F (total, « Bientôt », cartes par catégorie et leurs fiches),
//!   préparé d'après la liste des abonnements et paymind-moteur, dessiné par ui\portefeuille.js ;
//! - [`fiche`] : les gestes de la fiche (prix, rythme, date, renouvellement, rappel, note, pause, « C'est
//!   renouvelé », suppression), vérifiés et enregistrés ;
//! - [`catalogue`] : la liste toute prête (catalogue\catalogue.json rangé dans l'exe, vérifié ; ses logos dans
//!   ui\logos), la recherche sans accents, les logos des cartes ;
//! - [`ajout`] : la liste toute prête à l'écran, « Un abonnement à moi », l'ajout vérifié et enregistré ;
//! - [`pont`] : les messages du kit vérifiés, l'habillage du thème, le retour à la console de Mindset ;
//! - [`pack`] : la réponse du protocole `pack` (les images et polices du thème, règle 6), par suite-commun ;
//! - [`preparer`] et [`contexte`] : ce que le programme et les tests donnent à Tauri ;
//! - `programme` : le programme Mindset.exe tout branché (démarrage, verrou, fenêtre, surveillances, fin) ;
//! - `windows` : le seul module où le code « unsafe » est permis (verrou, signal, fenêtre, registre, WebView2) ;
//! - `essai` et `png` : la construction d'essai seulement (option cargo `essai`, jamais livrée).
//!
//! Le programme (src\main.rs) appelle `programme::lancer` ; les tests (tests\permissions.rs) font tourner la
//! fenêtre avec le faux Windows de Tauri (option « test ») : aucune vraie fenêtre, aucune page chargée.

#![deny(unsafe_code)]

pub mod ajout;
pub mod catalogue;
pub mod commandes;
pub mod ecran;
#[cfg(feature = "essai")]
pub mod essai;
pub mod fenetre;
pub mod fiche;
pub mod pack;
#[cfg(any(feature = "essai", test))]
pub mod png;
pub mod pont;
pub mod portefeuille;
#[cfg(windows)]
pub mod programme;
#[cfg(windows)]
pub mod windows;

pub use commandes::Page;

/// Le nom de l'appli : titre de sa fenêtre et de sa page, nom de son dossier WebView2 (`WebView2-Tauri\Paymind`)
pub const NOM_APPLI: &str = "Paymind";
/// La version de l'appli (celle de son Cargo.toml, la même que version.json et la fiche de version de l'exe)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Ajoute à Tauri la page partagée et les commandes de l'appli (les seules que la page peut appeler, chacune avec
/// sa permission dans capabilities\principale.json).
pub fn preparer<R: tauri::Runtime>(constructeur: tauri::Builder<R>, page: Page) -> tauri::Builder<R> {
    constructeur.manage(page).invoke_handler(tauri::generate_handler![
        commandes::suivre_page,
        commandes::noter_visibilite,
        commandes::action
    ])
}

/// La configuration de Tauri lue à la construction (tauri.conf.json, capabilities\, les pages de ui\ rangées dans
/// l'exe, avec leurs empreintes ajoutées à la CSP).
pub fn contexte<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}
