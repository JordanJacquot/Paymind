//! Les rappels près de l'horloge, avec un faux temps (une horloge qu'on avance à la main) sur un dossier d'essai sous
//! %TEMP% (jamais les vraies données). Aucune vraie bulle : personne ne lit notification.demande à part le « faux
//! Mindset » de ces tests ([`Essai::bulle_lue`]), qui la lit et la supprime comme Mindset.
//!
//! Le rappel la veille à 9 h, le rappel manqué pendant que le PC était éteint, pas de doublon après un redémarrage, le
//! premier lancement, plusieurs rappels regroupés, « Jamais », la pause, une bulle à la fois (6 s, 5 minutes), le clic
//! (la fiche, le portefeuille, une bulle inconnue), la carte de Mindset au passage de minuit.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use paymind_donnees::rappels::{AFFICHER, FICHIER, NOTIFICATION};
use paymind_donnees::{Cible, Demande, Donnees, Moment};
use paymind_moteur::{Abonnement, Date, Rappel, Rythme};
use serde_json::Value;

fn d(a: i32, m: u32, j: u32) -> Date {
    Date::nouvelle(a, m, j).unwrap()
}

fn m(a: i32, mo: u32, j: u32, h: u32, mi: u32, s: u32) -> Moment {
    Moment::nouveau(a, mo, j, h, mi, s).unwrap()
}

/// Un dossier d'essai neuf sous %TEMP%, effacé à la fin, et son horloge (le faux temps).
struct Essai {
    dossier: PathBuf,
    heure: Arc<Mutex<Moment>>,
}

impl Essai {
    fn nouveau(nom: &str, depart: Moment) -> Essai {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dossier = std::env::temp_dir().join(format!("PaymindEssai-rappels-{}-{nom}-{n}", std::process::id()));
        assert!(dossier.is_absolute());
        std::fs::create_dir_all(&dossier).unwrap();
        Essai { dossier, heure: Arc::new(Mutex::new(depart)) }
    }

    /// L'appli démarre (ou redémarre) : les données relues sur le disque, à l'heure du faux temps.
    fn demarrer(&self) -> Donnees {
        let h = Arc::clone(&self.heure);
        Donnees::ouvrir(&self.dossier, Arc::new(move || *h.lock().unwrap()), true)
    }

    /// Le faux temps avance (ou recule) jusqu'à ce moment.
    fn a(&self, moment: Moment) -> Moment {
        *self.heure.lock().unwrap() = moment;
        moment
    }

    /// Une liste d'abonnements enregistrée, comme si la personne l'avait remplie avant.
    fn liste(&self, liste: Vec<Abonnement>) {
        let mut dn = self.demarrer();
        dn.abonnements.liste = liste;
        dn.abonnements.sauver(&mut dn.fichiers).unwrap();
    }

    /// Le faux Mindset : lit notification.demande et le supprime (titre, texte, id), comme Watch-NotificationsApplis.
    fn bulle_lue(&self) -> Option<(String, String, String)> {
        let f = self.dossier.join(NOTIFICATION);
        let o = std::fs::read(&f).ok()?;
        std::fs::remove_file(&f).unwrap();
        assert_ne!(o.first(), Some(&0xEF), "UTF-8 sans BOM");
        let v: Value = serde_json::from_slice(&o).unwrap();
        let t = |n: &str| v[n].as_str().unwrap().to_string();
        Some((t("titre"), t("texte"), t("id")))
    }

    /// Le faux Mindset : un clic sur la bulle écrit « notif:<id> » dans afficher.demande.
    fn cliquer(&self, contenu: &str) {
        std::fs::write(self.dossier.join(AFFICHER), contenu).unwrap();
    }

    fn existe(&self, nom: &str) -> bool {
        self.dossier.join(nom).exists()
    }

    fn journal(&self) -> String {
        std::fs::read_to_string(self.dossier.join("journal.txt")).unwrap_or_default()
    }

    fn etat(&self) -> String {
        std::fs::read_to_string(self.dossier.join("etat.json")).unwrap_or_default()
    }
}

impl Drop for Essai {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dossier);
    }
}

fn abo(id: &str, nom: &str, prix: u64, date: Date, rappel: Rappel) -> Abonnement {
    let mut a = Abonnement::nouveau(id, nom, prix, Rythme::Mois, date);
    a.rappel = rappel;
    a
}

/// Un tour de l'appli à ce moment : la vérification puis l'envoi (ce que fait le programme toutes les 30 s et 500 ms).
fn tour(e: &Essai, dn: &mut Donnees, moment: Moment) {
    e.a(moment);
    dn.verifier_rappels(moment);
    dn.envoyer_bulle(moment);
}

#[test]
fn la_veille_a_9_h() {
    let e = Essai::nouveau("veille", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 8, 59, 30));
    assert_eq!(e.bulle_lue(), None, "pas avant 9 h");
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 10));
    let (titre, texte, id) = e.bulle_lue().expect("la bulle à 9 h");
    assert_eq!(titre, "Netflix demain : 13,49\u{a0}€");
    assert_eq!(texte, "Paiement prévu le mardi 17 novembre.");
    assert!(id.len() == 12 && id.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()), "{id}");
    // Un clic sur la bulle : la fiche de Netflix ; afficher.demande toujours supprimé
    e.cliquer(&format!("notif:{id}"));
    assert_eq!(dn.demande_affichage(), Demande::Ouvrir(Cible::Fiche("a1".into())));
    assert!(!e.existe(AFFICHER));
    assert!(e.journal().contains(&format!("Bulle ouverte : {id}")));
    // Les tours suivants (la même minute, puis toute la journée) : plus rien
    for (h, mi) in [(9, 0), (9, 1), (12, 0), (23, 59)] {
        tour(&e, &mut dn, m(2026, 11, 16, h, mi, 40));
        assert_eq!(e.bulle_lue(), None, "{h}:{mi}");
    }
}

#[test]
fn le_jour_meme_3_et_7_jours_avant() {
    let e = Essai::nouveau("delais", m(2026, 11, 1, 10, 0, 0));
    e.liste(vec![
        abo("a1", "Box", 2999, d(2026, 11, 20), Rappel::SeptJours),
        abo("a2", "Spotify", 1112, d(2026, 11, 20), Rappel::TroisJours),
        abo("a3", "Mutuelle", 4500, d(2026, 11, 20), Rappel::JourMeme),
    ]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 1, 10, 0, 0));
    let mut vues = Vec::new();
    for j in 12..=20 {
        tour(&e, &mut dn, m(2026, 11, j, 9, 0, 0));
        if let Some((t, _, _)) = e.bulle_lue() {
            vues.push((j, t));
        }
    }
    assert_eq!(
        vues,
        vec![
            (13, "Box dans 7 jours : 29,99\u{a0}€".to_string()),
            (17, "Spotify dans 3 jours : 11,12\u{a0}€".to_string()),
            (20, "Mutuelle aujourd'hui : 45\u{a0}€".to_string()),
        ]
    );
}

#[test]
fn rappel_manque_pc_eteint() {
    let e = Essai::nouveau("eteint", m(2026, 11, 10, 12, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    {
        let mut dn = e.demarrer();
        tour(&e, &mut dn, m(2026, 11, 10, 12, 0, 0));
    }
    // Le PC est éteint du 10 au 17 au soir : la bulle de la veille (le 16 à 9 h) sort au démarrage suivant
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 17, 18, 0, 0));
    let (titre, texte, _) = e.bulle_lue().expect("la bulle manquée");
    assert_eq!(
        (titre.as_str(), texte.as_str()),
        ("Netflix aujourd'hui : 13,49\u{a0}€", "Paiement prévu le mardi 17 novembre.")
    );
    assert_eq!(dn.abonnements.liste[0].date, d(2026, 11, 17), "le jour même, la date n'avance pas encore");
    drop(dn);
    // Éteint de nouveau jusqu'au 20 : la date a avancé (renouvellement automatique), enregistrée ; aucune bulle
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 20, 8, 0, 0));
    assert_eq!(e.bulle_lue(), None);
    assert_eq!(dn.abonnements.liste[0].date, d(2026, 12, 17));
    assert_eq!(e.demarrer().abonnements.liste[0].date, d(2026, 12, 17), "la liste avancée est enregistrée");
    assert!(e.journal().contains("1 date(s) avancée(s)"));
}

#[test]
fn des_mois_eteint_une_seule_bulle_par_abonnement() {
    let e = Essai::nouveau("mois", m(2026, 6, 1, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 6, 17), Rappel::Veille)]);
    tour(&e, &mut e.demarrer(), m(2026, 6, 1, 10, 0, 0));
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 16, 10, 0, 0));
    let (titre, _, _) = e.bulle_lue().unwrap();
    assert_eq!(titre, "Netflix demain : 13,49\u{a0}€", "seulement la plus récente");
    for s in [6, 12, 30] {
        tour(&e, &mut dn, m(2026, 11, 16, 10, 0, s));
        assert_eq!(e.bulle_lue(), None);
    }
    assert_eq!(dn.abonnements.liste[0].date, d(2026, 11, 17));
}

#[test]
fn pas_de_doublon_apres_redemarrage() {
    let e = Essai::nouveau("doublon", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    tour(&e, &mut e.demarrer(), m(2026, 11, 15, 10, 0, 0));
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 0));
    let (_, _, id) = e.bulle_lue().unwrap();
    drop(dn);
    // Redémarrage juste après (même minute, puis plus tard) : rien de nouveau
    for moment in [m(2026, 11, 16, 9, 0, 30), m(2026, 11, 16, 9, 5, 0), m(2026, 11, 16, 20, 0, 0)] {
        let mut dn = e.demarrer();
        tour(&e, &mut dn, moment);
        tour(&e, &mut dn, m(moment.annee, moment.mois, moment.jour, moment.heure, moment.minute, 59));
        assert_eq!(e.bulle_lue(), None, "{moment}");
        // Le clic sur l'ancienne bulle marche encore après la relance
        e.cliquer(&format!("notif:{id}"));
        assert_eq!(dn.demande_affichage(), Demande::Ouvrir(Cible::Fiche("a1".into())));
    }
    // Mindset ne l'avait pas encore lue au redémarrage : elle n'est pas réécrite (une seule bulle)
    let e = Essai::nouveau("doublon-pas-lue", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    tour(&e, &mut e.demarrer(), m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut e.demarrer(), m(2026, 11, 16, 9, 0, 0));
    assert!(e.existe(NOTIFICATION));
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 16, 9, 1, 0));
    assert!(e.bulle_lue().is_some());
    tour(&e, &mut dn, m(2026, 11, 16, 9, 2, 0));
    assert_eq!(e.bulle_lue(), None);
}

#[test]
fn plantage_avant_l_envoi_la_bulle_sort_une_fois() {
    // La vérification a trouvé le rappel (enregistré en attente), puis l'appli s'arrête net avant de l'envoyer
    let e = Essai::nouveau("plantage", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    tour(&e, &mut e.demarrer(), m(2026, 11, 15, 10, 0, 0));
    let mut dn = e.demarrer();
    e.a(m(2026, 11, 16, 9, 0, 0));
    assert!(dn.verifier_rappels(m(2026, 11, 16, 9, 0, 0)).bulle.is_some());
    std::mem::forget(dn); // (rien de plus n'est fait, comme un plantage)
    assert!(!e.existe(NOTIFICATION));
    let mut dn = e.demarrer();
    assert_eq!(dn.rappels.attente().len(), 1);
    tour(&e, &mut dn, m(2026, 11, 16, 9, 3, 0));
    assert_eq!(e.bulle_lue().unwrap().0, "Netflix demain : 13,49\u{a0}€");
    // Arrêt net juste après l'envoi : la bulle est déjà notée envoyée, elle ne revient pas
    std::mem::forget(dn);
    let mut dn = e.demarrer();
    assert!(dn.rappels.attente().is_empty());
    tour(&e, &mut dn, m(2026, 11, 16, 9, 4, 0));
    assert_eq!(e.bulle_lue(), None);
}

#[test]
fn premier_lancement_rien_du_passe() {
    let e = Essai::nouveau("premier", m(2026, 11, 16, 18, 0, 0));
    // Le rappel de la veille (ce matin à 9 h) est passé avant le tout premier lancement
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    assert!(!e.existe(FICHIER));
    let mut dn = e.demarrer();
    assert_eq!(dn.rappels.verifie(), None);
    tour(&e, &mut dn, m(2026, 11, 16, 18, 0, 0));
    assert_eq!(e.bulle_lue(), None);
    assert!(e.existe(FICHIER), "l'instant de la vérification est gardé");
    assert!(e.journal().contains("premier lancement"));
    // Un rappel.json abîmé : comme un premier lancement (jamais de vieilles bulles en rafale)
    drop(dn);
    std::fs::write(e.dossier.join(FICHIER), "{ pas du json").unwrap();
    std::fs::remove_file(e.dossier.join(format!("{FICHIER}.bak"))).ok();
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 20, 18, 0, 0));
    assert_eq!(e.bulle_lue(), None);
}

#[test]
fn plusieurs_rappels_une_seule_bulle() {
    let e = Essai::nouveau("plusieurs", m(2026, 11, 15, 10, 0, 0));
    let mut salle = abo("a3", "Salle", 2999, d(2026, 11, 17), Rappel::Veille);
    salle.auto = false;
    e.liste(vec![
        abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille),
        abo("a2", "Spotify", 1112, d(2026, 11, 19), Rappel::TroisJours),
        salle,
    ]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 0));
    let (titre, texte, id) = e.bulle_lue().unwrap();
    assert_eq!(titre, "3 rappels de paiement");
    assert_eq!(texte, "Netflix demain (13,49\u{a0}€), Salle à renouveler demain, Spotify dans 3 jours (11,12\u{a0}€).");
    for s in [10, 20, 40] {
        tour(&e, &mut dn, m(2026, 11, 16, 9, 0, s));
        assert_eq!(e.bulle_lue(), None, "une seule bulle");
    }
    e.cliquer(&format!("notif:{id}"));
    assert_eq!(dn.demande_affichage(), Demande::Ouvrir(Cible::Portefeuille));
}

#[test]
fn jamais_et_pause_aucune_bulle() {
    let e = Essai::nouveau("jamais", m(2026, 11, 15, 10, 0, 0));
    let mut pause = abo("a2", "Spotify", 1112, d(2026, 11, 17), Rappel::Veille);
    pause.pause = true;
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Jamais), pause]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    for j in 16..=30 {
        tour(&e, &mut dn, m(2026, 11, j, 9, 0, 0));
        assert_eq!(e.bulle_lue(), None, "le {j}");
    }
    // La pause continue d'avancer son calendrier, sans bulle ; « Jamais » aussi
    assert_eq!(dn.abonnements.liste[0].date, d(2026, 12, 17));
    assert_eq!(dn.abonnements.liste[1].date, d(2026, 12, 17));
    // Repris : la bulle revient pour le paiement suivant
    dn.abonnements.liste[1].pause = false;
    tour(&e, &mut dn, m(2026, 12, 16, 9, 0, 0));
    assert_eq!(e.bulle_lue().unwrap().0, "Spotify demain : 11,12\u{a0}€");
}

#[test]
fn une_bulle_a_la_fois() {
    let e = Essai::nouveau("une-a-la-fois", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![
        abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille),
        abo("a2", "Spotify", 1112, d(2026, 11, 18), Rappel::Veille),
    ]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    // Deux bulles en attente (deux jours sans que Mindset ne lise rien)
    dn.verifier_rappels(e.a(m(2026, 11, 16, 9, 0, 0)));
    dn.verifier_rappels(e.a(m(2026, 11, 17, 9, 0, 0)));
    assert_eq!(dn.rappels.attente().len(), 2);
    let t = m(2026, 11, 17, 9, 0, 0);
    assert!(dn.envoyer_bulle(t).is_some());
    // La première pas encore lue : la seconde attend
    assert!(dn.envoyer_bulle(m(2026, 11, 17, 9, 0, 30)).is_none());
    assert!(e.bulle_lue().unwrap().0.starts_with("Netflix"));
    // Lue tout de suite : 6 s d'écart au moins
    assert!(dn.envoyer_bulle(m(2026, 11, 17, 9, 0, 3)).is_none());
    assert!(dn.envoyer_bulle(m(2026, 11, 17, 9, 0, 6)).is_some());
    assert!(e.bulle_lue().unwrap().0.starts_with("Spotify"));
    assert!(dn.rappels.attente().is_empty());
}

#[test]
fn bulle_abandonnee_apres_5_minutes() {
    let e = Essai::nouveau("abandon", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 0));
    assert!(e.existe(NOTIFICATION));
    dn.envoyer_bulle(m(2026, 11, 16, 9, 4, 59));
    assert!(e.existe(NOTIFICATION), "pas encore");
    dn.envoyer_bulle(m(2026, 11, 16, 9, 5, 1));
    assert!(!e.existe(NOTIFICATION), "Mindset ne l'a pas lue en 5 minutes");
    assert!(e.journal().contains("Bulle abandonnée"));
    // (abandonnée veut dire perdue : elle ne revient pas)
    tour(&e, &mut dn, m(2026, 11, 16, 9, 10, 0));
    assert_eq!(e.bulle_lue(), None);
}

#[test]
fn les_demandes_d_affichage() {
    let e = Essai::nouveau("demandes", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 0));
    let (_, _, id) = e.bulle_lue().unwrap();
    assert_eq!(dn.demande_affichage(), Demande::Rien, "pas de fichier");
    for (contenu, attendu) in [
        ("notif:000000000000".to_string(), Demande::Rien),
        (format!("notif:{}", id.to_uppercase()), Demande::Rien),
        ("notif:12".to_string(), Demande::Rien),
        ("4242".to_string(), Demande::Montrer),
        ("\u{feff}4242\r\n".to_string(), Demande::Montrer),
        ("appli:paymind".to_string(), Demande::Rien),
        ("x".repeat(200), Demande::Rien),
        (format!("\u{feff}notif:{id}\r\n"), Demande::Ouvrir(Cible::Fiche("a1".into()))),
    ] {
        e.cliquer(&contenu);
        assert_eq!(dn.demande_affichage(), attendu, "{contenu}");
        assert!(!e.existe(AFFICHER), "toujours supprimé : {contenu}");
    }
    assert!(e.journal().contains("Bulle inconnue ignorée (000000000000)"));
    // L'abonnement supprimé depuis : le portefeuille
    dn.abonnements.liste.clear();
    e.cliquer(&format!("notif:{id}"));
    assert_eq!(dn.demande_affichage(), Demande::Ouvrir(Cible::Portefeuille));
}

#[test]
fn l_horloge_qui_recule_ou_se_trompe() {
    let e = Essai::nouveau("horloge", m(2026, 11, 15, 10, 0, 0));
    e.liste(vec![abo("a1", "Netflix", 1349, d(2026, 11, 17), Rappel::Veille)]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 10, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 9, 0, 0));
    assert!(e.bulle_lue().is_some());
    // L'horloge recule d'une heure puis repart : pas de seconde bulle
    tour(&e, &mut dn, m(2026, 11, 16, 8, 0, 0));
    tour(&e, &mut dn, m(2026, 11, 16, 9, 30, 0));
    assert_eq!(e.bulle_lue(), None);
    drop(dn);
    // Une dernière vérification notée en 2030 (une horloge fausse) : on repart de maintenant, les rappels reprennent
    std::fs::write(
        e.dossier.join(FICHIER),
        r#"{ "format": 1, "verifie": "2030-01-01T10:00", "attente": [], "envoyees": [] }"#,
    )
    .unwrap();
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 12, 15, 10, 0, 0));
    assert!(e.journal().contains("dans le futur"));
    tour(&e, &mut dn, m(2026, 12, 16, 9, 0, 0));
    assert_eq!(e.bulle_lue().unwrap().0, "Netflix demain : 13,49\u{a0}€");
}

#[test]
fn la_carte_de_mindset_au_passage_de_minuit() {
    let e = Essai::nouveau("minuit", m(2026, 11, 15, 23, 59, 0));
    e.liste(vec![
        abo("a1", "Netflix", 1349, d(2026, 11, 16), Rappel::Veille),
        abo("a2", "Box", 7351, d(2026, 11, 25), Rappel::Jamais),
    ]);
    let mut dn = e.demarrer();
    tour(&e, &mut dn, m(2026, 11, 15, 23, 59, 0));
    dn.ecrire_etat();
    let carte = e.etat();
    assert!(carte.contains("\"87\u{a0}€ par mois\""), "{carte}");
    assert!(carte.contains("\"Netflix demain\""), "{carte}");
    assert!(carte.contains("\"2 abonnements\""), "{carte}");
    // Minuit passé : « aujourd'hui »
    e.a(m(2026, 11, 16, 0, 0, 5));
    dn.ecrire_etat();
    assert!(e.etat().contains("\"Netflix aujourd'hui\""), "{}", e.etat());
    // Le lendemain : la date de Netflix avance (vérification), la carte suit tout de suite
    tour(&e, &mut dn, m(2026, 11, 17, 0, 0, 5));
    let carte = e.etat();
    assert!(carte.contains("\"Box dans 8 jours\""), "{carte}");
    assert_eq!(e.demarrer().abonnements.liste[0].date, d(2026, 12, 16));
}
