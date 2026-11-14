//! Ce que la page de Paymind montre : l'état du kit d'écrans de la suite (format en tête de ui\kit.js). Étape 1 : une
//! seule tuile « Mes abonnements » et sa page, qui disent que les abonnements arrivent bientôt. Le portefeuille de la
//! maquette F viendra aux étapes 3 à 5.
//!
//! Tous les textes vont au kit tels quels (il les échappe) : en français simple, sans tiret long ni demi-cadratin.

use serde_json::{Value, json};

/// L'id de la tuile et de sa page.
pub const ABONNEMENTS: &str = "abonnements";
/// La phrase de l'accueil et de la page, tant qu'il n'y a rien d'autre à montrer.
pub const BIENTOT: &str = "Tes abonnements arriveront ici bientôt.";

/// L'état de la page : le nom, la version et le logo de l'appli, l'accueil (une tuile) et sa page. Le thème, les
/// couleurs d'un pack et ses images s'ajoutent par l'habillage ([`crate::Page::habiller`]).
pub fn etat(version: &str) -> Value {
    json!({
        "app": { "nom": crate::NOM_APPLI, "version": version, "logo": "logo.svg" },
        "accueil": {
            "tuiles": [
                { "id": ABONNEMENTS, "nom": "Mes abonnements", "icone": "euro", "sous": BIENTOT, "couleur": "P" },
            ],
            "ronds": [],
        },
        "pages": {
            ABONNEMENTS: {
                "titre": "Mes abonnements",
                "sous": BIENTOT,
                "icone": "euro",
                "couleur": "P",
                "module": {
                    "icone": "euro",
                    "couleur": "P",
                    "titre": "Bientôt",
                    "texte": "Paymind fera le compte de tes abonnements et te préviendra avant chaque paiement.",
                },
                "blocs": [
                    { "type": "texte", "texte": BIENTOT },
                    { "type": "note", "texte": "Tout reste sur ce PC : rien n'est envoyé sur Internet." },
                ],
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tous les textes d'une valeur (récursivement).
    fn textes(v: &Value, l: &mut Vec<String>) {
        match v {
            Value::String(s) => l.push(s.clone()),
            Value::Array(a) => a.iter().for_each(|x| textes(x, l)),
            Value::Object(o) => o.values().for_each(|x| textes(x, l)),
            _ => {}
        }
    }

    #[test]
    fn une_tuile_et_sa_page() {
        let e = etat("0.1.0");
        assert_eq!(e["app"], json!({ "nom": "Paymind", "version": "0.1.0", "logo": "logo.svg" }));
        let tuiles = e["accueil"]["tuiles"].as_array().unwrap();
        assert_eq!(tuiles.len(), 1);
        assert_eq!(tuiles[0]["id"], ABONNEMENTS);
        assert_eq!(e["pages"][ABONNEMENTS]["blocs"][0]["texte"], BIENTOT);
    }

    /// Règle de la suite : aucun tiret long ni demi-cadratin, aucun HTML (le kit écrit les textes tels quels).
    #[test]
    fn textes_simples() {
        let mut l = Vec::new();
        textes(&etat("0.1.0"), &mut l);
        for t in l {
            assert!(!t.contains(['\u{2013}', '\u{2014}', '<', '>']), "{t}");
        }
    }

    /// Le logo de l'état est bien livré avec les pages (ui\).
    #[test]
    fn le_logo_existe() {
        let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
        assert!(ui.join(etat("0.1.0")["app"]["logo"].as_str().unwrap()).is_file());
    }
}
