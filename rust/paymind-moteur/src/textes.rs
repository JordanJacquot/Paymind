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
}
