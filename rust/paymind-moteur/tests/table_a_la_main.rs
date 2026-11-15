//! Le test « jamais vu » : 22 dates calculées à la main, une par une, avec un calendrier papier (sans regarder le
//! code), comparées à ce que donne le moteur. Chaque ligne dit pourquoi.

use paymind_moteur::{Abonnement, Date, Rythme, avancer, prochaine_echeance};

fn d(a: i32, m: u32, j: u32) -> Date {
    Date::nouvelle(a, m, j).unwrap()
}

/// (rythme, date gardée, jour voulu, renouvellement automatique, aujourd'hui, prochain paiement attendu, pourquoi)
type Cas = (Rythme, Date, u32, bool, Date, Option<Date>, &'static str);

fn table() -> Vec<Cas> {
    use Rythme::*;
    vec![
        (
            Mois,
            d(2026, 1, 31),
            31,
            true,
            d(2026, 2, 1),
            Some(d(2026, 2, 28)),
            "31 janvier, février 2026 n'a que 28 jours",
        ),
        (Mois, d(2026, 1, 31), 31, true, d(2026, 3, 1), Some(d(2026, 3, 31)), "28 février passé, mars a bien un 31"),
        (Mois, d(2026, 3, 31), 31, true, d(2026, 4, 1), Some(d(2026, 4, 30)), "avril a 30 jours"),
        (Mois, d(2026, 4, 30), 31, true, d(2026, 5, 1), Some(d(2026, 5, 31)), "ancré au 31 : retour au 31 en mai"),
        (Mois, d(2026, 1, 30), 30, true, d(2026, 3, 1), Some(d(2026, 3, 30)), "30 janvier, 28 février passé, 30 mars"),
        (An, d(2028, 2, 29), 29, true, d(2028, 3, 1), Some(d(2029, 2, 28)), "2029 n'est pas bissextile"),
        (
            An,
            d(2028, 2, 29),
            29,
            true,
            d(2031, 3, 1),
            Some(d(2032, 2, 29)),
            "2029, 2030, 2031 au 28 passés ; 2032 bissextile",
        ),
        (An, d(2031, 2, 28), 29, true, d(2031, 3, 1), Some(d(2032, 2, 29)), "ancré au 29 : revient au 29 en 2032"),
        (An, d(2099, 2, 28), 29, true, d(2099, 3, 1), Some(d(2100, 2, 28)), "2100 n'est pas bissextile (siècle)"),
        (An, d(1999, 2, 28), 29, true, d(1999, 3, 1), Some(d(2000, 2, 29)), "2000 est bissextile (divisible par 400)"),
        (Mois, d(1900, 1, 29), 29, true, d(1900, 2, 1), Some(d(1900, 2, 28)), "1900 n'est pas bissextile"),
        (Semaine, d(2026, 12, 28), 28, true, d(2026, 12, 29), Some(d(2027, 1, 4)), "lundi 28 décembre + 7 jours"),
        (Semaine, d(2026, 12, 31), 31, true, d(2027, 1, 1), Some(d(2027, 1, 7)), "jeudi 31 décembre + 7 jours"),
        (Trimestre, d(2026, 1, 31), 31, true, d(2026, 2, 1), Some(d(2026, 4, 30)), "trimestre : avril, 30 jours"),
        (Trimestre, d(2026, 1, 31), 31, true, d(2026, 5, 1), Some(d(2026, 7, 31)), "30 avril passé, 31 juillet"),
        (Trimestre, d(2026, 11, 30), 31, true, d(2026, 12, 1), Some(d(2027, 2, 28)), "trimestre qui passe l'année"),
        (Mois, d(2026, 12, 15), 15, true, d(2026, 12, 16), Some(d(2027, 1, 15)), "passage d'année"),
        (
            Mois,
            d(2025, 9, 15),
            15,
            true,
            d(2026, 11, 16),
            Some(d(2026, 12, 15)),
            "PC éteint 14 mois : 15 novembre passé d'un jour",
        ),
        (
            Mois,
            d(2026, 11, 15),
            15,
            true,
            d(2026, 11, 15),
            Some(d(2026, 11, 15)),
            "le paiement est aujourd'hui : il reste",
        ),
        (Mois, d(2026, 11, 10), 10, false, d(2026, 11, 15), None, "sans renouvellement automatique : à renouveler"),
        (
            Semaine,
            d(2026, 11, 1),
            1,
            true,
            d(2026, 11, 15),
            Some(d(2026, 11, 15)),
            "dimanche 1er + 14 jours tombe aujourd'hui",
        ),
        (An, d(2026, 12, 31), 31, true, d(2027, 1, 1), Some(d(2027, 12, 31)), "un an plus tard, 31 décembre"),
    ]
}

#[test]
fn vingt_deux_dates_calculees_a_la_main() {
    let t = table();
    assert!(t.len() >= 20);
    for (rythme, date, ancrage, auto, auj, attendu, pourquoi) in t {
        let mut a = Abonnement::nouveau("a1", "Essai", 999, rythme, date);
        a.ancrage = ancrage;
        a.auto = auto;
        assert_eq!(prochaine_echeance(&a, auj), attendu, "{pourquoi}");
        let change = avancer(&mut a, auj);
        match attendu {
            Some(x) => {
                assert_eq!(a.date, x, "{pourquoi}");
                assert_eq!(change, x != date, "{pourquoi}");
            }
            // (sans renouvellement : la date reste)
            None => assert!(!change && a.date == date, "{pourquoi}"),
        }
        assert_eq!(a.ancrage, ancrage, "le jour voulu ne change jamais : {pourquoi}");
    }
}

#[test]
fn jours_de_la_semaine_verifies_au_calendrier() {
    // 0 dimanche ... 6 samedi
    assert_eq!(d(2026, 11, 15).jour_semaine(), 0);
    assert_eq!(d(2026, 12, 28).jour_semaine(), 1);
    assert_eq!(d(2026, 12, 31).jour_semaine(), 4);
    assert_eq!(d(2027, 1, 9).jour_semaine(), 6);
    assert_eq!(d(2000, 2, 29).jour_semaine(), 2);
    assert_eq!(d(1900, 1, 1).jour_semaine(), 1);
    assert_eq!(d(2100, 3, 1).jour_semaine(), 1);
}
