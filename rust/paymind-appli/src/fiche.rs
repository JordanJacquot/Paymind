//! La fiche modifiable (étape 5 du plan) : les gestes de la fiche de la maquette F, vérifiés ici (jamais confiance
//! dans la page), puis enregistrés dans abonnements.json ; la carte de Mindset (etat.json) suit aussitôt.
//!
//! Chaque geste passe par la commande `action` du kit (aucune nouvelle commande Tauri), avec pour valeur un objet
//! `{ id, ... }` qui a exactement ces champs :
//!
//! | geste             | valeur                         | ce qui change                                            |
//! |-------------------|--------------------------------|----------------------------------------------------------|
//! | `fiche-prix`      | `{ id, prix: "13,49" }`        | le prix (plus grand que zéro, 999 999,99 € au plus)      |
//! | `fiche-rythme`    | `{ id, rythme: "mois" }`       | semaine, mois, trimestre ou an                           |
//! | `fiche-date`      | `{ id, date: "2026-12-05" }`   | la date du prochain paiement, et le jour voulu (ancrage) |
//! | `fiche-auto`      | `{ id, oui: true }`            | le renouvellement automatique                            |
//! | `fiche-rappel`    | `{ id, rappel: "veille" }`     | jamais, jour-meme, veille, 3-jours, 7-jours              |
//! | `fiche-note`      | `{ id, note: "..." }`          | une note courte ([`NOTE_FICHE_MAX`] lettres au plus)     |
//! | `fiche-nom`       | `{ id, nom: "..." }`           | le nom d'un abonnement « à moi » seulement               |
//! | `fiche-pause`     | `{ id, oui: true }`            | en pause (oui) ou repris (non)                           |
//! | `fiche-renouvele` | `{ id }`                       | « C'est renouvelé » : la date passe au prochain paiement |
//! | `fiche-supprimer` | `{ id }`                       | l'abonnement retiré de la liste (la page a demandé       |
//! |                   |                                | « Supprimer Netflix ? » avant)                           |
//!
//! La date choisie va d'aujourd'hui à [`ANS_MAX`] ans plus tard ; elle devient aussi le jour voulu
//! ([`Abonnement::changer_date`] : choisir le 31 fait revenir les paiements au 31). Avant chaque changement, une date
//! gardée déjà passée d'un abonnement à renouvellement automatique est avancée au prochain paiement (le nouveau
//! rythme ou la nouvelle date partent alors du bon jour).
//!
//! Une liste qui ne peut pas être enregistrée (une version plus récente de Paymind) n'est jamais changée : le geste
//! est refusé avec une phrase simple, et la page reçoit de nouveau l'état d'avant.

use paymind_donnees::Donnees;
use paymind_moteur::abonnement::NOM_MAX;
use paymind_moteur::textes::{date_longue, euros_court, lire_prix};
use paymind_moteur::{Abonnement, Date, Rappel, Rythme, Statut, avancer, renouveler};
use serde_json::{Map, Value};

use crate::ajout::nom_propre;

pub const PRIX: &str = "fiche-prix";
pub const RYTHME: &str = "fiche-rythme";
pub const DATE: &str = "fiche-date";
pub const AUTO: &str = "fiche-auto";
pub const RAPPEL: &str = "fiche-rappel";
pub const NOTE: &str = "fiche-note";
pub const NOM: &str = "fiche-nom";
pub const PAUSE: &str = "fiche-pause";
pub const RENOUVELE: &str = "fiche-renouvele";
pub const SUPPRIMER: &str = "fiche-supprimer";
/// Tous les gestes de la fiche.
pub const GESTES: [&str; 10] = [PRIX, RYTHME, DATE, AUTO, RAPPEL, NOTE, NOM, PAUSE, RENOUVELE, SUPPRIMER];

/// La longueur la plus grande d'une note tapée dans la fiche (en caractères).
pub const NOTE_FICHE_MAX: usize = 200;
/// La date du prochain paiement se choisit d'aujourd'hui à 3 ans plus tard.
pub const ANS_MAX: i64 = 3;

/// Les refus, en mots simples (la bulle de l'écran).
pub const REFUS: &str = "Changement refusé.";
pub const ERREUR_PRIX: &str = "Écris un prix entre 0,01\u{a0}€ et 999\u{a0}999,99\u{a0}€, par exemple 12,50.";
pub const ABSENT: &str = "Cet abonnement n'est plus dans ta liste.";
pub const PAS_ENREGISTRE: &str = "Paymind n'a pas pu enregistrer ta liste. Réessaie dans un moment.";
pub const LISTE_PLUS_RECENTE: &str =
    "Ta liste a été enregistrée par une version plus récente de Paymind : elle ne peut pas être changée ici.";

/// Ce qu'un geste change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Changement {
    Prix(u64),
    Rythme(Rythme),
    Date(Date),
    Auto(bool),
    Rappel(Rappel),
    Note(String),
    Nom(String),
    Pause(bool),
    Renouvele,
    Supprimer,
}

/// Un geste de la fiche, lu et vérifié : l'abonnement visé et ce qui change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Geste {
    pub id: String,
    pub changement: Changement,
}

/// Ce geste est-il un geste de la fiche ?
pub fn est_geste(id: &str) -> bool {
    GESTES.contains(&id)
}

/// La date la plus lointaine qu'on peut choisir (3 ans après aujourd'hui).
pub fn date_max(aujourdhui: Date) -> Date {
    aujourdhui.plus_mois(ANS_MAX * 12, aujourdhui.jour()).unwrap_or(aujourdhui)
}

/// Un texte tapé dans la fiche (nom ou note), rendu propre comme un nom ([`nom_propre`]) et pas plus long que `max`.
fn texte_propre(v: &Value, max: usize, trop_long: String) -> Result<String, String> {
    let t = v.as_str().filter(|t| t.chars().count() <= max * 4).and_then(nom_propre).ok_or(REFUS)?;
    if t.chars().count() > max {
        return Err(trop_long);
    }
    Ok(t)
}

/// Lit le geste `geste` de la page et sa valeur, ce jour-là. Le refus est un texte simple pour la bulle de l'écran
/// (et le journal). L'abonnement lui-même est cherché ensuite ([`appliquer`]).
pub fn lire(geste: &str, valeur: &Value, aujourdhui: Date) -> Result<Geste, String> {
    let champ = match geste {
        PRIX => Some("prix"),
        RYTHME => Some("rythme"),
        DATE => Some("date"),
        AUTO | PAUSE => Some("oui"),
        RAPPEL => Some("rappel"),
        NOTE => Some("note"),
        NOM => Some("nom"),
        RENOUVELE | SUPPRIMER => None,
        _ => return Err(REFUS.into()),
    };
    let o: &Map<String, Value> = valeur.as_object().ok_or(REFUS)?;
    // (exactement « id » et le champ du geste : aucun champ en trop)
    let attendus = 1 + usize::from(champ.is_some());
    if o.len() != attendus || !o.contains_key("id") || champ.is_some_and(|c| !o.contains_key(c)) {
        return Err(REFUS.into());
    }
    let id = o["id"].as_str().filter(|i| crate::pont::id_valable(i)).ok_or(REFUS)?.to_string();
    let v = champ.map_or(&Value::Null, |c| &o[c]);
    let changement = match geste {
        PRIX => Changement::Prix(
            v.as_str().filter(|p| p.chars().count() <= 20).and_then(lire_prix).filter(|p| *p > 0).ok_or(ERREUR_PRIX)?,
        ),
        RYTHME => Changement::Rythme(v.as_str().and_then(Rythme::depuis_code).ok_or(REFUS)?),
        RAPPEL => Changement::Rappel(v.as_str().and_then(Rappel::depuis_code).ok_or(REFUS)?),
        AUTO => Changement::Auto(v.as_bool().ok_or(REFUS)?),
        PAUSE => Changement::Pause(v.as_bool().ok_or(REFUS)?),
        DATE => {
            let d = v.as_str().and_then(Date::lire_iso).ok_or("Choisis une date dans le calendrier.")?;
            if d < aujourdhui {
                return Err("Choisis une date à partir d'aujourd'hui.".into());
            }
            if d > date_max(aujourdhui) {
                return Err(format!("Choisis une date dans les {ANS_MAX} ans qui viennent."));
            }
            Changement::Date(d)
        }
        NOTE => Changement::Note(texte_propre(
            v,
            NOTE_FICHE_MAX,
            format!("Cette note est trop longue ({NOTE_FICHE_MAX} lettres au plus)."),
        )?),
        NOM => {
            let nom = texte_propre(v, NOM_MAX, format!("Ce nom est trop long ({NOM_MAX} lettres au plus)."))?;
            if nom.is_empty() {
                return Err("Écris le nom de l'abonnement.".into());
            }
            Changement::Nom(nom)
        }
        RENOUVELE => Changement::Renouvele,
        _ => Changement::Supprimer,
    };
    Ok(Geste { id, changement })
}

/// Ce que [`appliquer`] a fait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fait {
    /// La phrase de la bulle de l'écran (rien : l'écran montre déjà le changement)
    pub message: Option<String>,
    /// L'abonnement a été retiré de la liste
    pub supprime: bool,
}

/// La phrase de la bulle d'un changement fait.
fn message(c: &Changement, a: &Abonnement, aujourdhui: Date) -> Option<String> {
    let nom = &a.nom;
    Some(match c {
        Changement::Prix(p) => format!("Nouveau prix de {nom} : {} {}.", euros_court(*p), a.rythme.par_texte()),
        Changement::Date(d) => format!("Prochain paiement de {nom} : {}.", date_longue(*d, aujourdhui)),
        Changement::Note(n) if n.is_empty() => "Note retirée.".into(),
        Changement::Note(_) => "Note enregistrée.".into(),
        Changement::Nom(_) => format!("Nouveau nom enregistré : {nom}."),
        Changement::Pause(true) => format!("{nom} est en pause : il ne compte plus dans le total."),
        Changement::Pause(false) => format!("{nom} compte de nouveau dans le total."),
        Changement::Renouvele => {
            format!("{nom} est renouvelé : prochain paiement {}.", date_longue(a.date, aujourdhui))
        }
        Changement::Supprimer => format!("{nom} est retiré de tes cartes."),
        Changement::Rythme(_) | Changement::Auto(_) | Changement::Rappel(_) => return None,
    })
}

/// La ligne du journal d'un changement fait (sans le prix ni la note : seulement ce qui a changé).
fn ligne_journal(c: &Changement, a: &Abonnement) -> String {
    let quoi = match c {
        Changement::Prix(_) => "prix".to_string(),
        Changement::Rythme(r) => format!("rythme {}", r.code()),
        Changement::Date(d) => format!("prochain paiement le {}", d.iso()),
        Changement::Auto(o) => format!("renouvellement automatique {}", if *o { "oui" } else { "non" }),
        Changement::Rappel(r) => format!("rappel {}", r.code()),
        Changement::Note(_) => "note".to_string(),
        Changement::Nom(_) => "nom".to_string(),
        Changement::Pause(o) => (if *o { "en pause" } else { "repris" }).to_string(),
        Changement::Renouvele => format!("renouvelé, prochain paiement le {}", a.date.iso()),
        Changement::Supprimer => "supprimé".to_string(),
    };
    format!("Fiche de {} ({}) : {quoi}", a.nom, a.id)
}

/// Fait le geste sur la liste, l'enregistre (abonnements.json, l'ancienne version dans « .bak »), puis met à jour la
/// carte de Mindset (etat.json). Refusé ou pas enregistré : la liste revient comme avant, et le refus est un texte
/// simple pour la bulle de l'écran.
pub fn appliquer(d: &mut Donnees, g: &Geste, aujourdhui: Date) -> Result<Fait, String> {
    if d.abonnements.plus_recente() {
        return Err(LISTE_PLUS_RECENTE.into());
    }
    let i = d.abonnements.liste.iter().position(|a| a.id == g.id).ok_or(ABSENT)?;
    let avant = d.abonnements.liste[i].clone();
    let mut a = avant.clone();
    if g.changement != Changement::Supprimer {
        avancer(&mut a, aujourdhui);
    }
    match &g.changement {
        Changement::Prix(p) => a.prix = *p,
        Changement::Rythme(r) => a.changer_rythme(*r),
        Changement::Date(dt) => a.changer_date(*dt),
        Changement::Auto(o) => a.auto = *o,
        Changement::Rappel(r) => a.rappel = *r,
        Changement::Note(n) => a.note.clone_from(n),
        Changement::Nom(n) => {
            if a.catalogue.is_some() {
                return Err("Le nom d'un abonnement de la liste toute prête ne se change pas.".into());
            }
            a.nom.clone_from(n);
        }
        Changement::Pause(o) => a.pause = *o,
        Changement::Renouvele => {
            if a.statut(aujourdhui) != Statut::ARenouveler || !renouveler(&mut a, aujourdhui) {
                return Err("Cet abonnement n'est pas à renouveler.".into());
            }
        }
        Changement::Supprimer => {}
    }
    if let Some(p) = a.probleme() {
        d.journal().ecrire(&format!("Fiche de {} : changement refusé ({p})", avant.id));
        return Err(REFUS.into());
    }
    let supprime = g.changement == Changement::Supprimer;
    // (rien de changé : rien à écrire)
    if !supprime && a == avant {
        return Ok(Fait { message: None, supprime: false });
    }
    if supprime {
        d.abonnements.liste.remove(i);
    } else {
        d.abonnements.liste[i] = a.clone();
    }
    if let Err(e) = d.abonnements.sauver(&mut d.fichiers) {
        if supprime {
            d.abonnements.liste.insert(i, avant.clone());
        } else {
            d.abonnements.liste[i] = avant.clone();
        }
        d.journal().ecrire(&format!("Fiche de {} : changement non enregistré ({e})", avant.id));
        return Err(PAS_ENREGISTRE.into());
    }
    d.journal().ecrire(&ligne_journal(&g.changement, &a));
    d.ecrire_etat();
    Ok(Fait { message: message(&g.changement, &a, aujourdhui), supprime })
}

#[cfg(test)]
mod tests {
    use super::*;
    use paymind_donnees::{Moment, horloge_figee};
    use paymind_moteur::bilan;
    use serde_json::json;
    use std::path::PathBuf;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    fn auj() -> Date {
        d(2026, 11, 15)
    }

    /// Un dossier de données d'essai neuf sous le dossier temporaire, effacé à la fin (jamais les vraies données)
    struct Essai(PathBuf);
    impl Essai {
        fn new(nom: &str) -> Essai {
            let p = std::env::temp_dir().join(format!("paymind-fiche-{}-{nom}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Essai(p)
        }
        fn donnees(&self) -> Donnees {
            let h = horloge_figee(Moment::nouveau(2026, 11, 15, 10, 30, 0).unwrap());
            Donnees::ouvrir(&self.0, h, true)
        }
        fn lire(&self, f: &str) -> String {
            std::fs::read_to_string(self.0.join(f)).unwrap_or_default()
        }
    }
    impl Drop for Essai {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Une liste de trois abonnements enregistrée : Netflix (a1), Spotify (a2), Dofus à renouveler (a3), et un
    /// abonnement à moi (a4).
    fn preparer(e: &Essai) -> Donnees {
        let mut dn = e.donnees();
        let mut netflix = Abonnement::nouveau("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18));
        netflix.catalogue = Some("netflix".into());
        netflix.categorie = "films".into();
        let mut spotify = Abonnement::nouveau("a2", "Spotify", 1112, Rythme::Mois, d(2026, 11, 22));
        spotify.catalogue = Some("spotify".into());
        spotify.categorie = "musique".into();
        let mut dofus = Abonnement::nouveau("a3", "Dofus", 599, Rythme::Mois, d(2026, 11, 3));
        dofus.auto = false;
        dofus.categorie = "jeux".into();
        let club = Abonnement::nouveau("a4", "Club de foot", 4000, Rythme::Trimestre, d(2026, 12, 10));
        dn.abonnements.liste = vec![netflix, spotify, dofus, club];
        dn.abonnements.sauver(&mut dn.fichiers).unwrap();
        dn.ecrire_etat();
        dn
    }

    fn faire(dn: &mut Donnees, geste: &str, valeur: Value) -> Result<Fait, String> {
        let g = lire(geste, &valeur, auj())?;
        appliquer(dn, &g, auj())
    }

    fn total_mois(dn: &Donnees) -> u64 {
        bilan(&dn.abonnements.liste, auj()).par_mois
    }

    #[test]
    fn modifier_le_prix_change_le_total() {
        let e = Essai::new("prix");
        let mut dn = preparer(&e);
        // 13,49 + 11,12 + 40 / 3 (Dofus à renouveler ne compte pas)
        let avant = total_mois(&dn);
        assert_eq!(avant, 3794);
        assert!(e.lire("etat.json").contains("37,94\u{a0}€ par mois"));
        let f = faire(&mut dn, PRIX, json!({ "id": "a1", "prix": "15,99" })).unwrap();
        assert_eq!(f.message.as_deref(), Some("Nouveau prix de Netflix : 15,99\u{a0}€ par mois."));
        assert_eq!(total_mois(&dn), avant + 250);
        // Enregistré (relu depuis le disque) ; la carte de Mindset suit
        assert_eq!(e.donnees().abonnements.liste[0].prix, 1599);
        assert!(e.lire("etat.json").contains("40,44\u{a0}€ par mois"), "{}", e.lire("etat.json"));
        assert!(e.lire("abonnements.json.bak").contains("1349"));
        assert!(e.lire("journal.txt").contains("Fiche de Netflix (a1) : prix"));
        // Le point, les espaces et l'euro passent ; le plus haut prix aussi
        faire(&mut dn, PRIX, json!({ "id": "a1", "prix": " 9.5 € " })).unwrap();
        assert_eq!(dn.abonnements.liste[0].prix, 950);
        faire(&mut dn, PRIX, json!({ "id": "a1", "prix": "999 999,99" })).unwrap();
        assert_eq!(dn.abonnements.liste[0].prix, 99_999_999);
    }

    #[test]
    fn pause_ne_compte_plus_puis_reprise() {
        let e = Essai::new("pause");
        let mut dn = preparer(&e);
        let avant = total_mois(&dn);
        let f = faire(&mut dn, PAUSE, json!({ "id": "a1", "oui": true })).unwrap();
        assert_eq!(f.message.as_deref(), Some("Netflix est en pause : il ne compte plus dans le total."));
        assert_eq!(total_mois(&dn), avant - 1349);
        assert_eq!(dn.abonnements.liste[0].statut(auj()), Statut::EnPause);
        assert!(e.lire("etat.json").contains("1 en pause"), "{}", e.lire("etat.json"));
        assert!(e.donnees().abonnements.liste[0].pause);
        faire(&mut dn, PAUSE, json!({ "id": "a1", "oui": false })).unwrap();
        assert_eq!(total_mois(&dn), avant);
        assert!(!e.lire("etat.json").contains("en pause"));
    }

    #[test]
    fn supprimer_fait_disparaitre() {
        let e = Essai::new("supprimer");
        let mut dn = preparer(&e);
        let f = faire(&mut dn, SUPPRIMER, json!({ "id": "a2" })).unwrap();
        assert!(f.supprime);
        assert_eq!(f.message.as_deref(), Some("Spotify est retiré de tes cartes."));
        assert!(dn.abonnements.liste.iter().all(|a| a.id != "a2"));
        assert_eq!(e.donnees().abonnements.liste.len(), 3);
        assert!(!e.lire("abonnements.json").contains("Spotify"));
        assert!(e.lire("abonnements.json.bak").contains("Spotify"));
        let p = crate::portefeuille::portefeuille(&dn.abonnements.liste, auj());
        assert!(!p.to_string().contains("Spotify"));
        // Déjà supprimé : refusé simplement
        assert_eq!(faire(&mut dn, SUPPRIMER, json!({ "id": "a2" })).unwrap_err(), ABSENT);
    }

    #[test]
    fn renouvele_passe_a_la_date_suivante() {
        let e = Essai::new("renouvele");
        let mut dn = preparer(&e);
        let avant = total_mois(&dn);
        assert_eq!(dn.abonnements.liste[2].statut(auj()), Statut::ARenouveler);
        let f = faire(&mut dn, RENOUVELE, json!({ "id": "a3" })).unwrap();
        // (dû le 3 novembre : le 3 décembre, au même jour voulu ; il compte de nouveau)
        assert_eq!(dn.abonnements.liste[2].date, d(2026, 12, 3));
        assert_eq!(dn.abonnements.liste[2].statut(auj()), Statut::Actif);
        assert!(!dn.abonnements.liste[2].auto);
        assert_eq!(f.message.as_deref(), Some("Dofus est renouvelé : prochain paiement le jeudi 3 décembre."));
        assert_eq!(total_mois(&dn), avant + 599);
        assert_eq!(e.donnees().abonnements.liste[2].date, d(2026, 12, 3));
        assert!(!e.lire("etat.json").contains("à renouveler"));
        // Un abonnement qui n'est pas à renouveler : refusé, rien ne change
        assert_eq!(
            faire(&mut dn, RENOUVELE, json!({ "id": "a1" })).unwrap_err(),
            "Cet abonnement n'est pas à renouveler."
        );
        assert_eq!(dn.abonnements.liste[0].date, d(2026, 11, 18));
    }

    #[test]
    fn la_date_change_le_jour_voulu() {
        let e = Essai::new("date");
        let mut dn = preparer(&e);
        let f = faire(&mut dn, DATE, json!({ "id": "a1", "date": "2027-01-31" })).unwrap();
        assert_eq!(f.message.as_deref(), Some("Prochain paiement de Netflix : le dimanche 31 janvier 2027."));
        let a = &dn.abonnements.liste[0];
        assert_eq!((a.date, a.ancrage), (d(2027, 1, 31), 31));
        assert!(e.lire("abonnements.json").contains("\"ancrage\": 31"));
        // Un abonnement à renouveler qui reçoit une date à venir redevient actif
        faire(&mut dn, DATE, json!({ "id": "a3", "date": "2026-11-15" })).unwrap();
        assert_eq!(dn.abonnements.liste[2].statut(auj()), Statut::Actif);
        assert!(e.lire("etat.json").contains("Dofus aujourd'hui"), "{}", e.lire("etat.json"));
        // Les bornes : aujourd'hui et dans 3 ans passent ; hier et un jour de plus non
        assert!(lire(DATE, &json!({ "id": "a1", "date": "2029-11-15" }), auj()).is_ok());
        assert_eq!(
            lire(DATE, &json!({ "id": "a1", "date": "2029-11-16" }), auj()).unwrap_err(),
            "Choisis une date dans les 3 ans qui viennent."
        );
        assert_eq!(
            lire(DATE, &json!({ "id": "a1", "date": "2026-11-14" }), auj()).unwrap_err(),
            "Choisis une date à partir d'aujourd'hui."
        );
    }

    #[test]
    fn rythme_auto_rappel_note_nom() {
        let e = Essai::new("reste");
        let mut dn = preparer(&e);
        // Par an : 12 × 13,49 + 12 × 11,12 + 4 × 40 = 455,32 €
        assert_eq!(bilan(&dn.abonnements.liste, auj()).par_an, 45_532);
        assert_eq!(faire(&mut dn, RYTHME, json!({ "id": "a1", "rythme": "an" })).unwrap().message, None);
        assert_eq!(dn.abonnements.liste[0].rythme, Rythme::An);
        // Netflix 13,49 € par an au lieu de par mois : 11 × 13,49 € de moins par an
        assert_eq!(bilan(&dn.abonnements.liste, auj()).par_an, 45_532 - 11 * 1349);
        faire(&mut dn, AUTO, json!({ "id": "a1", "oui": false })).unwrap();
        assert!(!dn.abonnements.liste[0].auto);
        faire(&mut dn, RAPPEL, json!({ "id": "a1", "rappel": "7-jours" })).unwrap();
        assert_eq!(dn.abonnements.liste[0].rappel, Rappel::SeptJours);
        let f = faire(&mut dn, NOTE, json!({ "id": "a1", "note": "  Compte   de la famille \u{2014} Léa " })).unwrap();
        assert_eq!(f.message.as_deref(), Some("Note enregistrée."));
        assert_eq!(dn.abonnements.liste[0].note, "Compte de la famille - Léa");
        assert_eq!(
            faire(&mut dn, NOTE, json!({ "id": "a1", "note": "" })).unwrap().message.as_deref(),
            Some("Note retirée.")
        );
        let relu = e.donnees();
        assert_eq!(relu.abonnements.liste[0].rythme, Rythme::An);
        assert_eq!(relu.abonnements.liste[0].rappel, Rappel::SeptJours);
        assert!(!relu.abonnements.liste[0].auto);
        // Le nom : seulement un abonnement à moi
        faire(&mut dn, NOM, json!({ "id": "a4", "nom": "Club de foot des enfants" })).unwrap();
        assert_eq!(dn.abonnements.liste[3].nom, "Club de foot des enfants");
        assert_eq!(
            faire(&mut dn, NOM, json!({ "id": "a1", "nom": "Netflix 2" })).unwrap_err(),
            "Le nom d'un abonnement de la liste toute prête ne se change pas."
        );
        // Rien de changé : rien n'est écrit
        let avant = e.lire("abonnements.json");
        assert_eq!(faire(&mut dn, RAPPEL, json!({ "id": "a1", "rappel": "7-jours" })).unwrap().message, None);
        assert_eq!(e.lire("abonnements.json"), avant);
    }

    /// Un abonnement à renouvellement automatique dont la date gardée est passée : elle avance d'abord au prochain
    /// paiement, puis le nouveau rythme part de là.
    #[test]
    fn la_date_passee_avance_avant_le_changement() {
        let e = Essai::new("avance");
        let mut dn = preparer(&e);
        dn.abonnements.liste[1].date = d(2026, 9, 22);
        faire(&mut dn, RYTHME, json!({ "id": "a2", "rythme": "trimestre" })).unwrap();
        assert_eq!(dn.abonnements.liste[1].date, d(2026, 11, 22));
        assert_eq!(dn.abonnements.liste[1].rythme, Rythme::Trimestre);
    }

    /// Jamais confiance dans la page : chaque geste mal formé est refusé, et rien ne change.
    #[test]
    fn valeurs_fausses_refusees() {
        let e = Essai::new("refus");
        let mut dn = preparer(&e);
        let avant = e.lire("abonnements.json");
        for (geste, v) in [
            (PRIX, json!({ "id": "a1", "prix": "0" })),
            (PRIX, json!({ "id": "a1", "prix": "0,00" })),
            (PRIX, json!({ "id": "a1", "prix": "-5" })),
            (PRIX, json!({ "id": "a1", "prix": "1 000 000" })),
            (PRIX, json!({ "id": "a1", "prix": "12,345" })),
            (PRIX, json!({ "id": "a1", "prix": "douze" })),
            (PRIX, json!({ "id": "a1", "prix": "" })),
            (PRIX, json!({ "id": "a1", "prix": 12 })),
            (PRIX, json!({ "id": "a1", "prix": "1".repeat(30) })),
            (PRIX, json!({ "id": "a1" })),
            (PRIX, json!({ "id": "a1", "prix": "12", "en_plus": 1 })),
            (PRIX, json!({ "prix": "12" })),
            (PRIX, json!("12")),
            (PRIX, Value::Null),
            (PRIX, json!({ "id": "<b>", "prix": "12" })),
            (PRIX, json!({ "id": 1, "prix": "12" })),
            (RYTHME, json!({ "id": "a1", "rythme": "jour" })),
            (RYTHME, json!({ "id": "a1", "rythme": "Mois" })),
            (RAPPEL, json!({ "id": "a1", "rappel": "2-jours" })),
            (RAPPEL, json!({ "id": "a1", "rappel": 1 })),
            (AUTO, json!({ "id": "a1", "oui": "oui" })),
            (PAUSE, json!({ "id": "a1", "oui": 1 })),
            (DATE, json!({ "id": "a1", "date": "2026-02-30" })),
            (DATE, json!({ "id": "a1", "date": "2026-13-01" })),
            (DATE, json!({ "id": "a1", "date": "15/12/2026" })),
            (DATE, json!({ "id": "a1", "date": "2026-12-1" })),
            (DATE, json!({ "id": "a1", "date": "2025-12-01" })),
            (DATE, json!({ "id": "a1", "date": "2099-12-01" })),
            (NOTE, json!({ "id": "a1", "note": "x".repeat(NOTE_FICHE_MAX + 1) })),
            (NOTE, json!({ "id": "a1", "note": "a\u{0}b" })),
            (NOTE, json!({ "id": "a1", "note": ["x"] })),
            (NOM, json!({ "id": "a4", "nom": "   " })),
            (NOM, json!({ "id": "a4", "nom": "x".repeat(NOM_MAX + 1) })),
            (RENOUVELE, json!({ "id": "a3", "oui": true })),
            (SUPPRIMER, json!({ "id": "a1", "confirme": true })),
            (SUPPRIMER, json!(["a1"])),
            ("fiche-tout", json!({ "id": "a1" })),
        ] {
            assert!(lire(geste, &v, auj()).is_err(), "{geste} {v} accepté");
        }
        // Un id qui n'existe pas
        for geste in [PRIX, SUPPRIMER] {
            let v = if geste == PRIX { json!({ "id": "a99", "prix": "12" }) } else { json!({ "id": "a99" }) };
            assert_eq!(faire(&mut dn, geste, v).unwrap_err(), ABSENT);
        }
        // Les refus disent quoi faire, en mots simples
        assert_eq!(lire(PRIX, &json!({ "id": "a1", "prix": "abc" }), auj()).unwrap_err(), ERREUR_PRIX);
        assert_eq!(lire(PRIX, &json!({ "id": "a1", "prix": "0" }), auj()).unwrap_err(), ERREUR_PRIX);
        assert_eq!(lire(PRIX, &json!({ "id": "a1", "prix": "x", "y": 1 }), auj()).unwrap_err(), REFUS);
        // Une note de 200 lettres passe
        assert!(lire(NOTE, &json!({ "id": "a1", "note": "é".repeat(NOTE_FICHE_MAX) }), auj()).is_ok());
        // Rien n'a bougé
        assert_eq!(e.lire("abonnements.json"), avant);
        assert!(est_geste(PRIX) && est_geste(SUPPRIMER) && !est_geste("fiche-tout") && !est_geste("ajouter"));
    }

    /// Une liste d'une version plus récente de Paymind : aucun changement, une phrase simple.
    #[test]
    fn lecture_seule_si_format_futur() {
        let e = Essai::new("format-futur");
        let futur = r#"{ "format": 99, "abonnements": [ { "id": "a1", "nom": "Netflix", "prix": 1349,
            "rythme": "mois", "date": "2026-11-18" } ] }"#;
        std::fs::write(e.0.join("abonnements.json"), futur).unwrap();
        let mut dn = e.donnees();
        assert_eq!(dn.abonnements.liste.len(), 1);
        for (geste, v) in [
            (PRIX, json!({ "id": "a1", "prix": "15" })),
            (PAUSE, json!({ "id": "a1", "oui": true })),
            (SUPPRIMER, json!({ "id": "a1" })),
        ] {
            assert_eq!(faire(&mut dn, geste, v).unwrap_err(), LISTE_PLUS_RECENTE);
        }
        assert_eq!(dn.abonnements.liste[0].prix, 1349);
        assert_eq!(std::fs::read_to_string(e.0.join("abonnements.json")).unwrap(), futur);
    }

    /// Un fichier qui ne peut pas être enregistré (gardé ouvert par un autre programme, par exemple) : la liste revient
    /// comme avant.
    #[test]
    fn pas_enregistre_la_liste_revient() {
        let e = Essai::new("pas-enregistre");
        let mut dn = preparer(&e);
        // (abonnements.json remplacé par un dossier : l'écriture échoue)
        std::fs::remove_file(e.0.join("abonnements.json")).unwrap();
        std::fs::create_dir(e.0.join("abonnements.json")).unwrap();
        assert_eq!(faire(&mut dn, PRIX, json!({ "id": "a1", "prix": "20" })).unwrap_err(), PAS_ENREGISTRE);
        assert_eq!(dn.abonnements.liste[0].prix, 1349);
        assert_eq!(faire(&mut dn, SUPPRIMER, json!({ "id": "a1" })).unwrap_err(), PAS_ENREGISTRE);
        assert_eq!(dn.abonnements.liste.len(), 4);
        assert_eq!(dn.abonnements.liste[0].id, "a1");
    }
}
