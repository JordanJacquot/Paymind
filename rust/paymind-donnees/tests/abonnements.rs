//! abonnements.json sur un dossier d'essai sous %TEMP% (jamais les vraies données) : aller-retour, fichier absent,
//! copie de secours, fichier abîmé, JSON qui n'est pas une liste, abonnement illisible, fichier d'une version future,
//! et la carte de Mindset branchée sur la liste.

use std::path::{Path, PathBuf};

use paymind_donnees::abonnements::{self, Abonnements, FICHIER, FORMAT, liste_json};
use paymind_donnees::{Donnees, Fichiers, Journal, Moment, horloge_figee};
use paymind_moteur::{Abonnement, Date, Rappel, Rythme};
use serde_json::{Value, json};

/// Un dossier d'essai neuf sous %TEMP%, effacé à la fin.
struct Essai(PathBuf);

impl Essai {
    fn nouveau(nom: &str) -> Essai {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("PaymindEssai-abonnements-{}-{nom}-{n}", std::process::id()));
        assert!(d.is_absolute());
        std::fs::create_dir_all(&d).unwrap();
        Essai(d)
    }

    fn fichier(&self) -> PathBuf {
        self.0.join(FICHIER)
    }

    fn fichiers(&self) -> Fichiers {
        let h = horloge_figee(Moment::nouveau(2026, 11, 15, 10, 30, 0).unwrap());
        Fichiers::nouveau(Journal::ouvrir(&self.0.join("journal.txt"), h))
    }

    fn journal(&self) -> String {
        std::fs::read_to_string(self.0.join("journal.txt")).unwrap_or_default()
    }

    /// Les noms des fichiers du dossier, triés.
    fn noms(&self) -> Vec<String> {
        let mut l: Vec<String> = std::fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        l.sort();
        l
    }
}

impl Drop for Essai {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn d(a: i32, m: u32, j: u32) -> Date {
    Date::nouvelle(a, m, j).unwrap()
}

fn exemple() -> Vec<Abonnement> {
    let mut n = Abonnement::nouveau("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 30));
    n.catalogue = Some("netflix".into());
    n.categorie = "video".into();
    n.ancrage = 31;
    let mut s = Abonnement::nouveau("a2", "Salle de sport « Forme »", 2999, Rythme::Trimestre, d(2027, 2, 28));
    s.auto = false;
    s.rappel = Rappel::SeptJours;
    s.pause = true;
    s.note = "Résilier avant juin\nCode client 42".into();
    vec![n, s]
}

fn charger(e: &Essai) -> (Abonnements, Fichiers) {
    let mut f = e.fichiers();
    let a = Abonnements::charger(&mut f, &e.fichier());
    (a, f)
}

fn ecrire(p: &Path, t: &str) {
    std::fs::write(p, t.as_bytes()).unwrap();
}

#[test]
fn fichier_absent_liste_vide() {
    let e = Essai::nouveau("absent");
    let (a, _) = charger(&e);
    assert!(a.liste.is_empty());
    assert!(a.lecture_seule().is_none());
    // (rien n'est créé à la lecture)
    assert!(!e.fichier().exists());
}

#[test]
fn aller_retour_et_copie_de_secours() {
    let e = Essai::nouveau("aller-retour");
    let (mut a, mut f) = charger(&e);
    a.liste = exemple();
    a.sauver(&mut f).unwrap();
    let j: Value = serde_json::from_str(&std::fs::read_to_string(e.fichier()).unwrap()).unwrap();
    assert_eq!(j["format"], json!(FORMAT));
    assert_eq!(j["abonnements"][0]["prix"], json!(1349));
    assert_eq!(j["abonnements"][0]["date"], json!("2026-11-30"));
    assert_eq!(j["abonnements"][0]["ancrage"], json!(31));
    assert_eq!(j["abonnements"][1]["catalogue"], Value::Null);
    assert_eq!(j["abonnements"][1]["rappel"], json!("7-jours"));
    let (b, _) = charger(&e);
    assert_eq!(b.liste, exemple());
    // Deuxième enregistrement : la version précédente reste dans .bak
    let mut a = b;
    a.liste.truncate(1);
    a.sauver(&mut f).unwrap();
    let bak = Abonnements::charger(&mut e.fichiers(), &e.0.join(format!("{FICHIER}.bak")));
    assert_eq!(bak.liste, exemple());
    assert_eq!(charger(&e).0.liste.len(), 1);
    // (rien d'autre : ni fichier temporaire oublié, ni ligne au journal)
    assert_eq!(e.noms(), vec!["abonnements.json", "abonnements.json.bak"]);
}

#[test]
fn fichier_abime_repris_de_sa_copie_sans_rien_effacer() {
    let e = Essai::nouveau("abime");
    let (mut a, mut f) = charger(&e);
    a.liste = exemple();
    a.sauver(&mut f).unwrap();
    a.sauver(&mut f).unwrap(); // (le .bak contient la liste)
    ecrire(&e.fichier(), "{ \"format\": 1, \"abonnements\": [ { \"id\": \"a1\", "); // coupé en pleine écriture
    let (b, _) = charger(&e);
    assert_eq!(b.liste, exemple());
    let noms = e.noms();
    let abime: Vec<_> = noms.iter().filter(|n| n.starts_with("abonnements.json.abime-20261115-103000")).collect();
    assert_eq!(abime.len(), 1, "{noms:?}");
    // le fichier abîmé est gardé tel quel
    assert_eq!(
        std::fs::read_to_string(e.0.join(abime[0])).unwrap(),
        "{ \"format\": 1, \"abonnements\": [ { \"id\": \"a1\", "
    );
    // et la bonne liste est remise en place
    assert_eq!(charger(&e).0.liste, exemple());
}

#[test]
fn fichier_vide_ou_abime_sans_copie_liste_vide() {
    let e = Essai::nouveau("abime-sans-copie");
    ecrire(&e.fichier(), "pas du json");
    let (a, mut f) = charger(&e);
    assert!(a.liste.is_empty());
    assert!(e.noms().iter().any(|n| n.starts_with("abonnements.json.abime-")));
    // on peut repartir de zéro
    let mut a = a;
    a.liste = exemple();
    a.sauver(&mut f).unwrap();
    assert_eq!(charger(&e).0.liste, exemple());
}

#[test]
fn json_qui_n_est_pas_une_liste_d_abonnements() {
    let e = Essai::nouveau("pas-une-liste");
    // une bonne copie de secours
    ecrire(&e.0.join(format!("{FICHIER}.bak")), &liste_json(&exemple()).to_string());
    ecrire(&e.fichier(), "{ \"theme\": \"neon\" }");
    let (a, _) = charger(&e);
    assert_eq!(a.liste, exemple());
    let noms = e.noms();
    let abime: Vec<_> = noms.iter().filter(|n| n.starts_with("abonnements.json.abime-")).collect();
    assert_eq!(abime.len(), 1, "{noms:?}");
    assert_eq!(std::fs::read_to_string(e.0.join(abime[0])).unwrap(), "{ \"theme\": \"neon\" }");
    assert!(e.journal().contains("pas de numéro de format"));
    // la copie elle-même ne vaut pas mieux : mise de côté aussi, liste vide
    let e = Essai::nouveau("pas-une-liste-2");
    ecrire(&e.0.join(format!("{FICHIER}.bak")), "{ \"format\": 1, \"abonnements\": 3 }");
    ecrire(&e.fichier(), "{ \"format\": 1 }");
    let (a, _) = charger(&e);
    assert!(a.liste.is_empty() && a.lecture_seule().is_none());
    let noms = e.noms();
    assert_eq!(noms.iter().filter(|n| n.starts_with("abonnements.json.abime-")).count(), 1, "{noms:?}");
    assert_eq!(noms.iter().filter(|n| n.starts_with("abonnements.json.bak.abime-")).count(), 1, "{noms:?}");
}

#[test]
fn un_abonnement_illisible_est_laisse_de_cote_mais_garde() {
    let e = Essai::nouveau("illisible");
    let mut j = liste_json(&exemple());
    let l = j["abonnements"].as_array_mut().unwrap();
    l.push(json!({ "id": "a3", "nom": "Date fausse", "prix": 100, "rythme": "mois", "date": "2026-02-30" }));
    l.push(json!({ "id": "a4", "nom": "Prix négatif", "prix": -5, "rythme": "mois", "date": "2026-02-03" }));
    l.push(json!({ "id": "a1", "nom": "En double", "prix": 100, "rythme": "mois", "date": "2026-02-03" }));
    l.push(json!({ "id": "a5", "nom": "Rythme inconnu", "prix": 100, "rythme": "jour", "date": "2026-02-03" }));
    l.push(
        json!({ "id": "a6", "nom": "Ancré au 32", "prix": 100, "rythme": "mois", "date": "2026-02-03", "ancrage": 32 }),
    );
    l.push(json!("pas un objet"));
    // le strict nécessaire suffit : le reste prend sa valeur par défaut
    l.push(json!({ "id": "a7", "nom": "Minimal", "prix": 250, "rythme": "an", "date": "2027-01-31" }));
    let texte = serde_json::to_string_pretty(&j).unwrap();
    ecrire(&e.fichier(), &texte);
    let (a, _) = charger(&e);
    let ids: Vec<_> = a.liste.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, vec!["a1", "a2", "a7"]);
    let m = &a.liste[2];
    assert_eq!((m.ancrage, m.auto, m.rappel, m.pause, m.categorie.as_str()), (31, true, Rappel::Veille, false, "moi"));
    assert!(a.lecture_seule().is_none());
    // tout le fichier lu est recopié à côté avant que l'enregistrement suivant ne le remplace
    let noms = e.noms();
    let copie: Vec<_> = noms.iter().filter(|n| n.starts_with("abonnements.json.incomplet-")).collect();
    assert_eq!(copie.len(), 1, "{noms:?}");
    let c: Value = serde_json::from_str(&std::fs::read_to_string(e.0.join(copie[0])).unwrap()).unwrap();
    assert_eq!(c, j);
    let journal = e.journal();
    for n in 3..=8 {
        assert!(journal.contains(&format!("abonnement numéro {n} laissé de côté")), "{journal}");
    }
}

#[test]
fn fichier_d_une_version_future_jamais_reecrit() {
    let e = Essai::nouveau("futur");
    let mut j = liste_json(&exemple());
    j["format"] = json!(FORMAT + 1);
    j["nouveaute"] = json!({ "budget": 100 });
    let texte = serde_json::to_string_pretty(&j).unwrap();
    ecrire(&e.fichier(), &texte);
    let (mut a, mut f) = charger(&e);
    // lu comme il peut, pour montrer quelque chose
    assert_eq!(a.liste, exemple());
    assert!(a.lecture_seule().is_some());
    a.liste.clear();
    assert!(a.sauver(&mut f).is_err());
    assert_eq!(std::fs::read_to_string(e.fichier()).unwrap(), texte);
    assert!(e.journal().contains("version plus récente"));
}

#[test]
fn les_textes_et_accents_restent_tels_quels() {
    let e = Essai::nouveau("accents");
    let (mut a, mut f) = charger(&e);
    let mut x = Abonnement::nouveau("a1", "Crédit Mutuel « Assurance » ", 2500, Rythme::Mois, d(2026, 12, 1));
    x.note = "Ligne 1\r\nLigne 2 \"guillemets\" et \\ barre".into();
    a.liste = vec![x.clone()];
    a.sauver(&mut f).unwrap();
    assert_eq!(charger(&e).0.liste, vec![x]);
}

#[test]
fn la_carte_de_mindset_vient_de_la_liste() {
    let e = Essai::nouveau("carte");
    let h = horloge_figee(Moment::nouveau(2026, 11, 15, 10, 30, 0).unwrap());
    // aucun abonnement
    let mut dn = Donnees::ouvrir(&e.0, h.clone(), true);
    dn.ecrire_etat();
    let etat: Value = serde_json::from_str(&std::fs::read_to_string(e.0.join("etat.json")).unwrap()).unwrap();
    assert_eq!(etat["Lignes"], json!([{ "Texte": "Aucun abonnement pour l'instant", "Etat": "info" }]));
    // trois abonnements à 9,99 € par mois, enregistrés, puis relus à l'ouverture suivante
    dn.abonnements.liste =
        (1..=3).map(|i| Abonnement::nouveau(&format!("a{i}"), "Vidéo", 999, Rythme::Mois, d(2026, 11, 18))).collect();
    dn.abonnements.sauver(&mut dn.fichiers).unwrap();
    let mut dn = Donnees::ouvrir(&e.0, h, true);
    assert_eq!(dn.abonnements.liste.len(), 3);
    dn.ecrire_etat();
    let etat: Value = serde_json::from_str(&std::fs::read_to_string(e.0.join("etat.json")).unwrap()).unwrap();
    assert_eq!(
        etat["Lignes"],
        json!([
            { "Texte": "29,97\u{a0}€ par mois", "Etat": "ok" },
            { "Texte": "Vidéo dans 3 jours", "Etat": "info" },
            { "Texte": "3 abonnements", "Etat": "info" }
        ])
    );
    assert_eq!(abonnements::FICHIER, "abonnements.json");
}
