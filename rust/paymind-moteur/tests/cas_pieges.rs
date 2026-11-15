//! Les cas pièges du calcul des comptes : montants, pause, sans renouvellement, PC éteint longtemps, « Bientôt »,
//! rappels (la veille d'un 1er du mois, le jour même, manqués PC éteint, sans doublon), textes de l'écran.

use paymind_moteur::textes::{euros, euros_court, texte_prochain, texte_rappel};
use paymind_moteur::{
    Abonnement, Date, Instant, Rappel, Rythme, Statut, avancer_tout, bilan, paiements_entre, rappels_dus, renouveler,
    verifier,
};

fn d(a: i32, m: u32, j: u32) -> Date {
    Date::nouvelle(a, m, j).unwrap()
}

fn h(date: Date, heure: u32) -> Instant {
    Instant::nouveau(date, heure, 0)
}

fn ab(id: &str, nom: &str, prix: u64, rythme: Rythme, date: Date) -> Abonnement {
    Abonnement::nouveau(id, nom, prix, rythme, date)
}

const AUJ: (i32, u32, u32) = (2026, 11, 15);

fn auj() -> Date {
    d(AUJ.0, AUJ.1, AUJ.2)
}

// ---------- Montants ----------

#[test]
fn trois_abonnements_a_9_99_par_mois() {
    let l: Vec<_> = (1..=3).map(|i| ab(&format!("a{i}"), "Vidéo", 999, Rythme::Mois, d(2026, 11, 20))).collect();
    let b = bilan(&l, auj());
    assert_eq!(b.par_mois, 2997);
    assert_eq!(b.par_an, 35_964);
    assert_eq!(euros(b.par_mois), "29,97\u{a0}€");
    assert_eq!(euros(b.par_an), "359,64\u{a0}€");
    assert_eq!(b.actifs, 3);
}

#[test]
fn un_abonnement_a_52_euros_par_semaine() {
    let l = vec![ab("a1", "Panier", 5200, Rythme::Semaine, d(2026, 11, 16))];
    let b = bilan(&l, auj());
    assert_eq!(b.par_an, 270_400); // 52 × 52 €
    assert_eq!(b.par_mois, 22_533); // 2704 € / 12 = 225,333...
    assert_eq!(euros(b.par_an), "2\u{a0}704,00\u{a0}€");
    assert_eq!(euros(b.par_mois), "225,33\u{a0}€");
}

#[test]
fn le_total_ne_derive_pas_avec_les_arrondis() {
    // 12 abonnements à 0,01 € par an : 0,0833 centime par mois chacun. Arrondis un par un, ils feraient 0 ;
    // le total par an fait 12 centimes, soit 1 centime par mois.
    let l: Vec<_> = (1..=12).map(|i| ab(&format!("a{i}"), "Petit", 1, Rythme::An, d(2027, 1, 1))).collect();
    let b = bilan(&l, auj());
    assert_eq!(b.par_an, 12);
    assert_eq!(b.par_mois, 1);
    // Un trimestre à 10 € : 40 € par an, 3,33 € par mois ; trois de suite : 120 € par an, 10 € par mois tout rond
    let t: Vec<_> = (1..=3).map(|i| ab(&format!("t{i}"), "Trim", 1000, Rythme::Trimestre, d(2027, 1, 1))).collect();
    let b = bilan(&t, auj());
    assert_eq!((b.par_an, b.par_mois), (12_000, 1000));
}

#[test]
fn la_pause_et_les_abonnements_a_renouveler_ne_comptent_pas() {
    let mut l = vec![
        ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18)),
        ab("a2", "Spotify", 1112, Rythme::Mois, d(2026, 11, 22)),
        ab("a3", "Salle", 2999, Rythme::Mois, d(2026, 11, 10)),
    ];
    l[1].pause = true;
    l[2].auto = false; // 10 novembre passé : à renouveler
    assert_eq!(l[1].statut(auj()), Statut::EnPause);
    assert_eq!(l[2].statut(auj()), Statut::ARenouveler);
    let b = bilan(&l, auj());
    assert_eq!((b.total, b.actifs, b.en_pause, b.a_renouveler), (3, 1, 1, 1));
    assert_eq!(b.par_mois, 1349);
    assert_eq!(b.bientot.len(), 1);
    assert_eq!(b.prochain.as_ref().unwrap().nom, "Netflix");
}

#[test]
fn categories_de_la_plus_chere_a_la_moins_chere() {
    let mut l = vec![
        ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18)),
        ab("a2", "Assurance", 54_000, Rythme::An, d(2027, 1, 9)),
        ab("a3", "Spotify", 1112, Rythme::Mois, d(2026, 11, 22)),
        ab("a4", "Dofus", 599, Rythme::Mois, d(2026, 12, 1)),
        ab("a5", "Box", 2999, Rythme::Mois, d(2026, 11, 20)),
    ];
    for (a, c) in l.iter_mut().zip(["video", "assurance", "video", "jeux", "telephone"]) {
        a.categorie = c.to_string();
    }
    let b = bilan(&l, auj());
    let noms: Vec<_> = b.categories.iter().map(|c| (c.categorie.as_str(), c.par_an, c.nombre)).collect();
    assert_eq!(noms, vec![("assurance", 54_000, 1), ("telephone", 35_988, 1), ("video", 29_532, 2), ("jeux", 7188, 1)]);
    assert_eq!(b.categories[0].par_mois, 4500);
    // (le total vient du total par an de tout, pas de la somme des catégories arrondies)
    assert_eq!(b.par_an, 54_000 + 35_988 + 29_532 + 7188);
    assert_eq!(b.par_mois, (b.par_an + 6) / 12);
}

#[test]
fn liste_vide() {
    let b = bilan(&[], auj());
    assert_eq!((b.par_an, b.par_mois, b.total, b.actifs), (0, 0, 0, 0));
    assert!(b.bientot.is_empty() && b.prochain.is_none() && b.categories.is_empty());
}

// ---------- Dates qui avancent ----------

#[test]
fn pc_eteint_quatorze_mois() {
    let mut l = vec![
        ab("a1", "Mois", 1000, Rythme::Mois, d(2025, 9, 30)),
        ab("a2", "Semaine", 500, Rythme::Semaine, d(2025, 9, 30)),
        ab("a3", "Trimestre", 3000, Rythme::Trimestre, d(2025, 8, 31)),
        ab("a4", "An", 12_000, Rythme::An, d(2025, 9, 30)),
        ab("a5", "Sans", 1000, Rythme::Mois, d(2025, 9, 30)),
    ];
    l[4].auto = false;
    let maintenant = d(2026, 11, 30);
    assert_eq!(avancer_tout(&mut l, maintenant), 4);
    assert_eq!(l[0].date, d(2026, 11, 30)); // aujourd'hui même
    // 30 septembre 2025 (un mardi) + 61 semaines = 427 jours : le mardi 1er décembre 2026
    assert_eq!(l[1].date, d(2026, 12, 1));
    assert_eq!(l[1].date.jour_semaine(), d(2025, 9, 30).jour_semaine());
    // 31 août 2025, ancré au 31 : 30 nov. 2025, 28 févr., 31 mai, 31 août, 30 nov. 2026 (aujourd'hui)
    assert_eq!(l[2].date, d(2026, 11, 30));
    assert_eq!(l[3].date, d(2027, 9, 30));
    assert_eq!(l[4].date, d(2025, 9, 30)); // sans renouvellement : ne bouge pas
    // (une seconde fois : plus rien ne bouge)
    assert_eq!(avancer_tout(&mut l, maintenant), 0);
}

#[test]
fn trimestre_ancre_au_31_sur_deux_ans() {
    let a = ab("a1", "Trim", 3000, Rythme::Trimestre, d(2026, 1, 31));
    let l = paiements_entre(&a, d(2026, 1, 1), d(2027, 12, 31));
    assert_eq!(
        l,
        vec![
            d(2026, 1, 31),
            d(2026, 4, 30),
            d(2026, 7, 31),
            d(2026, 10, 31),
            d(2027, 1, 31),
            d(2027, 4, 30),
            d(2027, 7, 31),
            d(2027, 10, 31)
        ]
    );
}

#[test]
fn mois_ancre_au_31_toute_l_annee_2028() {
    let a = ab("a1", "M", 100, Rythme::Mois, d(2028, 1, 31));
    let jours: Vec<u32> = paiements_entre(&a, d(2028, 1, 1), d(2028, 12, 31)).iter().map(|x| x.jour()).collect();
    assert_eq!(jours, vec![31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]);
}

#[test]
fn semaine_qui_traverse_la_fin_d_annee() {
    let a = ab("a1", "S", 100, Rythme::Semaine, d(2026, 12, 17));
    assert_eq!(
        paiements_entre(&a, d(2026, 12, 20), d(2027, 1, 20)),
        vec![d(2026, 12, 24), d(2026, 12, 31), d(2027, 1, 7), d(2027, 1, 14)]
    );
}

#[test]
fn renouveler_a_la_main() {
    let mut a = ab("a1", "Salle", 2999, Rythme::Mois, d(2026, 10, 31));
    a.auto = false;
    assert_eq!(a.statut(auj()), Statut::ARenouveler);
    assert!(renouveler(&mut a, auj()));
    assert_eq!(a.date, d(2026, 11, 30));
    assert!(!a.auto, "renouveler ne change pas le réglage");
    assert_eq!(a.statut(auj()), Statut::Actif);
    assert!(!renouveler(&mut a, auj()));
}

// ---------- Bientôt et prochain paiement ----------

#[test]
fn bientot_trente_jours_tries_par_date() {
    let mut l = vec![
        ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18)),
        ab("a2", "Assurance", 54_000, Rythme::An, d(2027, 1, 9)),
        ab("a3", "Orange", 1599, Rythme::Mois, d(2026, 11, 15)), // aujourd'hui
        ab("a4", "Panier", 2000, Rythme::Semaine, d(2026, 11, 20)),
        ab("a5", "Dernier jour", 100, Rythme::An, d(2026, 12, 14)), // dans 29 jours, le 30e jour : oui
        ab("a6", "Trop loin", 100, Rythme::An, d(2026, 12, 15)),    // dans 30 jours, le 31e jour : non
        ab("a7", "Pause", 100, Rythme::Mois, d(2026, 11, 16)),
        ab("a8", "Fini", 100, Rythme::Mois, d(2026, 11, 14)),
        ab("a9", "Abc", 100, Rythme::Mois, d(2026, 11, 18)), // même jour que Netflix : avant (ordre des noms)
    ];
    l[6].pause = true;
    l[7].auto = false;
    let b = bilan(&l, auj());
    let vus: Vec<_> = b.bientot.iter().map(|p| (p.nom.as_str(), p.date, p.dans_jours)).collect();
    assert_eq!(
        vus,
        vec![
            ("Orange", d(2026, 11, 15), 0),
            ("Abc", d(2026, 11, 18), 3),
            ("Netflix", d(2026, 11, 18), 3),
            ("Panier", d(2026, 11, 20), 5),
            ("Panier", d(2026, 11, 27), 12),
            ("Panier", d(2026, 12, 4), 19),
            ("Panier", d(2026, 12, 11), 26),
            ("Dernier jour", d(2026, 12, 14), 29),
        ]
    );
    // (Orange, payé aujourd'hui, revient le 15 décembre : au-delà des 30 jours, il ne paraît qu'une fois)
    let p = b.prochain.unwrap();
    assert_eq!((p.nom.as_str(), p.dans_jours), ("Orange", 0));
    assert_eq!(texte_prochain(&p, auj()), "Orange aujourd'hui");
}

#[test]
fn bientot_avec_des_dates_gardees_passees() {
    // (la liste n'a pas encore avancé : le bilan avance lui-même, sans la changer)
    let l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 10, 5))];
    let b = bilan(&l, auj());
    assert_eq!(b.bientot.len(), 1);
    assert_eq!(b.bientot[0].date, d(2026, 12, 5));
    assert_eq!(l[0].date, d(2026, 10, 5));
}

#[test]
fn prochain_paiement_au_dela_de_trente_jours() {
    let l = vec![ab("a1", "Assurance", 54_000, Rythme::An, d(2027, 1, 9))];
    let b = bilan(&l, auj());
    assert!(b.bientot.is_empty());
    let p = b.prochain.unwrap();
    assert_eq!(p.dans_jours, 55);
    assert_eq!(texte_prochain(&p, auj()), "Assurance le samedi 9 janvier 2027");
    let l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18))];
    assert_eq!(texte_prochain(&bilan(&l, auj()).prochain.unwrap(), auj()), "Netflix dans 3 jours");
    let l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 16))];
    assert_eq!(texte_prochain(&bilan(&l, auj()).prochain.unwrap(), auj()), "Netflix demain");
}

// ---------- Rappels ----------

#[test]
fn rappel_la_veille_d_un_paiement_le_premier_du_mois() {
    let l = vec![ab("a1", "Dofus", 599, Rythme::Mois, d(2026, 12, 1))];
    assert_eq!(l[0].rappel, Rappel::Veille);
    // dû le 30 novembre à 9 h (novembre a 30 jours)
    assert!(rappels_dus(&l, h(d(2026, 11, 30), 8), Instant::nouveau(d(2026, 11, 30), 8, 59)).is_empty());
    let r = rappels_dus(&l, h(d(2026, 11, 30), 8), h(d(2026, 11, 30), 10));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].du, h(d(2026, 11, 30), 9));
    assert_eq!(r[0].paiement, d(2026, 12, 1));
    assert_eq!(texte_rappel(&r[0], d(2026, 11, 30)), "Dofus : 5,99\u{a0}€ demain");
    // le 1er mars : la veille est le 28 février (2027), le 29 (2028)
    let l = vec![ab("a1", "M", 100, Rythme::An, d(2027, 3, 1))];
    assert_eq!(rappels_dus(&l, h(d(2027, 2, 27), 12), h(d(2027, 3, 2), 0))[0].du.date, d(2027, 2, 28));
    let l = vec![ab("a1", "M", 100, Rythme::An, d(2028, 3, 1))];
    assert_eq!(rappels_dus(&l, h(d(2028, 2, 27), 12), h(d(2028, 3, 2), 0))[0].du.date, d(2028, 2, 29));
    // le 1er janvier : la veille est le 31 décembre de l'année d'avant
    let l = vec![ab("a1", "M", 100, Rythme::Mois, d(2027, 1, 1))];
    assert_eq!(rappels_dus(&l, h(d(2026, 12, 30), 12), h(d(2027, 1, 2), 0))[0].du.date, d(2026, 12, 31));
}

#[test]
fn rappel_le_jour_meme_et_plusieurs_jours_avant() {
    let mut a = ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18));
    a.rappel = Rappel::JourMeme;
    let r = rappels_dus(std::slice::from_ref(&a), h(d(2026, 11, 18), 0), h(d(2026, 11, 18), 12));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].du, h(d(2026, 11, 18), 9));
    assert_eq!(texte_rappel(&r[0], d(2026, 11, 18)), "Netflix : 13,49\u{a0}€ aujourd'hui");
    a.rappel = Rappel::TroisJours;
    assert_eq!(rappels_dus(std::slice::from_ref(&a), h(auj(), 0), h(auj(), 23))[0].du.date, d(2026, 11, 15));
    a.rappel = Rappel::SeptJours;
    assert_eq!(
        rappels_dus(std::slice::from_ref(&a), h(d(2026, 11, 10), 0), h(d(2026, 11, 12), 0))[0].du.date,
        d(2026, 11, 11)
    );
    a.rappel = Rappel::Jamais;
    assert!(rappels_dus(std::slice::from_ref(&a), h(d(2026, 1, 1), 0), h(d(2027, 1, 1), 0)).is_empty());
}

#[test]
fn rappel_manque_pc_eteint() {
    // Le rappel tombait le samedi 14 novembre à 9 h ; le PC était éteint du 10 au 20 novembre
    let mut l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 15))];
    let v = verifier(&mut l, h(d(2026, 11, 10), 8), h(d(2026, 11, 20), 12));
    assert_eq!(v.rappels.len(), 1);
    assert_eq!(v.rappels[0].paiement, d(2026, 11, 15));
    assert_eq!(texte_rappel(&v.rappels[0], d(2026, 11, 20)), "Netflix : 13,49\u{a0}€ payé le dimanche 15 novembre");
    // la date a avancé après le calcul des rappels
    assert_eq!(v.avances, 1);
    assert_eq!(l[0].date, d(2026, 12, 15));
}

#[test]
fn pc_eteint_quatorze_mois_un_seul_rappel_par_abonnement() {
    let mut l = vec![
        ab("a1", "Netflix", 1349, Rythme::Mois, d(2025, 9, 18)),
        ab("a2", "Panier", 2000, Rythme::Semaine, d(2025, 9, 18)),
    ];
    let v = verifier(&mut l, h(d(2025, 9, 1), 12), h(d(2026, 11, 20), 12));
    assert_eq!(v.rappels.len(), 2);
    // les plus récents : Netflix le 18 novembre 2026 (veille : 17), Panier le jeudi 19 novembre 2026 (veille : 18)
    let p: Vec<_> = v.rappels.iter().map(|r| (r.nom.as_str(), r.paiement)).collect();
    assert_eq!(p, vec![("Netflix", d(2026, 11, 18)), ("Panier", d(2026, 11, 19))]);
    assert_eq!(v.avances, 2);
    assert_eq!(l[0].date, d(2026, 12, 18));
    assert_eq!(l[1].date, d(2026, 11, 26));
}

#[test]
fn deux_verifications_de_suite_sans_doublon() {
    let mut l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18))];
    let t1 = h(d(2026, 11, 17), 8);
    let t2 = h(d(2026, 11, 17), 9); // pile l'instant du rappel
    let t3 = h(d(2026, 11, 17), 10);
    let t4 = h(d(2026, 11, 19), 10);
    let mut vus = Vec::new();
    for (a, b) in [(t1, t2), (t2, t3), (t3, t4)] {
        vus.extend(verifier(&mut l, a, b).rappels.into_iter().map(|r| r.cle()));
    }
    assert_eq!(vus, vec!["a1@2026-11-18".to_string()]);
    assert_eq!(l[0].date, d(2026, 12, 18));
    // (l'horloge qui recule : rien)
    assert!(rappels_dus(&l, t4, t1).is_empty());
    assert!(rappels_dus(&l, t4, t4).is_empty());
}

#[test]
fn pas_de_rappel_en_pause() {
    let mut a = ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18));
    a.pause = true;
    assert!(rappels_dus(&[a], h(d(2026, 11, 1), 0), h(d(2026, 12, 31), 0)).is_empty());
}

#[test]
fn rappel_sans_renouvellement_automatique() {
    let mut a = ab("a1", "Salle", 2999, Rythme::Mois, d(2026, 11, 18));
    a.auto = false;
    let l = vec![a];
    // un seul paiement, jamais ceux des mois suivants
    let r = rappels_dus(&l, h(d(2026, 11, 1), 0), h(d(2027, 6, 1), 0));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].paiement, d(2026, 11, 18));
    assert_eq!(texte_rappel(&r[0], d(2026, 11, 17)), "Salle : à renouveler demain");
    assert_eq!(texte_rappel(&r[0], d(2026, 11, 19)), "Salle : à renouveler depuis hier");
    assert_eq!(texte_rappel(&r[0], d(2026, 11, 25)), "Salle : à renouveler depuis le mercredi 18 novembre");
}

#[test]
fn rappels_tries_par_instant() {
    let mut l = vec![
        ab("a1", "Zèbre", 100, Rythme::Mois, d(2026, 11, 20)),
        ab("a2", "Alpha", 100, Rythme::Mois, d(2026, 11, 22)),
        ab("a3", "Beta", 100, Rythme::Mois, d(2026, 11, 20)),
    ];
    l[1].rappel = Rappel::SeptJours; // dû le 15
    let r = rappels_dus(&l, h(d(2026, 11, 14), 0), h(d(2026, 11, 20), 0));
    let noms: Vec<_> = r.iter().map(|x| x.nom.as_str()).collect();
    assert_eq!(noms, vec!["Alpha", "Beta", "Zèbre"]);
}

// ---------- Textes ----------

#[test]
fn aucun_tiret_long_dans_les_textes() {
    let l = vec![ab("a1", "Netflix", 1349, Rythme::Mois, d(2026, 11, 18))];
    let b = bilan(&l, auj());
    let r = rappels_dus(&l, h(auj(), 0), h(d(2026, 11, 30), 0));
    let mut textes =
        vec![euros(b.par_an), euros_court(b.par_mois), texte_prochain(b.prochain.as_ref().unwrap(), auj())];
    textes.extend(r.iter().map(|x| texte_rappel(x, auj())));
    for jour in 0..400 {
        let x = auj().plus_jours(jour - 30).unwrap();
        textes.push(paymind_moteur::textes::quand(x, auj()));
        textes.push(paymind_moteur::textes::date_fiche(x, auj()));
    }
    for r in Rappel::TOUS {
        textes.push(r.texte().to_string());
    }
    for t in textes {
        assert!(!t.contains(['\u{2013}', '\u{2014}']), "{t}");
    }
}
