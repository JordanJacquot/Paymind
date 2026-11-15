//! La carte vivante de Paymind dans l'accueil de Mindset (etat.json, format commun du contrat, partie 5). Repris de la
//! carte de Mindcare (`EtatMindset`) : écrite à côté puis remplacée d'un coup, et seulement quand elle change.
//!
//! etat.json : { Version, Titre, Scores : [ { Nom, Valeur 0 à 100 } ] (2 au plus), Lignes : [ { Texte, Etat } ]
//! (3 au plus), Maj }. Etat : ok (vert), info (blanc), attention (orange), urgent (rouge). Mindset coupe chaque ligne à
//! 80 caractères.
//!
//! Depuis l'étape 2, la carte vient du calcul des comptes (paymind-moteur) : le total du mois, le prochain paiement et
//! le nombre d'abonnements ; « Aucun abonnement pour l'instant » quand la liste est vide.

use std::path::{Path, PathBuf};

use paymind_moteur::textes::{euros_court, nombre_abonnements, texte_prochain};
use paymind_moteur::{Abonnement, Date, bilan};
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

/// La ligne quand aucun abonnement actif n'a de paiement à venir.
pub const AUCUN_PAIEMENT: &str = "Aucun paiement à venir";

/// Un texte coupé pour tenir dans une ligne de la carte (« ... » à la fin s'il est coupé).
fn tenir(t: String) -> String {
    if longueur(&t) <= LIGNE_MAX {
        return t;
    }
    let mut s = String::new();
    for c in t.chars() {
        if longueur(&s) + c.len_utf16() + 3 > LIGNE_MAX {
            break;
        }
        s.push(c);
    }
    s.trim_end().to_string() + "..."
}

fn ligne(texte: String, etat: &str) -> Ligne {
    Ligne { texte: tenir(texte), etat: etat.to_string() }
}

/// Ce que la carte montre ce jour-là : « 86,97 € par mois », le prochain paiement (« Netflix dans 3 jours »), le nombre
/// d'abonnements ; « Aucun abonnement pour l'instant » quand la liste est vide.
pub fn etat_carte(liste: &[Abonnement], aujourdhui: Date) -> Carte {
    let mut c = Carte { titre: TITRE.to_string(), scores: Vec::new(), lignes: Vec::new() };
    if liste.is_empty() {
        c.lignes.push(ligne(AUCUN.to_string(), "info"));
        return c;
    }
    let b = bilan(liste, aujourdhui);
    c.lignes.push(ligne(format!("{} par mois", euros_court(b.par_mois)), "ok"));
    c.lignes.push(match &b.prochain {
        // (aujourd'hui ou demain : en orange)
        Some(p) => ligne(texte_prochain(p, aujourdhui), if p.dans_jours <= 1 { "attention" } else { "info" }),
        None => ligne(AUCUN_PAIEMENT.to_string(), "info"),
    });
    let mut n = nombre_abonnements(b.actifs);
    if b.en_pause > 0 {
        n.push_str(&format!(", {} en pause", b.en_pause));
    }
    if b.a_renouveler > 0 {
        n.push_str(&format!(", {} à renouveler", b.a_renouveler));
    }
    c.lignes.push(ligne(n, if b.a_renouveler > 0 { "attention" } else { "info" }));
    c
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

    use paymind_moteur::Rythme;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    fn l(texte: &str, etat: &str) -> Ligne {
        Ligne { texte: texte.into(), etat: etat.into() }
    }

    fn dans_les_limites(c: &Carte) {
        assert_eq!(c.titre, "Mes abonnements");
        assert!(c.scores.is_empty() && c.lignes.len() <= 3);
        for l in &c.lignes {
            assert!(longueur(&l.texte) <= LIGNE_MAX, "{}", l.texte);
            // (aucun tiret long ni demi-cadratin : règle de la suite)
            assert!(!l.texte.contains(['\u{2013}', '\u{2014}']));
        }
    }

    #[test]
    fn la_carte_sans_abonnement() {
        let c = etat_carte(&[], d(2026, 11, 15));
        assert_eq!(c.lignes, vec![l("Aucun abonnement pour l'instant", "info")]);
        dans_les_limites(&c);
    }

    #[test]
    fn la_carte_avec_des_abonnements() {
        let auj = d(2026, 11, 15);
        let mut liste = vec![
            Abonnement::nouveau("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18)),
            Abonnement::nouveau("a2", "Assurance", 54_000, Rythme::An, d(2027, 1, 9)),
            Abonnement::nouveau("a3", "Spotify", 1112, Rythme::Mois, d(2026, 11, 16)),
            Abonnement::nouveau("a4", "Salle", 2999, Rythme::Mois, d(2026, 11, 1)),
        ];
        liste[2].pause = true;
        liste[3].auto = false;
        let c = etat_carte(&liste, auj);
        assert_eq!(
            c.lignes,
            vec![
                l("58,49\u{a0}€ par mois", "ok"),
                l("Netflix dans 3 jours", "info"),
                l("2 abonnements, 1 en pause, 1 à renouveler", "attention")
            ]
        );
        dans_les_limites(&c);
        // demain : en orange
        let c = etat_carte(&liste[..1], d(2026, 11, 17));
        assert_eq!(c.lignes[1], l("Netflix demain", "attention"));
        assert_eq!(c.lignes[2], l("1 abonnement", "info"));
        // tout en pause : rien à venir
        let c = etat_carte(&liste[2..3], auj);
        assert_eq!(
            c.lignes,
            vec![l("0\u{a0}€ par mois", "ok"), l(AUCUN_PAIEMENT, "info"), l("0 abonnement, 1 en pause", "info")]
        );
    }

    #[test]
    fn un_nom_tres_long_est_coupe() {
        let nom = "Abonnement au nom vraiment très très long ".repeat(4);
        let c = etat_carte(&[Abonnement::nouveau("a1", &nom, 100, Rythme::Mois, d(2026, 11, 20))], d(2026, 11, 15));
        dans_les_limites(&c);
        assert!(c.lignes[1].texte.ends_with("...") && c.lignes[1].texte.starts_with("Abonnement au nom"));
    }
}
