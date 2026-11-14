//! La carte vivante de Paymind dans l'accueil de Mindset (etat.json, format commun du contrat, partie 5). Repris de la
//! carte de Mindcare (`EtatMindset`) : écrite à côté puis remplacée d'un coup, et seulement quand elle change.
//!
//! etat.json : { Version, Titre, Scores : [ { Nom, Valeur 0 à 100 } ] (2 au plus), Lignes : [ { Texte, Etat } ]
//! (3 au plus), Maj }. Etat : ok (vert), info (blanc), attention (orange), urgent (rouge). Mindset coupe chaque ligne à
//! 80 caractères.
//!
//! Étape 1 : pas encore d'abonnements, la carte le dit. Le total du mois et le prochain paiement viendront à l'étape 6.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::fichiers::Fichiers;
use crate::valeurs::egal;

/// La longueur d'une ligne de la carte (en caractères .NET : unités UTF-16).
pub const LIGNE_MAX: usize = 80;
/// Le titre de la carte.
pub const TITRE: &str = "Mes abonnements";
/// La ligne quand il n'y a encore aucun abonnement.
pub const AUCUN: &str = "Aucun abonnement pour l'instant";

/// La longueur d'un texte comme `.Length` de .NET.
pub fn longueur(t: &str) -> usize {
    t.encode_utf16().count()
}

/// Une ligne de la carte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub texte: String,
    /// ok, info, attention, urgent
    pub etat: String,
}

/// La carte de Mindset.
#[derive(Debug, Clone, PartialEq)]
pub struct Carte {
    pub titre: String,
    pub scores: Vec<Value>,
    pub lignes: Vec<Ligne>,
}

/// Ce que la carte montre (étape 1 : aucun abonnement).
pub fn etat_carte() -> Carte {
    Carte {
        titre: TITRE.to_string(),
        scores: Vec::new(),
        lignes: vec![Ligne { texte: AUCUN.to_string(), etat: "info".to_string() }],
    }
}

/// L'écriture de etat.json (`Write-EtatMindset`) : seulement quand la carte change.
#[derive(Debug, Clone)]
pub struct EtatMindset {
    fichier: PathBuf,
    /// (la dernière carte écrite : rien de neuf, pas d'écriture)
    ecrit: Option<String>,
}

impl EtatMindset {
    pub fn nouveau(fichier: &Path) -> EtatMindset {
        EtatMindset { fichier: fichier.to_path_buf(), ecrit: None }
    }

    pub fn fichier(&self) -> &Path {
        &self.fichier
    }

    /// Écrit la carte dans etat.json si elle a changé depuis la dernière écriture (UTF-8 sans BOM, à côté puis
    /// remplacé d'un coup).
    pub fn ecrire(&mut self, fichiers: &mut Fichiers, carte: &Carte) {
        let mut etat = serde_json::Map::new();
        etat.insert("Version".into(), json!(1));
        etat.insert("Titre".into(), json!(carte.titre));
        etat.insert("Scores".into(), Value::Array(carte.scores.iter().take(2).cloned().collect()));
        etat.insert(
            "Lignes".into(),
            Value::Array(carte.lignes.iter().take(3).map(|l| json!({ "Texte": l.texte, "Etat": l.etat })).collect()),
        );
        let cle = Value::Object(etat.clone()).to_string();
        // (comparé comme `-eq` de PowerShell : une carte qui ne change que de majuscules n'est pas réécrite)
        if self.ecrit.as_deref().is_some_and(|e| egal(e, &cle)) {
            return;
        }
        etat.insert("Maj".into(), json!(fichiers.journal.maintenant().iso()));
        match fichiers.sauver(&self.fichier, &Value::Object(etat)) {
            Ok(()) => self.ecrit = Some(cle),
            Err(e) => fichiers.journal.ecrire(&format!("etat.json pour Mindset : {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_carte_tient_dans_les_limites_de_mindset() {
        let c = etat_carte();
        assert_eq!(c.titre, "Mes abonnements");
        assert_eq!(c.lignes, vec![Ligne { texte: "Aucun abonnement pour l'instant".into(), etat: "info".into() }]);
        assert!(c.scores.is_empty());
        for l in &c.lignes {
            assert!(longueur(&l.texte) <= LIGNE_MAX);
            // (aucun tiret long ni demi-cadratin : règle de la suite)
            assert!(!l.texte.contains(['\u{2013}', '\u{2014}']));
        }
    }
}
