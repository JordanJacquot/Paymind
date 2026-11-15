//! Les textes de l'écran, en mots simples (aucun tiret long ni demi-cadratin) : les montants (« 13,49 € »), les
//! délais (« dans 3 jours », « demain », « aujourd'hui »), les dates (« le mardi 17 novembre »).
//!
//! Les espaces des montants (milliers, avant « € ») sont insécables (U+00A0) : « 1 234,50 € » ne se coupe jamais en
//! fin de ligne.

use crate::abonnement::PRIX_MAX;
use crate::bilan::{BIENTOT_JOURS, Paiement};
use crate::date::Date;
use crate::rappels::RappelDu;

/// L'espace insécable des montants.
pub const ESPACE: char = '\u{a0}';

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
const MOIS_COURTS: [&str; 12] =
    ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."];
const JOURS: [&str; 7] = ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"];

/// Les euros d'un montant, avec un espace insécable tous les trois chiffres : 1234 donne « 1 234 ».
fn milliers(n: u64) -> String {
    let t = n.to_string();
    let mut s = String::new();
    for (i, c) in t.chars().enumerate() {
        if i > 0 && (t.len() - i).is_multiple_of(3) {
            s.push(ESPACE);
        }
        s.push(c);
    }
    s
}

/// Un montant en centimes, toujours avec ses centimes : 1349 donne « 13,49 € », 8700 « 87,00 € ».
pub fn euros(centimes: u64) -> String {
    format!("{},{:02}{ESPACE}€", milliers(centimes / 100), centimes % 100)
}

/// Un montant sans « ,00 » quand il tombe juste : 8700 donne « 87 € », 1349 « 13,49 € ».
pub fn euros_court(centimes: u64) -> String {
    if centimes.is_multiple_of(100) { format!("{}{ESPACE}€", milliers(centimes / 100)) } else { euros(centimes) }
}

/// Un prix tapé par la personne, en centimes : « 13,49 », « 13.49 », « 13,5 », « 13 », « 1 234,50 », « 13,49 € ».
/// Rien s'il n'est pas lisible, négatif, a plus de deux chiffres après la virgule ou dépasse le prix le plus haut.
pub fn lire_prix(t: &str) -> Option<u64> {
    let t: String = t.chars().filter(|c| !c.is_whitespace() && *c != '€' && *c != '\u{202f}').collect();
    let (e, c) = match t.find([',', '.']) {
        Some(i) => (&t[..i], &t[i + 1..]),
        None => (&t[..], ""),
    };
    let chiffres = |p: &str| p.bytes().all(|b| b.is_ascii_digit());
    if (e.is_empty() && c.is_empty()) || !chiffres(e) || !chiffres(c) || c.len() > 2 || e.len() > 9 {
        return None;
    }
    let euros: u64 = if e.is_empty() { 0 } else { e.parse().ok()? };
    let cents: u64 = match c.len() {
        0 => 0,
        1 => c.parse::<u64>().ok()? * 10,
        _ => c.parse().ok()?,
    };
    let total = euros.checked_mul(100)?.checked_add(cents)?;
    (total <= PRIX_MAX).then_some(total)
}

/// « 17 nov. » (les pastilles de « Bientôt »).
pub fn date_courte(d: Date) -> String {
    format!("{} {}", d.jour(), MOIS_COURTS[(d.mois() - 1) as usize])
}

/// « le 1er » ou « le 17 ».
fn numero(jour: u32) -> String {
    if jour == 1 { "1er".to_string() } else { jour.to_string() }
}

/// « mardi 17 novembre », avec l'année si ce n'est pas celle d'aujourd'hui (« samedi 9 janvier 2027 »).
pub fn date_longue_sans_le(d: Date, aujourdhui: Date) -> String {
    let mut s = format!("{} {} {}", JOURS[d.jour_semaine() as usize], numero(d.jour()), MOIS[(d.mois() - 1) as usize]);
    if d.annee() != aujourdhui.annee() {
        s.push_str(&format!(" {}", d.annee()));
    }
    s
}

/// « le mardi 17 novembre », « le mardi 1er décembre », « le samedi 9 janvier 2027 ».
pub fn date_longue(d: Date, aujourdhui: Date) -> String {
    format!("le {}", date_longue_sans_le(d, aujourdhui))
}

/// « Mardi 17 novembre » (le haut de la fiche).
pub fn date_fiche(d: Date, aujourdhui: Date) -> String {
    let s = date_longue_sans_le(d, aujourdhui);
    let mut c = s.chars();
    c.next().map(|p| p.to_uppercase().chain(c).collect()).unwrap_or_default()
}

/// « aujourd'hui », « demain », « dans 3 jours », « hier », « il y a 3 jours ».
pub fn dans(d: Date, aujourdhui: Date) -> String {
    match aujourdhui.jours_jusqua(d) {
        0 => "aujourd'hui".to_string(),
        1 => "demain".to_string(),
        -1 => "hier".to_string(),
        n if n > 1 => format!("dans {n} jours"),
        n => format!("il y a {} jours", -n),
    }
}

/// Quand, pour un paiement à venir : « dans 3 jours » jusqu'à 30 jours, puis la date (« le samedi 9 janvier 2027 »).
pub fn quand(d: Date, aujourdhui: Date) -> String {
    let n = aujourdhui.jours_jusqua(d);
    if (-1..=BIENTOT_JOURS).contains(&n) { dans(d, aujourdhui) } else { date_longue(d, aujourdhui) }
}

/// Le prochain paiement, pour la carte de Mindset : « Netflix dans 3 jours ».
pub fn texte_prochain(p: &Paiement, aujourdhui: Date) -> String {
    format!("{} {}", p.nom, quand(p.date, aujourdhui))
}

/// La bulle d'un rappel : « Netflix : 13,49 € demain », « Netflix : 13,49 € le mardi 17 novembre » ; manqué pendant
/// que le PC était éteint : « Netflix : 13,49 € payé hier » ; sans renouvellement automatique : « Netflix : à
/// renouveler demain », « Netflix : à renouveler depuis le mardi 3 novembre ».
pub fn texte_rappel(r: &RappelDu, aujourdhui: Date) -> String {
    let n = aujourdhui.jours_jusqua(r.paiement);
    let quand = if (-1..=1).contains(&n) { dans(r.paiement, aujourdhui) } else { date_longue(r.paiement, aujourdhui) };
    match (r.auto, n >= 0) {
        (true, true) => format!("{} : {} {quand}", r.nom, euros_court(r.prix)),
        (true, false) => format!("{} : {} payé {quand}", r.nom, euros_court(r.prix)),
        (false, true) => format!("{} : à renouveler {quand}", r.nom),
        (false, false) if n == -1 => format!("{} : à renouveler depuis hier", r.nom),
        (false, false) => format!("{} : à renouveler depuis {quand}", r.nom),
    }
}

/// La longueur d'un titre de bulle que Mindset montre en entier (il coupe au-delà, en unités UTF-16).
pub const BULLE_TITRE_MAX: usize = 63;
/// La longueur d'un texte de bulle que Mindset montre en entier.
pub const BULLE_TEXTE_MAX: usize = 250;
/// La longueur d'un nom dans une bulle (au-delà : « ... »).
const BULLE_NOM_MAX: usize = 28;

/// La longueur d'un texte comme Mindset la mesure (unités UTF-16, `.Length` de .NET).
fn longueur(t: &str) -> usize {
    t.encode_utf16().count()
}

/// Un texte de `max` unités UTF-16 au plus (la mesure de Mindset), coupé avec « ... » s'il est plus long.
pub fn couper(t: &str, max: usize) -> String {
    if longueur(t) <= max {
        return t.to_string();
    }
    let mut s = String::new();
    for c in t.chars() {
        if longueur(&s) + c.len_utf16() + 3 > max {
            break;
        }
        s.push(c);
    }
    s.trim_end().to_string() + "..."
}

/// Quand, dans une bulle : « aujourd'hui », « demain », « dans 3 jours » (jusqu'à 7 jours), « hier » ; plus loin, la
/// date (« le samedi 9 janvier 2027 »).
fn quand_bulle(d: Date, aujourdhui: Date) -> String {
    if (-1..=7).contains(&aujourdhui.jours_jusqua(d)) { dans(d, aujourdhui) } else { date_longue(d, aujourdhui) }
}

/// Un rappel dans une bulle qui en regroupe plusieurs : « Netflix demain (13,49 €) », « Salle à renouveler dans 3
/// jours ».
fn rappel_court(r: &RappelDu, aujourdhui: Date) -> String {
    let nom = couper(&r.nom, BULLE_NOM_MAX);
    let q = quand_bulle(r.paiement, aujourdhui);
    if r.auto { format!("{nom} {q} ({})", euros_court(r.prix)) } else { format!("{nom} à renouveler {q}") }
}

/// La fin du texte d'une bulle qui regroupe des rappels, quand `n` ne tiennent plus : « . », « et 1 autre. »...
fn fin_bulle(n: usize) -> String {
    match n {
        0 => ".".to_string(),
        1 => " et 1 autre.".to_string(),
        n => format!(" et {n} autres."),
    }
}

/// La bulle près de l'horloge pour les rappels dus en même temps (titre, texte), en mots simples, sans tiret long :
///
/// - un seul : « Netflix demain : 13,49 € » et « Paiement prévu le lundi 16 novembre. » ; sans renouvellement
///   automatique : « Salle à renouveler demain » et « Cet abonnement ne se renouvelle pas tout seul. Date prévue : le
///   lundi 16 novembre. » ;
/// - plusieurs : une seule bulle, « 3 rappels de paiement » et « Netflix demain (13,49 €), Spotify dans 3 jours
///   (11,12 €), Salle à renouveler aujourd'hui. », par date de paiement ; trop long, la fin devient « et 2 autres. ».
///
/// Le titre tient en 63 unités UTF-16 et le texte en 250 (Mindset couperait au-delà). Rien pour une liste vide.
pub fn bulle_rappels(rappels: &[RappelDu], aujourdhui: Date) -> Option<(String, String)> {
    let (titre, texte) = match rappels {
        [] => return None,
        [r] => {
            let nom = couper(&r.nom, BULLE_NOM_MAX);
            let q = quand_bulle(r.paiement, aujourdhui);
            let date = date_longue(r.paiement, aujourdhui);
            if r.auto {
                (format!("{nom} {q} : {}", euros_court(r.prix)), format!("Paiement prévu {date}."))
            } else {
                let titre = if aujourdhui.jours_jusqua(r.paiement) < 0 {
                    format!("{nom} à renouveler")
                } else {
                    format!("{nom} à renouveler {q}")
                };
                (titre, format!("Cet abonnement ne se renouvelle pas tout seul. Date prévue : {date}."))
            }
        }
        _ => {
            let mut l: Vec<&RappelDu> = rappels.iter().collect();
            l.sort_by(|x, y| (x.paiement, &x.nom, &x.id).cmp(&(y.paiement, &y.nom, &y.id)));
            let morceaux: Vec<String> = l.iter().map(|r| rappel_court(r, aujourdhui)).collect();
            let mut texte = String::new();
            let mut fin = fin_bulle(0);
            for (i, m) in morceaux.iter().enumerate() {
                let suivant = if i == 0 { m.clone() } else { format!("{texte}, {m}") };
                let reste = fin_bulle(morceaux.len() - i - 1);
                // (celui-ci ne tient plus : on s'arrête avant lui, « et N autres. » tenait au tour d'avant)
                if i > 0 && longueur(&suivant) + longueur(&reste) > BULLE_TEXTE_MAX {
                    fin = fin_bulle(morceaux.len() - i);
                    break;
                }
                texte = suivant;
            }
            (format!("{} rappels de paiement", l.len()), texte + &fin)
        }
    };
    Some((couper(&titre, BULLE_TITRE_MAX), couper(&texte, BULLE_TEXTE_MAX)))
}

/// « 1 abonnement », « 12 abonnements ».
pub fn nombre_abonnements(n: usize) -> String {
    if n <= 1 { format!("{n} abonnement") } else { format!("{n} abonnements") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    #[test]
    fn montants() {
        assert_eq!(euros(1349), "13,49\u{a0}€");
        assert_eq!(euros(8700), "87,00\u{a0}€");
        assert_eq!(euros(5), "0,05\u{a0}€");
        assert_eq!(euros(123_456), "1\u{a0}234,56\u{a0}€");
        assert_eq!(euros(99_999_999), "999\u{a0}999,99\u{a0}€");
        assert_eq!(euros_court(8700), "87\u{a0}€");
        assert_eq!(euros_court(1349), "13,49\u{a0}€");
        assert_eq!(euros_court(0), "0\u{a0}€");
    }

    #[test]
    fn prix_tapes() {
        assert_eq!(lire_prix("13,49"), Some(1349));
        assert_eq!(lire_prix("13.49"), Some(1349));
        assert_eq!(lire_prix(" 13,5 "), Some(1350));
        assert_eq!(lire_prix("13"), Some(1300));
        assert_eq!(lire_prix("13,"), Some(1300));
        assert_eq!(lire_prix(",5"), Some(50));
        assert_eq!(lire_prix("1 234,50 €"), Some(123_450));
        assert_eq!(lire_prix("1\u{a0}234,50"), Some(123_450));
        for faux in ["", ",", "-3", "13,499", "abc", "1,2,3", "1e3", "1000000", "+5"] {
            assert_eq!(lire_prix(faux), None, "{faux}");
        }
    }

    #[test]
    fn dates_et_delais() {
        let auj = d(2026, 11, 15);
        assert_eq!(date_longue(d(2026, 11, 17), auj), "le mardi 17 novembre");
        assert_eq!(date_longue(d(2026, 12, 1), auj), "le mardi 1er décembre");
        assert_eq!(date_longue(d(2027, 1, 9), auj), "le samedi 9 janvier 2027");
        assert_eq!(date_fiche(d(2026, 11, 17), auj), "Mardi 17 novembre");
        assert_eq!(date_courte(d(2026, 2, 3)), "3 févr.");
        assert_eq!(dans(auj, auj), "aujourd'hui");
        assert_eq!(dans(d(2026, 11, 16), auj), "demain");
        assert_eq!(dans(d(2026, 11, 18), auj), "dans 3 jours");
        assert_eq!(dans(d(2026, 11, 14), auj), "hier");
        assert_eq!(dans(d(2026, 11, 12), auj), "il y a 3 jours");
        assert_eq!(quand(d(2026, 12, 15), auj), "dans 30 jours");
        assert_eq!(quand(d(2026, 12, 16), auj), "le mercredi 16 décembre");
        assert_eq!(nombre_abonnements(0), "0 abonnement");
        assert_eq!(nombre_abonnements(1), "1 abonnement");
        assert_eq!(nombre_abonnements(12), "12 abonnements");
    }

    fn rappel(nom: &str, prix: u64, paiement: Date, auto: bool) -> RappelDu {
        use crate::{Instant, Rappel};
        RappelDu {
            id: format!("id-{nom}"),
            nom: nom.into(),
            prix,
            paiement,
            du: Instant::nouveau(paiement.plus_jours(-1).unwrap(), 9, 0),
            rappel: Rappel::Veille,
            auto,
        }
    }

    fn sans_tiret_long(t: &str) {
        assert!(!t.contains(['\u{2013}', '\u{2014}']), "{t}");
    }

    #[test]
    fn la_bulle_d_un_rappel() {
        let auj = d(2026, 11, 15);
        let b = |r: RappelDu| bulle_rappels(&[r], auj).unwrap();
        assert_eq!(
            b(rappel("Netflix", 1349, d(2026, 11, 16), true)),
            ("Netflix demain : 13,49\u{a0}€".into(), "Paiement prévu le lundi 16 novembre.".into())
        );
        assert_eq!(b(rappel("Netflix", 1349, auj, true)).0, "Netflix aujourd'hui : 13,49\u{a0}€");
        assert_eq!(b(rappel("Box", 2999, d(2026, 11, 22), true)).0, "Box dans 7 jours : 29,99\u{a0}€");
        // manqué pendant que le PC était éteint
        assert_eq!(
            b(rappel("Netflix", 1349, d(2026, 11, 14), true)),
            ("Netflix hier : 13,49\u{a0}€".into(), "Paiement prévu le samedi 14 novembre.".into())
        );
        assert_eq!(
            b(rappel("Assurance", 54_000, d(2026, 9, 1), true)).0,
            "Assurance le mardi 1er septembre : 540\u{a0}€"
        );
        // sans renouvellement automatique
        assert_eq!(
            b(rappel("Salle", 2999, d(2026, 11, 16), false)),
            (
                "Salle à renouveler demain".into(),
                "Cet abonnement ne se renouvelle pas tout seul. Date prévue : le lundi 16 novembre.".into()
            )
        );
        assert_eq!(b(rappel("Salle", 2999, d(2026, 11, 13), false)).0, "Salle à renouveler");
        assert!(bulle_rappels(&[], auj).is_none());
        // un nom très long : le titre tient quand même
        let (t, x) = b(rappel(&"Abonnement très long ".repeat(4), 99_999_999, d(2027, 1, 9), true));
        assert!(
            t.encode_utf16().count() <= BULLE_TITRE_MAX && t.starts_with("Abonnement très long") && t.contains("...")
        );
        sans_tiret_long(&t);
        sans_tiret_long(&x);
    }

    #[test]
    fn plusieurs_rappels_dans_une_seule_bulle() {
        let auj = d(2026, 11, 15);
        let l = [
            rappel("Spotify", 1112, d(2026, 11, 18), true),
            rappel("Netflix", 1349, d(2026, 11, 16), true),
            rappel("Salle", 2999, auj, false),
        ];
        assert_eq!(
            bulle_rappels(&l, auj).unwrap(),
            (
                "3 rappels de paiement".into(),
                "Salle à renouveler aujourd'hui, Netflix demain (13,49\u{a0}€), Spotify dans 3 jours (11,12\u{a0}€)."
                    .into()
            )
        );
        // Beaucoup : la fin dit combien ne tiennent pas, et le texte reste dans les 250
        let beaucoup: Vec<RappelDu> =
            (0..30).map(|i| rappel(&format!("Abonnement numéro {i}"), 1000 + i, d(2026, 11, 16), true)).collect();
        let (t, x) = bulle_rappels(&beaucoup, auj).unwrap();
        assert_eq!(t, "30 rappels de paiement");
        assert!(x.encode_utf16().count() <= BULLE_TEXTE_MAX, "{x}");
        assert!(x.ends_with(" autres.") && x.starts_with("Abonnement numéro 0 demain"), "{x}");
        let montres = x.matches("demain").count();
        assert!(x.ends_with(&format!(" et {} autres.", 30 - montres)), "{x}");
        sans_tiret_long(&x);
    }
}
