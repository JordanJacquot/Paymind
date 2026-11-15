//! Ajouter un abonnement (étape 4 du plan) : la liste toute prête de la maquette F (rangée par catégorie, avec logos
//! et prix habituels, une recherche) et « Un abonnement à moi ».
//!
//! - [`vue`] : ce que la page dessine (le bloc `catalogue` de l'accueil, ui\portefeuille.js) ; la recherche se fait
//!   dans la page, dans la clé préparée ici pour chaque abonnement ([`crate::catalogue::cle_recherche`]) ;
//! - [`lire`] : le geste de la page, vérifié ici (jamais confiance dans la page) : `ajouter` avec l'id d'un abonnement
//!   de la liste, ou `ajouter-moi` avec `{ nom, prix, rythme, categorie }` ; tout le reste est refusé ;
//! - [`ajouter`] : l'abonnement neuf (prix et rythme habituels, prochain paiement un rythme après aujourd'hui,
//!   renouvellement automatique, rappel la veille) enregistré dans abonnements.json, puis la carte de Mindset
//!   (etat.json) mise à jour.
//!
//! Un même abonnement peut être ajouté deux fois (deux comptes Netflix, deux box) : chacun a sa carte, et la liste
//! toute prête le marque « Déjà là ». La fiche modifiable (prix, date...) vient à l'étape 5.

use paymind_donnees::Donnees;
use paymind_moteur::abonnement::NOM_MAX;
use paymind_moteur::textes::{euros_court, lire_prix};
use paymind_moteur::{Abonnement, Date, Rythme, id_libre};
use serde_json::{Value, json};

use crate::catalogue::{Modele, catalogue, cle_recherche, logo_carte};
use crate::portefeuille::{MOI, RYTHMES, categorie, categories, initiales};

/// Le geste « ajouter un abonnement de la liste » (valeur : son id dans la liste toute prête).
pub const ACTION_LISTE: &str = "ajouter";
/// Le geste « ajouter un abonnement à moi » (valeur : `{ nom, prix, rythme, categorie }`).
pub const ACTION_MOI: &str = "ajouter-moi";

const MOIS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Ce que la page affiche pour la liste toute prête, avec la liste de ses abonnements (pour « Déjà là » et le nombre
/// de cartes du bouton « Mes cartes »).
pub fn vue(liste: &[Abonnement]) -> Value {
    let c = catalogue();
    let groupes: Vec<Value> = c
        .categories
        .iter()
        .filter_map(|cat| {
            let elements: Vec<Value> = c
                .modeles
                .iter()
                .filter(|m| m.categorie == cat.id)
                .map(|m| {
                    let deja = liste.iter().filter(|a| a.catalogue.as_deref() == Some(m.id.as_str())).count();
                    json!({
                        "id": m.id,
                        "nom": m.nom,
                        "initiales": initiales(&m.nom),
                        "prix": euros_court(m.prix),
                        "rythme": m.rythme.par_texte(),
                        "offre": m.offre,
                        "verifier": m.a_verifier,
                        "deja": deja,
                        "c1": m.fond,
                        "c2": m.fond_fin,
                        "logo": logo_carte(m),
                        "cle": cle_recherche(m, &cat.nom),
                    })
                })
                .collect();
            (!elements.is_empty()).then(|| json!({ "id": cat.id, "nom": cat.nom, "elements": elements }))
        })
        .collect();
    let releve = Date::lire_iso(&c.releve).map(|d| format!(" en {} {}", MOIS[(d.mois() - 1) as usize], d.annee()));
    json!({
        "titre": "Ajouter un abonnement",
        "retour": "Mes cartes",
        "compte": liste.len().to_string(),
        "chercher": "Chercher : Netflix, box, mutuelle...",
        "rien": "Rien trouvé pour",
        "action": ACTION_LISTE,
        "deja": "Déjà là",
        "verifier": "à vérifier",
        "verifier_aide": "Prix moyen ou qui change selon les cas : à vérifier",
        "groupes": groupes,
        "note": format!(
            "Les prix habituels ont été relevés{}. Tout reste sur ce PC : rien n'est envoyé sur Internet.",
            releve.unwrap_or_default()
        ),
        "moi": {
            "titre": "Pas dans la liste ?",
            "bouton": "Un abonnement à moi",
            "action": ACTION_MOI,
            "nom": "Son nom",
            "nom_indice": "Par exemple : club de foot",
            "nom_max": NOM_MAX,
            "prix": "Je paie",
            "prix_indice": "12,50",
            "rythme": "Tous les",
            "rythmes": RYTHMES.iter().map(|(r, t)| json!([r.code(), t])).collect::<Vec<_>>(),
            "categorie": "Catégorie",
            // (« À moi » d'abord : le choix de départ)
            "categories": std::iter::once(MOI)
                .chain(categories().iter().copied().filter(|c| c.id != MOI.id))
                .map(|c| json!([c.id, c.nom]))
                .collect::<Vec<_>>(),
            "valider": "Ajouter",
            "annuler": "Annuler",
            "erreur_nom": "Écris le nom de l'abonnement.",
            "erreur_prix": "Écris le prix, par exemple 12,50.",
        },
    })
}

/// Un ajout demandé par la page, vérifié.
#[derive(Debug, Clone, PartialEq)]
pub enum Ajout {
    /// Un abonnement de la liste toute prête
    Liste(&'static Modele),
    /// Un abonnement à moi : nom propre, prix plus grand que zéro, catégorie connue
    Moi { nom: String, prix: u64, rythme: Rythme, categorie: &'static str },
}

/// Ce geste est-il un ajout ?
pub fn est_ajout(id: &str) -> bool {
    id == ACTION_LISTE || id == ACTION_MOI
}

/// Un nom tapé, rendu propre : espaces du début et de la fin ôtées, une seule espace entre les mots, tiret long et
/// demi-cadratin changés en tiret simple, chevrons ôtés. Un caractère de contrôle (autre qu'une espace) le fait
/// refuser.
pub fn nom_propre(t: &str) -> Option<String> {
    if t.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return None;
    }
    let t: String = t
        .chars()
        .filter(|c| !matches!(c, '<' | '>'))
        .map(|c| if matches!(c, '\u{2013}' | '\u{2014}') { '-' } else { c })
        .collect();
    Some(t.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Lit le geste `id` de la page et sa valeur. Le refus est un texte simple pour la bulle de l'écran (et le journal).
pub fn lire(id: &str, valeur: &Value) -> Result<Ajout, String> {
    match id {
        ACTION_LISTE => valeur
            .as_str()
            .and_then(|i| catalogue().modele(i))
            .map(Ajout::Liste)
            .ok_or_else(|| "Cet abonnement n'est pas dans la liste.".to_string()),
        ACTION_MOI => {
            let refus = || "Ajout refusé.".to_string();
            let o = valeur.as_object().ok_or_else(refus)?;
            if o.len() != 4 || !["nom", "prix", "rythme", "categorie"].iter().all(|k| o.contains_key(*k)) {
                return Err(refus());
            }
            let nom = o["nom"].as_str().and_then(nom_propre).ok_or_else(refus)?;
            if nom.is_empty() {
                return Err("Écris le nom de l'abonnement.".into());
            }
            if nom.chars().count() > NOM_MAX {
                return Err(format!("Ce nom est trop long ({NOM_MAX} lettres au plus)."));
            }
            let prix = o["prix"]
                .as_str()
                .filter(|p| p.chars().count() <= 20)
                .and_then(lire_prix)
                .filter(|p| *p > 0)
                .ok_or_else(|| "Écris le prix, par exemple 12,50.".to_string())?;
            let rythme = o["rythme"].as_str().and_then(Rythme::depuis_code).ok_or_else(refus)?;
            let cat = o["categorie"].as_str().ok_or_else(refus)?;
            let categorie = categories().iter().find(|c| c.id == cat).map(|c| c.id).ok_or_else(refus)?;
            Ok(Ajout::Moi { nom, prix, rythme, categorie })
        }
        _ => Err("Ajout refusé.".into()),
    }
}

/// Le premier paiement d'un abonnement ajouté aujourd'hui : un rythme plus tard (dans une semaine, un mois, un
/// trimestre, un an), au même jour du mois.
pub fn premier_paiement(rythme: Rythme, aujourdhui: Date) -> Date {
    let d = match rythme.mois() {
        Some(n) => aujourdhui.plus_mois(n, aujourdhui.jour()),
        None => aujourdhui.plus_jours(7),
    };
    d.unwrap_or(aujourdhui)
}

/// L'abonnement neuf de cet ajout (un id libre de la liste) : renouvellement automatique, rappel la veille, jour voulu
/// celui d'aujourd'hui.
pub fn nouvel_abonnement(ajout: &Ajout, liste: &[Abonnement], aujourdhui: Date) -> Abonnement {
    let (nom, prix, rythme, cat, modele) = match ajout {
        Ajout::Liste(m) => (m.nom.as_str(), m.prix, m.rythme, m.categorie.as_str(), Some(m.id.clone())),
        Ajout::Moi { nom, prix, rythme, categorie } => (nom.as_str(), *prix, *rythme, *categorie, None),
    };
    let mut a = Abonnement::nouveau(&id_libre(liste), nom, prix, rythme, premier_paiement(rythme, aujourdhui));
    // (le 31 janvier plus un mois tombe le 28 février : le jour voulu reste le 31)
    a.ancrage = aujourdhui.jour();
    a.categorie = categorie(cat).id.to_string();
    a.catalogue = modele;
    a
}

/// Ajoute l'abonnement à la liste et l'enregistre (abonnements.json, l'ancienne version dans « .bak »), puis met à
/// jour la carte de Mindset (etat.json). Pas enregistré : la liste revient comme avant, et le refus est un texte
/// simple pour la bulle de l'écran.
pub fn ajouter(d: &mut Donnees, ajout: &Ajout, aujourdhui: Date) -> Result<Abonnement, String> {
    let a = nouvel_abonnement(ajout, &d.abonnements.liste, aujourdhui);
    d.abonnements.liste.push(a.clone());
    if let Err(e) = d.abonnements.sauver(&mut d.fichiers) {
        d.abonnements.liste.pop();
        d.journal().ecrire(&format!("Ajout de {} refusé ({e})", a.nom));
        return Err("Paymind n'a pas pu enregistrer ta liste. Réessaie dans un moment.".into());
    }
    d.journal().ecrire(&format!(
        "Ajouté : {} ({}, {} {}, prochain paiement le {})",
        a.nom,
        a.catalogue.as_deref().unwrap_or("à moi"),
        euros_court(a.prix).replace('\u{a0}', " "),
        a.rythme.par_texte(),
        a.date.iso()
    ));
    d.ecrire_etat();
    Ok(a)
}

/// La phrase de la bulle après un ajout.
pub fn message_ajoute(a: &Abonnement) -> String {
    format!("{} est ajouté à tes cartes.", a.nom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use paymind_donnees::{Moment, horloge_figee};
    use paymind_moteur::{bilan, par_mois};
    use std::path::PathBuf;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    /// Un dossier de données d'essai neuf sous le dossier temporaire, effacé à la fin (jamais les vraies données)
    struct Essai(PathBuf);
    impl Essai {
        fn new(nom: &str) -> Essai {
            let p = std::env::temp_dir().join(format!("paymind-ajout-{}-{nom}", std::process::id()));
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

    /// Le scénario du plan : la liste vide, Netflix ajouté ; sa carte apparaît, le total du mois augmente de 7,99 €,
    /// abonnements.json et la carte de Mindset (etat.json) changent.
    #[test]
    fn netflix_ajoute_depuis_la_liste() {
        let auj = d(2026, 11, 15);
        let e = Essai::new("netflix");
        let mut dn = e.donnees();
        dn.ecrire_etat();
        assert!(e.lire("etat.json").contains("Aucun abonnement pour l'instant"));
        assert!(dn.abonnements.liste.is_empty());

        let ajout = lire(ACTION_LISTE, &json!("netflix")).unwrap();
        let a = ajouter(&mut dn, &ajout, auj).unwrap();
        assert_eq!((a.id.as_str(), a.nom.as_str(), a.prix, a.rythme), ("a1", "Netflix", 799, Rythme::Mois));
        assert_eq!(a.catalogue.as_deref(), Some("netflix"));
        assert_eq!(a.categorie, "films");
        assert_eq!(a.date, d(2026, 12, 15));
        assert!(a.auto && !a.pause);
        assert_eq!(a.rappel, paymind_moteur::Rappel::Veille);
        // Le total du mois : 0 puis 7,99 €
        assert_eq!(bilan(&dn.abonnements.liste, auj).par_mois, 799);
        // Enregistré : relu depuis le disque
        let relu = e.donnees();
        assert_eq!(relu.abonnements.liste, vec![a.clone()]);
        assert!(e.lire("abonnements.json").contains("\"catalogue\": \"netflix\""));
        // La carte de Mindset
        let etat = e.lire("etat.json");
        assert!(etat.contains("7,99\u{a0}€ par mois"), "{etat}");
        assert!(etat.contains("1 abonnement"), "{etat}");
        assert!(
            e.lire("journal.txt")
                .contains("Ajouté : Netflix (netflix, 7,99 € par mois, prochain paiement le 2026-12-15)")
        );
        // La carte apparaît dans le portefeuille, et « Déjà là » dans la liste
        let p = crate::portefeuille::portefeuille(&dn.abonnements.liste, auj);
        assert_eq!(p["groupes"][0]["cartes"][0]["nom"], "Netflix");
        assert_eq!(p["total"]["mois"], "7,99\u{a0}€");
        let v = vue(&dn.abonnements.liste);
        assert_eq!(v["compte"], "1");
        assert_eq!(v["groupes"][0]["elements"][0]["id"], "netflix");
        assert_eq!(v["groupes"][0]["elements"][0]["deja"], 1);

        // Deux comptes Netflix : permis, deux cartes, le total augmente encore
        let avant = bilan(&dn.abonnements.liste, auj).par_an;
        let b = ajouter(&mut dn, &ajout, auj).unwrap();
        assert_eq!(b.id, "a2");
        assert_eq!(dn.abonnements.liste.len(), 2);
        assert_eq!(bilan(&dn.abonnements.liste, auj).par_an, avant + 12 * 799);
        assert_eq!(vue(&dn.abonnements.liste)["groupes"][0]["elements"][0]["deja"], 2);
        assert!(e.lire("etat.json").contains("15,98\u{a0}€ par mois"));
        assert_eq!(par_mois(bilan(&dn.abonnements.liste, auj).par_an), 1598);
    }

    #[test]
    fn un_abonnement_a_moi() {
        let auj = d(2026, 11, 15);
        let e = Essai::new("moi");
        let mut dn = e.donnees();
        let v = json!({ "nom": "  Club   de foot \u{2014} enfants ", "prix": "40", "rythme": "trimestre", "categorie": "moi" });
        let ajout = lire(ACTION_MOI, &v).unwrap();
        assert_eq!(
            ajout,
            Ajout::Moi {
                nom: "Club de foot - enfants".into(),
                prix: 4000,
                rythme: Rythme::Trimestre,
                categorie: "moi"
            }
        );
        let a = ajouter(&mut dn, &ajout, auj).unwrap();
        assert_eq!(a.catalogue, None);
        assert_eq!(a.categorie, "moi");
        assert_eq!(a.date, d(2027, 2, 15));
        // Une autre catégorie au choix, un prix avec des centimes, chaque semaine
        let v = json!({ "nom": "Panier de légumes", "prix": "12,50 €", "rythme": "semaine", "categorie": "courses" });
        let a = ajouter(&mut dn, &lire(ACTION_MOI, &v).unwrap(), auj).unwrap();
        assert_eq!((a.id.as_str(), a.prix, a.date, a.categorie.as_str()), ("a2", 1250, d(2026, 11, 22), "courses"));
        assert_eq!(e.donnees().abonnements.liste.len(), 2);
        let p = crate::portefeuille::portefeuille(&dn.abonnements.liste, auj);
        assert_eq!(p["groupes"][0]["nom"], "Courses et livraison");
        assert_eq!(p["groupes"][1]["nom"], "À moi");
    }

    #[test]
    fn premier_paiement_un_rythme_plus_tard() {
        assert_eq!(premier_paiement(Rythme::Mois, d(2026, 11, 15)), d(2026, 12, 15));
        assert_eq!(premier_paiement(Rythme::An, d(2026, 11, 15)), d(2027, 11, 15));
        assert_eq!(premier_paiement(Rythme::Semaine, d(2026, 12, 29)), d(2027, 1, 5));
        assert_eq!(premier_paiement(Rythme::Trimestre, d(2026, 11, 30)), d(2027, 2, 28));
        // Le 31 janvier : le 28 février, et le jour voulu reste le 31
        let m = catalogue().modele("spotify").unwrap();
        let a = nouvel_abonnement(&Ajout::Liste(m), &[], d(2027, 1, 31));
        assert_eq!((a.date, a.ancrage), (d(2027, 2, 28), 31));
    }

    /// Jamais confiance dans la page : chaque geste mal formé est refusé, et rien n'est ajouté.
    #[test]
    fn arguments_faux_refuses() {
        let bon = json!({ "nom": "Club", "prix": "40", "rythme": "mois", "categorie": "moi" });
        let avec = |k: &str, v: Value| {
            let mut o = bon.clone();
            o[k] = v;
            o
        };
        let sans = |k: &str| {
            let mut o = bon.clone();
            o.as_object_mut().unwrap().remove(k);
            o
        };
        assert!(lire(ACTION_MOI, &bon).is_ok());
        for (id, v) in [
            (ACTION_LISTE, json!("inconnu")),
            (ACTION_LISTE, json!("../netflix")),
            (ACTION_LISTE, json!(7)),
            (ACTION_LISTE, json!(["netflix"])),
            (ACTION_LISTE, json!({ "id": "netflix" })),
            (ACTION_LISTE, Value::Null),
            (ACTION_MOI, json!("Club")),
            (ACTION_MOI, Value::Null),
            (ACTION_MOI, avec("nom", json!(""))),
            (ACTION_MOI, avec("nom", json!("   "))),
            (ACTION_MOI, avec("nom", json!("x".repeat(NOM_MAX + 1)))),
            (ACTION_MOI, avec("nom", json!("Club\u{0}"))),
            (ACTION_MOI, avec("nom", json!(3))),
            (ACTION_MOI, avec("prix", json!("0"))),
            (ACTION_MOI, avec("prix", json!("-5"))),
            (ACTION_MOI, avec("prix", json!("douze"))),
            (ACTION_MOI, avec("prix", json!("1".repeat(30)))),
            (ACTION_MOI, avec("prix", json!(12))),
            (ACTION_MOI, avec("rythme", json!("jour"))),
            (ACTION_MOI, avec("categorie", json!("autres"))),
            (ACTION_MOI, avec("categorie", json!("video"))),
            (ACTION_MOI, avec("en_plus", json!(1))),
            (ACTION_MOI, sans("categorie")),
            ("ajouter-tout", json!("netflix")),
        ] {
            assert!(lire(id, &v).is_err(), "{id} {v} accepté");
        }
        // Les refus disent quoi faire, en mots simples
        assert_eq!(lire(ACTION_MOI, &avec("nom", json!(" "))).unwrap_err(), "Écris le nom de l'abonnement.");
        assert_eq!(lire(ACTION_MOI, &avec("prix", json!("abc"))).unwrap_err(), "Écris le prix, par exemple 12,50.");
        // Rien n'est enregistré pour un refus ; un nom de 80 lettres passe
        assert!(lire(ACTION_MOI, &avec("nom", json!("é".repeat(NOM_MAX)))).is_ok());
        assert!(est_ajout(ACTION_LISTE) && est_ajout(ACTION_MOI) && !est_ajout("ajouter-tout"));
    }

    /// Une liste qui ne peut pas être enregistrée (une version plus récente de Paymind) : rien n'est ajouté.
    #[test]
    fn liste_en_lecture_seule() {
        let e = Essai::new("lecture-seule");
        std::fs::write(e.0.join("abonnements.json"), r#"{ "format": 99, "abonnements": [] }"#).unwrap();
        let mut dn = e.donnees();
        let ajout = lire(ACTION_LISTE, &json!("netflix")).unwrap();
        let r = ajouter(&mut dn, &ajout, d(2026, 11, 15));
        assert_eq!(r.unwrap_err(), "Paymind n'a pas pu enregistrer ta liste. Réessaie dans un moment.");
        assert!(dn.abonnements.liste.is_empty());
        assert!(e.lire("abonnements.json").contains("\"format\": 99"));
    }

    #[test]
    fn la_liste_toute_prete_a_l_ecran() {
        let v = vue(&[]);
        let groupes = v["groupes"].as_array().unwrap();
        assert_eq!(groupes.len(), 11);
        assert_eq!(groupes.iter().map(|g| g["elements"].as_array().unwrap().len()).sum::<usize>(), 88);
        assert_eq!(groupes[0]["nom"], "Films et séries");
        let netflix = &groupes[0]["elements"][0];
        assert_eq!(netflix["prix"], "7,99\u{a0}€");
        assert_eq!(netflix["rythme"], "par mois");
        assert_eq!(netflix["deja"], 0);
        assert_eq!(netflix["verifier"], false);
        assert_eq!(v["compte"], "0");
        assert_eq!(
            v["note"],
            "Les prix habituels ont été relevés en novembre 2026. Tout reste sur ce PC : rien n'est envoyé sur Internet."
        );
        let cats = v["moi"]["categories"].as_array().unwrap();
        assert_eq!(cats[0], json!(["moi", "À moi"]));
        assert_eq!(cats.len(), 12);
        // Aucun tiret long ni chevron dans ce que la page montre
        fn textes(v: &Value, l: &mut Vec<String>) {
            match v {
                Value::String(s) => l.push(s.clone()),
                Value::Array(a) => a.iter().for_each(|x| textes(x, l)),
                Value::Object(o) => o.values().for_each(|x| textes(x, l)),
                _ => {}
            }
        }
        let mut l = Vec::new();
        textes(&v, &mut l);
        assert!(l.len() > 800);
        for t in l {
            assert!(!t.contains(['\u{2013}', '\u{2014}', '<', '>']), "{t}");
        }
    }
}
