//! Le portefeuille (étape 3 du plan) : l'accueil de la maquette F (maquettes\proposition-f.html), préparé ici à partir
//! de la liste des abonnements et du calcul des comptes (paymind-moteur). La page (ui\portefeuille.js) ne calcule rien :
//! elle dessine ce qu'elle reçoit (des textes déjà écrits en français, des couleurs déjà vérifiées).
//!
//! - le total par mois et par an, et combien d'abonnements comptent (ceux en pause ou à renouveler sont dits à part) ;
//! - « Bientôt » : les trois prochains abonnements à payer dans les 30 jours, et la page de tous les paiements des
//!   30 jours ;
//! - les cartes rangées par catégorie (dans l'ordre de la liste toute prête), chacune avec sa fiche ;
//! - la liste vide : « Ajoute ton premier abonnement ».
//!
//! Depuis l'étape 4, une carte ajoutée depuis la liste toute prête (src\catalogue.rs) a les couleurs et le logo
//! officiel de sa marque ([`crate::catalogue::logo_carte`]) ; une carte « à moi » montre une plaque avec ses
//! initiales, aux couleurs de sa catégorie. Le bloc `catalogue` (la liste toute prête, src\ajout.rs) part avec
//! l'accueil : « Ajouter » la montre sans attendre.
//!
//! Depuis l'étape 5, la fiche est modifiable (src\fiche.rs) : chaque fiche porte aussi ce que ses champs montrent
//! (le prix à taper, la date du calendrier, « C'est renouvelé », la question « Supprimer Netflix ? »), et le bloc
//! `communs.fiche` les textes de la fiche et les bornes du calendrier (aujourd'hui, dans 3 ans).
//!
//! Tous les textes vont à la page tels quels (elle les échappe) : en français simple, sans tiret long ni
//! demi-cadratin.

use paymind_moteur::abonnement::NOM_MAX;
use paymind_moteur::textes::{dans, date_courte, date_fiche, date_longue, euros, euros_court, nombre_abonnements};
use paymind_moteur::{
    Abonnement, BIENTOT_JOURS, Date, Rappel, Rythme, Statut, bilan, par_mois, prochaine_echeance, renouveler,
};
use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::catalogue::{Modele, catalogue, logo_carte, logo_rond};

/// La couleur de Paymind (celle de sa carte dans Mindset, de son logo, du bouton « Ajouter »).
pub const COULEUR: &str = "#22E07A";

/// Le bouton « Ajouter » : il montre la liste toute prête, dessinée par la page dans l'accueil (src\ajout.rs,
/// bloc `catalogue`).
pub const PAGE_AJOUTER: &str = "ajouter";
/// L'id de la page de tous les paiements des 30 prochains jours.
pub const PAGE_BIENTOT: &str = "bientot";
/// Le nombre d'abonnements montrés en pastilles dans « Bientôt » (les autres sont sur la page « Bientôt »).
pub const PASTILLES: usize = 3;

/// Une catégorie : son id (celui de la liste toute prête), son nom à l'écran et les deux couleurs de ses cartes
/// sans logo (le haut à gauche, le bas à droite du dégradé).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Categorie {
    pub id: &'static str,
    pub nom: &'static str,
    pub c1: &'static str,
    pub c2: &'static str,
}

/// La catégorie d'un abonnement « à moi » (la seule, avec [`AUTRES`], qui n'est pas dans la liste toute prête).
pub const MOI: Categorie = Categorie { id: "moi", nom: "À moi", c1: "#5B4BD6", c2: "#1C1450" };
/// La catégorie d'un abonnement dont la catégorie n'est pas connue de cette version.
pub const AUTRES: Categorie = Categorie { id: "autres", nom: "Autres", c1: "#5A5A6E", c2: "#17171F" };

/// Les catégories, dans l'ordre de l'écran : celles de la liste toute prête (catalogue\catalogue.json : leurs noms
/// et leurs couleurs, dans leur ordre), puis « À moi ». Une catégorie inconnue (d'une version plus récente) va dans
/// [`AUTRES`].
pub fn categories() -> &'static [Categorie] {
    static LISTE: OnceLock<Vec<Categorie>> = OnceLock::new();
    LISTE.get_or_init(|| {
        let mut l: Vec<Categorie> = catalogue()
            .categories
            .iter()
            .map(|c| Categorie { id: &c.id, nom: &c.nom, c1: &c.fond, c2: &c.fond_fin })
            .collect();
        l.push(MOI);
        l
    })
}

/// La catégorie d'un id (inconnue : [`AUTRES`]).
pub fn categorie(id: &str) -> Categorie {
    categories().iter().copied().find(|c| c.id == id).unwrap_or(AUTRES)
}

/// Le modèle de la liste toute prête d'un abonnement (rien pour un abonnement « à moi », ou d'un modèle qui n'est
/// plus dans la liste).
fn modele(a: &Abonnement) -> Option<&'static Modele> {
    a.catalogue.as_deref().and_then(|id| catalogue().modele(id))
}

/// Les couleurs d'une carte : celles de sa marque (liste toute prête), sinon celles de sa catégorie.
fn couleurs(a: &Abonnement) -> (&'static str, &'static str) {
    match modele(a) {
        Some(m) => (&m.fond, &m.fond_fin),
        None => {
            let c = categorie(&a.categorie);
            (c.c1, c.c2)
        }
    }
}

/// Les initiales d'un nom, pour la plaque d'une carte sans logo : la première lettre ou le premier chiffre de chacun
/// des deux premiers mots qui comptent, en capitales (« World of Warcraft » donne « WW », « Crédit Mutuel » « CM »,
/// « Netflix » « N »). Rien de lisible : « ? ».
pub fn initiales(nom: &str) -> String {
    const PETITS: [&str; 12] = ["de", "du", "des", "la", "le", "les", "of", "the", "et", "and", "d", "l"];
    let mots: Vec<&str> = nom.split(|c: char| !c.is_alphanumeric()).filter(|m| !m.is_empty()).collect();
    let utiles: Vec<&&str> = mots.iter().filter(|m| !PETITS.contains(&m.to_lowercase().as_str())).collect();
    let pris = if utiles.is_empty() { mots.iter().collect() } else { utiles };
    let s: String = pris.iter().take(2).filter_map(|m| m.chars().next()).flat_map(char::to_uppercase).collect();
    if s.is_empty() { "?".to_string() } else { s }
}

/// Le mot de l'état d'un abonnement pour la page (une classe CSS) : actif, pause, renouveler.
fn statut_code(s: Statut) -> &'static str {
    match s {
        Statut::Actif => "actif",
        Statut::EnPause => "pause",
        Statut::ARenouveler => "renouveler",
    }
}

/// Les choix de rythme de la fiche, dans l'ordre (code, texte).
pub(crate) const RYTHMES: [(Rythme, &str); 4] =
    [(Rythme::Semaine, "Semaine"), (Rythme::Mois, "Mois"), (Rythme::Trimestre, "Trimestre"), (Rythme::An, "An")];

/// Le prochain paiement d'un abonnement actif ce jour-là (rien s'il n'en a pas).
fn prochain(a: &Abonnement, aujourdhui: Date) -> Option<Date> {
    prochaine_echeance(a, aujourdhui)
}

/// La phrase du rappel de la fiche : « Une bulle près de l'horloge la veille, le mardi 17 novembre ».
fn texte_rappel(a: &Abonnement, statut: Statut, paiement: Option<Date>, aujourdhui: Date) -> String {
    match statut {
        Statut::EnPause => return "Pas de bulle tant qu'il est en pause".into(),
        Statut::ARenouveler => return "Plus de bulle : sa date est passée".into(),
        Statut::Actif => {}
    }
    let Some(jours) = a.rappel.jours_avant() else { return "Pas de bulle pour cet abonnement".into() };
    let quand = match a.rappel {
        Rappel::JourMeme => "le jour même".to_string(),
        Rappel::Veille => "la veille".to_string(),
        _ => format!("{jours} jours avant"),
    };
    match paiement.and_then(|d| d.plus_jours(-jours)) {
        Some(d) if d >= aujourdhui => format!("Une bulle près de l'horloge {quand}, {}", date_longue(d, aujourdhui)),
        _ => format!("Une bulle près de l'horloge {quand}"),
    }
}

/// Un prix tel qu'on le tape dans la fiche : « 13,49 », « 540 » (sans espace ni euro).
pub fn prix_saisie(centimes: u64) -> String {
    if centimes.is_multiple_of(100) {
        (centimes / 100).to_string()
    } else {
        format!("{},{:02}", centimes / 100, centimes % 100)
    }
}

/// La fiche d'un abonnement (modifiable depuis l'étape 5, src\fiche.rs).
fn fiche(a: &Abonnement, statut: Statut, aujourdhui: Date) -> Value {
    let paiement = if statut == Statut::Actif { prochain(a, aujourdhui) } else { None };
    // (le jour montré par le calendrier en s'ouvrant : le prochain paiement, ou la date passée à renouveler)
    let (titre_date, date, quand, jour) = match statut {
        Statut::EnPause => ("Prochain paiement", "En pause".to_string(), String::new(), None),
        Statut::ARenouveler => {
            ("À renouveler depuis", date_fiche(a.date, aujourdhui), dans(a.date, aujourdhui), Some(a.date))
        }
        Statut::Actif => match paiement {
            Some(d) => ("Prochain paiement", date_fiche(d, aujourdhui), dans(d, aujourdhui), Some(d)),
            None => ("Prochain paiement", "Aucun".to_string(), String::new(), None),
        },
    };
    // (dit sous le titre de la date : pourquoi il ne compte pas)
    let etat = match statut {
        Statut::Actif => "",
        Statut::EnPause => "Il ne compte pas dans le total",
        Statut::ARenouveler => "Il ne compte plus dans le total",
    };
    // « C'est renouvelé » : la date où il repartira
    let renouvele = (statut == Statut::ARenouveler).then(|| {
        let mut b = a.clone();
        renouveler(&mut b, aujourdhui);
        json!({ "texte": "C'est renouvelé", "aide": format!("Le prochain paiement sera {}", date_longue(b.date, aujourdhui)) })
    });
    json!({
        "categorie": categorie(&a.categorie).nom,
        "prix": euros_court(a.prix),
        "prix_saisie": prix_saisie(a.prix),
        "rythme": a.rythme.code(),
        "mois": euros(par_mois(a.par_an())),
        "an": euros(a.par_an()),
        "titre_date": titre_date,
        "date": date,
        "quand": quand,
        // (en pause, la date ne se change pas : on la reprend d'abord)
        "date_iso": jour.map(|d| d.iso()),
        "auto": a.auto,
        "auto_texte": if a.auto { "Il repart tout seul à chaque fois" } else { "Non : il faut le renouveler soi-même" },
        "rappel": a.rappel.code(),
        "rappel_texte": texte_rappel(a, statut, paiement, aujourdhui),
        "etat": etat,
        "note": a.note,
        // (seul un abonnement « à moi » change de nom)
        "nom_modifiable": a.catalogue.is_none(),
        "pause": a.pause,
        "pause_texte": if a.pause { "Reprendre" } else { "Mettre en pause" },
        "renouvele": renouvele,
        "question": format!("Supprimer {} ?", a.nom),
    })
}

/// Les textes de la fiche, les mêmes pour toutes les cartes, et les bornes du calendrier.
fn textes_fiche(aujourdhui: Date) -> Value {
    json!({
        "je_paie": "Je paie",
        "tous_les": "Tous les",
        "par_mois": "Par mois",
        "par_an": "Par an",
        "erreur_prix": crate::fiche::ERREUR_PRIX,
        "nom": "Son nom",
        "nom_max": NOM_MAX,
        "note": "Une note",
        "note_indice": "Par exemple : le compte de la famille",
        "note_max": crate::fiche::NOTE_FICHE_MAX,
        "auto": "Renouvellement automatique",
        "prevenir": "Me prévenir",
        "changer_date": "Changer la date",
        "supprimer": "Supprimer",
        "suppr_aide": "Sa carte disparaît pour de bon.",
        "suppr_oui": "Oui, supprimer",
        "suppr_non": "Non, garder",
        "calendrier": {
            "titre": "Choisis le jour du prochain paiement",
            "annuler": "Annuler",
            "avant": "Mois d'avant",
            "apres": "Mois d'après",
            "mois": ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre",
                     "novembre", "décembre"],
            // (du lundi au dimanche)
            "jours": ["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."],
            "min": aujourdhui.iso(),
            "max": crate::fiche::date_max(aujourdhui).iso(),
        },
    })
}

/// Une carte du portefeuille.
fn carte(a: &Abonnement, aujourdhui: Date) -> Value {
    let statut = a.statut(aujourdhui);
    let (c1, c2) = couleurs(a);
    // (en pause ou à renouveler : une pastille en haut à droite, et la ligne du bas le dit ; la date ne se coupe
    // jamais en deux, « 10 » et « déc. » restent sur la même ligne)
    let date = |d: Date| date_courte(d).replace(' ', "\u{a0}");
    let (ligne, badge) = match statut {
        Statut::EnPause => ("Ne compte pas".to_string(), "En pause"),
        Statut::ARenouveler => (format!("Depuis le {}", date(a.date)), "À renouveler"),
        Statut::Actif => match prochain(a, aujourdhui) {
            Some(d) => (format!("Prochain : {}", date(d)), ""),
            None => (String::new(), ""),
        },
    };
    json!({
        "id": a.id,
        "nom": a.nom,
        "initiales": initiales(&a.nom),
        "c1": c1,
        "c2": c2,
        // (le logo officiel d'un abonnement de la liste toute prête ; rien pour un abonnement « à moi » : la plaque
        // de ses initiales)
        "logo": modele(a).map_or(Value::Null, logo_carte),
        "prix": euros_court(a.prix),
        "rythme": a.rythme.par_texte(),
        "ligne": ligne,
        "badge": badge,
        // (un petit mot en haut à droite, comme « Box » ou « Mobile » dans la maquette : pas dans la liste toute
        // prête pour l'instant)
        "etiquette": "",
        "statut": statut_code(statut),
        "fiche": fiche(a, statut, aujourdhui),
    })
}

/// Le bloc du portefeuille (l'accueil dessiné par ui\portefeuille.js), pour la liste ce jour-là.
pub fn portefeuille(liste: &[Abonnement], aujourdhui: Date) -> Value {
    let jour = date_fiche(aujourdhui, aujourdhui);
    let commun = json!({
        "rythmes": RYTHMES.iter().map(|(r, t)| json!([r.code(), t])).collect::<Vec<_>>(),
        "rappels": Rappel::TOUS.iter().map(|r| json!([r.code(), r.texte()])).collect::<Vec<_>>(),
        "fiche": textes_fiche(aujourdhui),
    });
    if liste.is_empty() {
        return json!({
            "jour": jour,
            "vide": true,
            "titre": "Ajoute ton premier abonnement",
            "texte": "Paymind fait le compte de ce que tu paies chaque mois et te prévient avant chaque paiement.",
            "note": "Tout reste sur ce PC : rien n'est envoyé sur Internet.",
            "ajouter": PAGE_AJOUTER,
            "communs": commun,
        });
    }
    let b = bilan(liste, aujourdhui);

    // « Bientôt » : les premiers abonnements à payer (un abonnement par semaine n'y paraît qu'une fois)
    let mut vus: Vec<&str> = Vec::new();
    let mut pastilles = Vec::new();
    for p in &b.bientot {
        if vus.contains(&p.id.as_str()) {
            continue;
        }
        vus.push(&p.id);
        if pastilles.len() < PASTILLES
            && let Some(a) = liste.iter().find(|a| a.id == p.id)
        {
            let (c1, c2) = couleurs(a);
            pastilles.push(json!({
                "id": a.id,
                "nom": a.nom,
                "initiales": initiales(&a.nom),
                "c1": c1,
                "c2": c2,
                // (un logo trop large pour un rond : les initiales, et le nom à côté)
                "logo": modele(a).and_then(logo_rond).unwrap_or(Value::Null),
                "date": date_courte(p.date),
                "aide": format!("{} : {} {}", a.nom, euros_court(p.prix), dans(p.date, aujourdhui)),
            }));
        }
    }
    let bientot = json!({
        "titre": "Bientôt",
        "elements": pastilles,
        // (plus de paiements que de pastilles : « Tout voir » mène à la page de tous les paiements des 30 jours)
        "tout": if b.bientot.len() > PASTILLES { json!({ "id": PAGE_BIENTOT, "texte": "Tout voir" }) } else { Value::Null },
        "vide": "Rien à payer dans les 30 prochains jours",
    });

    // Les cartes, par catégorie (dans l'ordre de la liste toute prête ; dans une catégorie, l'ordre de la liste)
    let mut ordre: Vec<Categorie> = categories().to_vec();
    ordre.push(AUTRES);
    let groupes: Vec<Value> = ordre
        .iter()
        .filter_map(|c| {
            let cartes: Vec<Value> =
                liste.iter().filter(|a| categorie(&a.categorie).id == c.id).map(|a| carte(a, aujourdhui)).collect();
            if cartes.is_empty() {
                return None;
            }
            let par_an: u64 = liste
                .iter()
                .filter(|a| categorie(&a.categorie).id == c.id && a.statut(aujourdhui) == Statut::Actif)
                .map(Abonnement::par_an)
                .sum();
            let total = if par_an > 0 { format!("{} par mois", euros(par_mois(par_an))) } else { String::new() };
            Some(json!({ "id": c.id, "nom": c.nom, "total": total, "cartes": cartes }))
        })
        .collect();

    // Combien comptent, et ceux qui ne comptent pas
    let mut compte = nombre_abonnements(b.actifs);
    let mut a_part = Vec::new();
    if b.en_pause > 0 {
        a_part.push(format!("{} en pause", b.en_pause));
    }
    if b.a_renouveler > 0 {
        a_part.push(format!("{} à renouveler", b.a_renouveler));
    }
    if !a_part.is_empty() {
        let verbe = if b.en_pause + b.a_renouveler > 1 { "comptent" } else { "compte" };
        compte = format!("{compte}, plus {} qui ne {verbe} pas", a_part.join(" et "));
    }

    json!({
        "jour": jour,
        "vide": false,
        "total": { "mois": euros(b.par_mois), "par": "par mois", "an": euros(b.par_an), "compte": compte },
        "bientot": bientot,
        "groupes": groupes,
        "ajouter": PAGE_AJOUTER,
        "communs": commun,
    })
}

/// La page de tous les paiements des 30 prochains jours (rien si la liste est vide).
pub fn page_bientot(liste: &[Abonnement], aujourdhui: Date) -> Option<Value> {
    if liste.is_empty() {
        return None;
    }
    let b = bilan(liste, aujourdhui);
    let total: u64 = b.bientot.iter().map(|p| p.prix).sum();
    let mut blocs: Vec<Value> = b
        .bientot
        .iter()
        .map(|p| {
            json!({
                "type": "ligne",
                "texte": p.nom,
                "note": date_longue(p.date, aujourdhui),
                "valeur": euros_court(p.prix),
                "etiquette": dans(p.date, aujourdhui),
                "ton": if p.dans_jours <= 1 { "attention" } else { "" },
            })
        })
        .collect();
    if blocs.is_empty() {
        blocs.push(json!({ "type": "texte", "texte": "Rien à payer dans les 30 prochains jours." }));
    }
    blocs.push(json!({
        "type": "note",
        "texte": "Les abonnements en pause ou à renouveler ne sont pas dans cette liste.",
    }));
    Some(json!({
        "titre": "Bientôt",
        "sous": format!("Les paiements des {BIENTOT_JOURS} prochains jours"),
        "icone": "agenda",
        "couleur": COULEUR,
        "module": {
            "icone": "agenda",
            "couleur": COULEUR,
            "titre": "Dans les 30 jours",
            "texte": format!("{} à payer", euros(total)),
            "resume": [
                { "nom": "Paiements", "valeur": b.bientot.len().to_string() },
                { "nom": "Par mois en tout", "valeur": euros(b.par_mois) },
            ],
        },
        "blocs": blocs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    fn ab(id: &str, nom: &str, cat: &str, prix: u64, rythme: Rythme, date: Date) -> Abonnement {
        let mut a = Abonnement::nouveau(id, nom, prix, rythme, date);
        a.categorie = cat.into();
        a
    }

    /// Une dizaine d'abonnements de catégories variées, dont des noms longs, un en pause et un à renouveler.
    fn liste() -> Vec<Abonnement> {
        let mut l = vec![
            ab("a1", "Netflix", "films", 1349, Rythme::Mois, d(2026, 11, 18)),
            ab("a2", "Spotify", "musique", 1112, Rythme::Mois, d(2026, 11, 22)),
            ab("a3", "World of Warcraft", "jeux", 1299, Rythme::Mois, d(2026, 11, 25)),
            ab("a4", "Crédit Mutuel", "banque", 2500, Rythme::Mois, d(2026, 12, 2)),
            ab("a5", "Free", "telephone", 2999, Rythme::Mois, d(2026, 11, 20)),
            ab("a6", "AXA", "assurances", 54000, Rythme::An, d(2027, 1, 9)),
            ab("a7", "Basic-Fit", "sport", 2999, Rythme::Mois, d(2026, 11, 19)),
            ab("a8", "Disney+", "films", 999, Rythme::Mois, d(2026, 11, 27)),
            ab("a9", "Dofus", "jeux", 599, Rythme::Mois, d(2026, 12, 1)),
            ab("a10", "Le Monde", "presse", 1099, Rythme::Mois, d(2026, 11, 16)),
            ab("a11", "Mon club de foot", "moi", 4000, Rythme::Trimestre, d(2026, 12, 10)),
        ];
        l[7].pause = true;
        l[8].auto = false;
        l[8].date = d(2026, 11, 3);
        l
    }

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
    fn initiales_des_cartes_sans_logo() {
        assert_eq!(initiales("World of Warcraft"), "WW");
        assert_eq!(initiales("Crédit Mutuel"), "CM");
        assert_eq!(initiales("Netflix"), "N");
        assert_eq!(initiales("Disney+"), "D");
        assert_eq!(initiales("Basic-Fit"), "BF");
        assert_eq!(initiales("Le Monde"), "M");
        assert_eq!(initiales("La Banque Postale"), "BP");
        assert_eq!(initiales("éDF"), "É");
        assert_eq!(initiales("Le"), "L");
        assert_eq!(initiales("+++"), "?");
        assert_eq!(initiales("123 Box"), "1B");
    }

    #[test]
    fn categories_connues_et_inconnues() {
        assert_eq!(categorie("films").nom, "Films et séries");
        assert_eq!(categorie("moi").nom, "À moi");
        assert_eq!(categorie("video"), AUTRES);
        assert_eq!(categorie(""), AUTRES);
        // Les noms et couleurs viennent de la liste toute prête, dans son ordre, puis « À moi »
        let ids: Vec<&str> = categories().iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            [
                "films",
                "musique",
                "jeux",
                "telephone",
                "banque",
                "assurances",
                "sport",
                "presse",
                "logiciels",
                "courses",
                "energie",
                "moi"
            ]
        );
        assert_eq!(categorie("jeux").c1, "#3E5FC8");
        assert_eq!(categorie("telephone").nom, "Téléphone et internet");
        // Chaque catégorie a deux couleurs que la page accepte, et un id unique
        let mut ids = Vec::new();
        for c in categories().iter().chain([&AUTRES]) {
            assert!(crate::pont::couleur_valable(c.c1) && crate::pont::couleur_valable(c.c2), "{c:?}");
            assert!(crate::pont::id_valable(c.id));
            assert!(!ids.contains(&c.id), "{} en double", c.id);
            ids.push(c.id);
        }
    }

    /// Une carte de la liste toute prête a les couleurs et le logo de sa marque ; une carte « à moi », ou d'un modèle
    /// qui n'est plus dans la liste, la plaque de ses initiales aux couleurs de sa catégorie.
    #[test]
    fn cartes_de_la_liste_toute_prete_avec_leur_logo() {
        let auj = d(2026, 11, 15);
        let avec = |mut a: Abonnement, id: &str| {
            a.catalogue = Some(id.into());
            a
        };
        let l = vec![
            avec(ab("a1", "Netflix", "films", 799, Rythme::Mois, d(2026, 11, 16)), "netflix"),
            avec(ab("a2", "World of Warcraft", "jeux", 1299, Rythme::Mois, d(2026, 11, 17)), "worldofwarcraft"),
            avec(ab("a3", "Dofus", "jeux", 599, Rythme::Mois, d(2026, 11, 18)), "dofus"),
            ab("a4", "Mon club", "moi", 4000, Rythme::Trimestre, d(2026, 12, 10)),
            avec(ab("a5", "Vieux service", "films", 500, Rythme::Mois, d(2026, 12, 1)), "plus-dans-la-liste"),
            avec(ab("a6", "Amazon Prime", "films", 699, Rythme::Mois, d(2026, 12, 2)), "amazonprime"),
        ];
        let p = portefeuille(&l, auj);
        let films = p["groupes"][0]["cartes"].as_array().unwrap();
        assert_eq!(films[0]["c1"], "#E50914");
        assert_eq!(films[0]["logo"]["forme"], "blanc");
        assert_eq!(films[0]["logo"]["url"], "logos/netflix.svg");
        assert_eq!(films[1]["c1"], categorie("films").c1);
        assert_eq!(films[1]["logo"], Value::Null);
        let jeux = p["groupes"][1]["cartes"].as_array().unwrap();
        assert_eq!(jeux[0]["logo"]["url"], "logos/worldofwarcraft.svg");
        assert_eq!(jeux[1]["logo"]["url"], "logos/dofus.png");
        let moi = &p["groupes"][2]["cartes"][0];
        assert_eq!((moi["c1"].as_str(), &moi["logo"]), (Some(MOI.c1), &Value::Null));
        // Les pastilles : Netflix (son N), World of Warcraft (son logo rond), Dofus (son œuf)
        let el = p["bientot"]["elements"].as_array().unwrap();
        assert_eq!(el[0]["logo"]["url"], "logos/netflix.svg");
        assert_eq!(el[1]["logo"]["url"], "logos/worldofwarcraft.svg");
        assert_eq!(el[2]["logo"]["url"], "logos/dofus-oeuf.png");
        assert_eq!(el[2]["c1"], "#7BA33A");
    }

    #[test]
    fn liste_vide() {
        let p = portefeuille(&[], d(2026, 11, 15));
        assert_eq!(p["vide"], true);
        assert_eq!(p["titre"], "Ajoute ton premier abonnement");
        assert_eq!(p["jour"], "Dimanche 15 novembre");
        assert!(p.get("groupes").is_none() && p.get("total").is_none());
        assert!(page_bientot(&[], d(2026, 11, 15)).is_none());
    }

    #[test]
    fn totaux_et_compte() {
        let auj = d(2026, 11, 15);
        let p = portefeuille(&liste(), auj);
        assert_eq!(p["vide"], false);
        // Actifs, par an : 12 × (13,49 + 11,12 + 12,99 + 25 + 29,99 + 29,99 + 10,99) + 540 + 4 × 40 = 2 302,84 € ;
        // par mois : 2 302,84 / 12 = 191,903... arrondi à la fin
        let b = bilan(&liste(), auj);
        assert_eq!(b.actifs, 9);
        assert_eq!(p["total"]["mois"], euros(b.par_mois));
        assert_eq!(p["total"]["an"], "2\u{a0}302,84\u{a0}€");
        assert_eq!(p["total"]["mois"], "191,90\u{a0}€");
        assert_eq!(p["total"]["compte"], "9 abonnements, plus 1 en pause et 1 à renouveler qui ne comptent pas");
        let mut l = liste();
        l[8].auto = true;
        assert_eq!(portefeuille(&l, auj)["total"]["compte"], "10 abonnements, plus 1 en pause qui ne compte pas");
        l[7].pause = false;
        assert_eq!(portefeuille(&l, auj)["total"]["compte"], "11 abonnements");
    }

    #[test]
    fn cartes_par_categorie_dans_l_ordre_de_la_liste_toute_prete() {
        let auj = d(2026, 11, 15);
        let p = portefeuille(&liste(), auj);
        let groupes = p["groupes"].as_array().unwrap();
        let ids: Vec<&str> = groupes.iter().map(|g| g["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["films", "musique", "jeux", "telephone", "banque", "assurances", "sport", "presse", "moi"]);
        // Films : Netflix puis Disney+ (en pause, il ne compte pas dans le total de la catégorie)
        let films = &groupes[0];
        assert_eq!(films["nom"], "Films et séries");
        assert_eq!(films["total"], "13,49\u{a0}€ par mois");
        let cartes = films["cartes"].as_array().unwrap();
        assert_eq!(cartes.len(), 2);
        assert_eq!(cartes[0]["nom"], "Netflix");
        assert_eq!(cartes[0]["statut"], "actif");
        assert_eq!(cartes[0]["ligne"], "Prochain : 18\u{a0}nov.");
        assert_eq!(cartes[0]["prix"], "13,49\u{a0}€");
        assert_eq!(cartes[0]["rythme"], "par mois");
        assert_eq!(cartes[0]["initiales"], "N");
        assert_eq!(cartes[0]["c1"], "#D6304A");
        assert_eq!(cartes[0]["logo"], Value::Null);
        assert_eq!(cartes[0]["badge"], "");
        assert_eq!(cartes[1]["statut"], "pause");
        assert_eq!(cartes[1]["badge"], "En pause");
        assert_eq!(cartes[1]["ligne"], "Ne compte pas");
        // Jeux : World of Warcraft (actif) et Dofus (à renouveler : ne compte plus)
        let jeux = &groupes[2];
        assert_eq!(jeux["total"], "12,99\u{a0}€ par mois");
        assert_eq!(jeux["cartes"][0]["nom"], "World of Warcraft");
        assert_eq!(jeux["cartes"][0]["initiales"], "WW");
        assert_eq!(jeux["cartes"][1]["statut"], "renouveler");
        assert_eq!(jeux["cartes"][1]["badge"], "À renouveler");
        assert_eq!(jeux["cartes"][1]["ligne"], "Depuis le 3\u{a0}nov.");
        // AXA, par an : 540 € sans les centimes
        assert_eq!(groupes[5]["cartes"][0]["prix"], "540\u{a0}€");
        assert_eq!(groupes[5]["cartes"][0]["rythme"], "par an");
        // Une catégorie sans aucun abonnement actif : pas de total
        let mut l = liste();
        l[2].pause = true;
        assert_eq!(portefeuille(&l, auj)["groupes"][2]["total"], "");
        // Une catégorie inconnue : « Autres », à la fin
        l[0].categorie = "video".into();
        let g = portefeuille(&l, auj)["groupes"].as_array().unwrap().clone();
        assert_eq!(g.last().unwrap()["nom"], "Autres");
        assert_eq!(g.last().unwrap()["cartes"][0]["nom"], "Netflix");
    }

    #[test]
    fn bientot_trois_pastilles_et_tout_voir() {
        let auj = d(2026, 11, 15);
        let p = portefeuille(&liste(), auj);
        let el = p["bientot"]["elements"].as_array().unwrap();
        let noms: Vec<&str> = el.iter().map(|x| x["nom"].as_str().unwrap()).collect();
        assert_eq!(noms, ["Le Monde", "Netflix", "Basic-Fit"]);
        assert_eq!(el[0]["date"], "16 nov.");
        assert_eq!(el[0]["aide"], "Le Monde : 10,99\u{a0}€ demain");
        assert_eq!(p["bientot"]["tout"]["id"], PAGE_BIENTOT);
        // Un abonnement par semaine n'a qu'une pastille ; peu de paiements : pas de « Tout voir »
        let l = vec![ab("a1", "Panier bio", "courses", 2500, Rythme::Semaine, d(2026, 11, 17))];
        let p = portefeuille(&l, auj);
        assert_eq!(p["bientot"]["elements"].as_array().unwrap().len(), 1);
        // (mais 4 paiements dans les 30 jours : « Tout voir » mène à la page qui les montre tous)
        assert_eq!(p["bientot"]["tout"]["id"], PAGE_BIENTOT);
        let l = vec![ab("a1", "Netflix", "films", 1349, Rythme::Mois, d(2026, 11, 18))];
        assert_eq!(portefeuille(&l, auj)["bientot"]["tout"], Value::Null);
        // Rien dans les 30 jours
        let l = vec![ab("a1", "AXA", "assurances", 54000, Rythme::An, d(2027, 1, 9))];
        let p = portefeuille(&l, auj);
        assert!(p["bientot"]["elements"].as_array().unwrap().is_empty());
        assert_eq!(p["bientot"]["vide"], "Rien à payer dans les 30 prochains jours");
    }

    #[test]
    fn fiche_de_chaque_carte() {
        let auj = d(2026, 11, 15);
        let p = portefeuille(&liste(), auj);
        let netflix = &p["groupes"][0]["cartes"][0]["fiche"];
        assert_eq!(netflix["prix"], "13,49\u{a0}€");
        assert_eq!(netflix["prix_saisie"], "13,49");
        assert_eq!(netflix["date_iso"], "2026-11-18");
        assert_eq!(netflix["nom_modifiable"], true);
        assert_eq!(netflix["pause_texte"], "Mettre en pause");
        assert_eq!(netflix["renouvele"], Value::Null);
        assert_eq!(netflix["question"], "Supprimer Netflix ?");
        assert_eq!(netflix["auto_texte"], "Il repart tout seul à chaque fois");
        assert_eq!(netflix["rythme"], "mois");
        assert_eq!(netflix["mois"], "13,49\u{a0}€");
        assert_eq!(netflix["an"], "161,88\u{a0}€");
        assert_eq!(netflix["titre_date"], "Prochain paiement");
        assert_eq!(netflix["date"], "Mercredi 18 novembre");
        assert_eq!(netflix["quand"], "dans 3 jours");
        assert_eq!(netflix["auto"], true);
        assert_eq!(netflix["rappel"], "veille");
        assert_eq!(netflix["rappel_texte"], "Une bulle près de l'horloge la veille, le mardi 17 novembre");
        assert_eq!(netflix["etat"], "");
        let pause = &p["groupes"][0]["cartes"][1]["fiche"];
        assert_eq!(pause["date"], "En pause");
        assert_eq!(pause["quand"], "");
        assert_eq!(pause["rappel_texte"], "Pas de bulle tant qu'il est en pause");
        assert_eq!(pause["etat"], "Il ne compte pas dans le total");
        assert_eq!(pause["date_iso"], Value::Null);
        assert_eq!(pause["pause_texte"], "Reprendre");
        let dofus = &p["groupes"][2]["cartes"][1]["fiche"];
        assert_eq!(dofus["titre_date"], "À renouveler depuis");
        assert_eq!(dofus["date"], "Mardi 3 novembre");
        assert_eq!(dofus["quand"], "il y a 12 jours");
        assert_eq!(dofus["rappel_texte"], "Plus de bulle : sa date est passée");
        assert_eq!(dofus["etat"], "Il ne compte plus dans le total");
        assert_eq!(dofus["date_iso"], "2026-11-03");
        assert_eq!(dofus["auto_texte"], "Non : il faut le renouveler soi-même");
        assert_eq!(
            dofus["renouvele"],
            json!({ "texte": "C'est renouvelé", "aide": "Le prochain paiement sera le mardi 1er décembre" })
        );
        // Les textes de la fiche et les bornes du calendrier
        let f = &p["communs"]["fiche"];
        assert_eq!(f["calendrier"]["min"], "2026-11-15");
        assert_eq!(f["calendrier"]["max"], "2029-11-15");
        assert_eq!(f["calendrier"]["jours"][0], "lun.");
        assert_eq!(prix_saisie(54_000), "540");
        assert_eq!(prix_saisie(5), "0,05");
        assert_eq!(prix_saisie(99_999_999), "999999,99");
        // Le trimestre ramené au mois et à l'an ; le paiement de l'an prochain avec son année
        let club = &p["groupes"][8]["cartes"][0]["fiche"];
        assert_eq!(club["mois"], "13,33\u{a0}€");
        assert_eq!(club["an"], "160,00\u{a0}€");
        let axa = &p["groupes"][5]["cartes"][0]["fiche"];
        assert_eq!(axa["date"], "Samedi 9 janvier 2027");
        assert_eq!(axa["rappel_texte"], "Une bulle près de l'horloge la veille, le vendredi 8 janvier 2027");
        // Les autres rappels
        let mut a = ab("a1", "Netflix", "films", 1349, Rythme::Mois, d(2026, 11, 18));
        a.rappel = Rappel::Jamais;
        assert_eq!(fiche(&a, Statut::Actif, auj)["rappel_texte"], "Pas de bulle pour cet abonnement");
        a.rappel = Rappel::SeptJours;
        assert_eq!(fiche(&a, Statut::Actif, auj)["rappel_texte"], "Une bulle près de l'horloge 7 jours avant");
        a.rappel = Rappel::TroisJours;
        assert_eq!(
            fiche(&a, Statut::Actif, auj)["rappel_texte"],
            "Une bulle près de l'horloge 3 jours avant, le dimanche 15 novembre"
        );
        a.rappel = Rappel::JourMeme;
        assert_eq!(
            fiche(&a, Statut::Actif, auj)["rappel_texte"],
            "Une bulle près de l'horloge le jour même, le mercredi 18 novembre"
        );
    }

    #[test]
    fn page_de_tous_les_paiements() {
        let auj = d(2026, 11, 15);
        let p = page_bientot(&liste(), auj).unwrap();
        let lignes: Vec<&Value> = p["blocs"].as_array().unwrap().iter().filter(|b| b["type"] == "ligne").collect();
        let b = bilan(&liste(), auj);
        assert_eq!(lignes.len(), b.bientot.len());
        assert_eq!(lignes[0]["texte"], "Le Monde");
        assert_eq!(lignes[0]["note"], "le lundi 16 novembre");
        assert_eq!(lignes[0]["etiquette"], "demain");
        assert_eq!(lignes[0]["ton"], "attention");
        assert_eq!(lignes[1]["ton"], "");
        // AXA (janvier) n'y est pas, ni Disney+ (en pause), ni Dofus (à renouveler)
        assert!(!lignes.iter().any(|l| ["AXA", "Disney+", "Dofus"].contains(&l["texte"].as_str().unwrap())));
    }

    /// Règle de la suite : aucun tiret long ni demi-cadratin, aucun HTML (la page écrit les textes tels quels).
    #[test]
    fn textes_simples() {
        let auj = d(2026, 11, 15);
        let mut l = Vec::new();
        textes(&portefeuille(&liste(), auj), &mut l);
        textes(&portefeuille(&[], auj), &mut l);
        textes(&page_bientot(&liste(), auj).unwrap(), &mut l);
        assert!(l.len() > 100);
        for t in l {
            assert!(!t.contains(['\u{2013}', '\u{2014}', '<', '>']), "{t}");
        }
    }
}
