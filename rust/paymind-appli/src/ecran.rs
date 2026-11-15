//! Ce que la page de Paymind montre : l'état du kit d'écrans de la suite (format en tête de ui\kit.js).
//!
//! Depuis l'étape 3, l'accueil est le portefeuille de la maquette F (src\portefeuille.rs, dessiné par
//! ui\portefeuille.js à la place des tuiles du kit : bloc `accueil.perso`). Le kit garde l'en-tête, le bas, le
//! clavier et la manette, les sons et les pages : « Ajouter un abonnement » (bientôt, étape 4) et « Bientôt » (tous les
//! paiements des 30 prochains jours).
//!
//! Tous les textes vont au kit tels quels (il les échappe) : en français simple, sans tiret long ni demi-cadratin.

use paymind_moteur::{Abonnement, Date};
use serde_json::{Map, Value, json};

use crate::portefeuille::{self, PAGE_AJOUTER, PAGE_BIENTOT};

/// L'état de la page : le nom, la version et le logo de l'appli, l'accueil (le portefeuille de la liste ce jour-là)
/// et les pages. Le thème, les couleurs d'un pack et ses images s'ajoutent par l'habillage
/// ([`crate::Page::habiller`]).
pub fn etat(version: &str, liste: &[Abonnement], aujourdhui: Date) -> Value {
    let mut pages = Map::new();
    pages.insert(PAGE_AJOUTER.into(), portefeuille::page_ajouter());
    if let Some(p) = portefeuille::page_bientot(liste, aujourdhui) {
        pages.insert(PAGE_BIENTOT.into(), p);
    }
    json!({
        "app": { "nom": crate::NOM_APPLI, "version": version, "logo": "logo.svg" },
        "accueil": {
            // (aucune tuile : le portefeuille prend toute la place, et le haut n'a pas d'onglets)
            "tuiles": [],
            "ronds": [],
            "perso": portefeuille::portefeuille(liste, aujourdhui),
        },
        "pages": pages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use paymind_moteur::Rythme;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    #[test]
    fn le_portefeuille_et_ses_pages() {
        let auj = d(2026, 11, 15);
        let e = etat("0.1.0", &[], auj);
        assert_eq!(e["app"], json!({ "nom": "Paymind", "version": "0.1.0", "logo": "logo.svg" }));
        assert_eq!(e["accueil"]["tuiles"], json!([]));
        assert_eq!(e["accueil"]["perso"]["vide"], true);
        assert_eq!(e["accueil"]["perso"]["ajouter"], PAGE_AJOUTER);
        // La liste vide : « Ajouter » seulement
        assert_eq!(e["pages"].as_object().unwrap().keys().collect::<Vec<_>>(), [PAGE_AJOUTER]);
        let l = vec![Abonnement::nouveau("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18))];
        let e = etat("0.1.0", &l, auj);
        assert_eq!(e["accueil"]["perso"]["vide"], false);
        assert_eq!(e["pages"].as_object().unwrap().keys().collect::<Vec<_>>(), [PAGE_AJOUTER, PAGE_BIENTOT]);
        // (chaque page a ce que le kit attend : un titre, des blocs)
        for (_, p) in e["pages"].as_object().unwrap() {
            assert!(p["titre"].is_string() && p["blocs"].is_array());
        }
    }

    /// Le logo de l'état est bien livré avec les pages (ui\), et la page charge le portefeuille.
    #[test]
    fn le_logo_et_le_portefeuille_existent() {
        let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
        let e = etat("0.1.0", &[], d(2026, 11, 15));
        assert!(ui.join(e["app"]["logo"].as_str().unwrap()).is_file());
        let index = std::fs::read_to_string(ui.join("index.html")).unwrap();
        for f in ["portefeuille.css", "portefeuille.js"] {
            assert!(ui.join(f).is_file(), "{f}");
            assert!(index.contains(f), "index.html ne charge pas {f}");
        }
    }
}
