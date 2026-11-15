//! Les dates des paiements d'un abonnement, et comment elles avancent.
//!
//! La suite des paiements part de la date gardée (le paiement numéro 0) :
//! - par semaine : 7 jours de plus à chaque fois ;
//! - par mois, trimestre ou an : 1, 3 ou 12 mois de plus, au jour voulu (l'ancrage), ramené au dernier jour du mois
//!   s'il n'y existe pas. Chaque paiement est calculé depuis la date gardée (jamais de proche en proche) : le 31
//!   janvier donne le 28 février, puis le 31 mars, et non le 28 mars.
//!
//! Renouvellement automatique : une date passée avance jusqu'au premier paiement à venir (aujourd'hui compris), même
//! après des mois de PC éteint, en une fois. Un abonnement en pause avance aussi (son calendrier continue : à la
//! reprise, il retombe au bon jour).
//!
//! Sans renouvellement automatique : une date passée n'avance jamais seule. L'abonnement devient « à renouveler »
//! ([`crate::Statut::ARenouveler`]) : il ne compte plus dans les totaux ni dans « Bientôt », et n'a plus de prochain
//! paiement, jusqu'à ce que la personne le renouvelle ([`renouveler`]) ou le supprime. La date gardée reste celle du
//! dernier paiement prévu, pour pouvoir dire « à renouveler depuis le 3 novembre ».

use crate::abonnement::{Abonnement, Rythme};
use crate::date::Date;

/// Le paiement numéro `k` de la suite (0 : la date gardée) ; rien au-delà de l'an 9999.
pub fn paiement_numero(a: &Abonnement, k: i64) -> Option<Date> {
    if k == 0 {
        return Some(a.date);
    }
    match a.rythme.mois() {
        None => a.date.plus_jours(k.checked_mul(7)?),
        Some(m) => a.date.plus_mois(k.checked_mul(m)?, a.ancrage),
    }
}

/// Le numéro du premier paiement de la suite qui tombe ce jour-là ou après (0 si la date gardée n'est pas passée).
fn premier_numero_a_partir(a: &Abonnement, jour: Date) -> Option<i64> {
    if a.date >= jour {
        return Some(0);
    }
    let mut k = match a.rythme {
        Rythme::Semaine => (a.date.jours_jusqua(jour) + 6) / 7,
        // (le paiement numéro k tombe dans le mois date + k × pas : avant k = écart / pas, il est dans un mois passé)
        _ => {
            let pas = a.rythme.mois().unwrap_or(1);
            (a.date.mois_jusqua(jour) / pas).max(1)
        }
    };
    loop {
        if paiement_numero(a, k)? >= jour {
            return Some(k);
        }
        k += 1;
    }
}

/// Le prochain paiement (aujourd'hui compris), sans tenir compte de la pause : renouvellement automatique, le premier
/// paiement à venir de la suite ; sinon la date gardée, si elle n'est pas passée.
pub fn prochaine_echeance(a: &Abonnement, aujourdhui: Date) -> Option<Date> {
    if !a.auto {
        return (a.date >= aujourdhui).then_some(a.date);
    }
    paiement_numero(a, premier_numero_a_partir(a, aujourdhui)?)
}

/// Les paiements de `de` à `a` (compris), dans l'ordre. Sans renouvellement automatique, seulement la date gardée.
pub fn paiements_entre(ab: &Abonnement, de: Date, a: Date) -> Vec<Date> {
    let mut l = Vec::new();
    if !ab.auto {
        if ab.date >= de && ab.date <= a {
            l.push(ab.date);
        }
        return l;
    }
    let Some(mut k) = premier_numero_a_partir(ab, de) else { return l };
    while let Some(d) = paiement_numero(ab, k) {
        if d > a {
            break;
        }
        l.push(d);
        k += 1;
    }
    l
}

/// Avance la date d'un abonnement à renouvellement automatique dont la date est passée, jusqu'au premier paiement à
/// venir (aujourd'hui compris). Vrai si elle a changé. Le jour voulu ne change jamais.
pub fn avancer(a: &mut Abonnement, aujourdhui: Date) -> bool {
    if !a.auto || a.date >= aujourdhui {
        return false;
    }
    match prochaine_echeance(a, aujourdhui) {
        Some(d) => {
            a.date = d;
            true
        }
        None => false,
    }
}

/// [`avancer`] pour toute la liste ; le nombre d'abonnements dont la date a changé.
pub fn avancer_tout(liste: &mut [Abonnement], aujourdhui: Date) -> usize {
    liste.iter_mut().map(|a| avancer(a, aujourdhui)).filter(|&c| c).count()
}

/// Renouvelle à la main un abonnement « à renouveler » : sa date passe au premier paiement à venir de sa suite
/// (aujourd'hui compris), au même jour voulu. Sans effet si sa date n'est pas passée. Vrai si elle a changé.
pub fn renouveler(a: &mut Abonnement, aujourdhui: Date) -> bool {
    if a.date >= aujourdhui {
        return false;
    }
    let auto = std::mem::replace(&mut a.auto, true);
    let change = avancer(a, aujourdhui);
    a.auto = auto;
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    #[test]
    fn la_suite_part_toujours_de_la_date_gardee() {
        let mut a = Abonnement::nouveau("a1", "x", 100, Rythme::Mois, d(2026, 1, 31));
        assert_eq!(paiement_numero(&a, 1), Some(d(2026, 2, 28)));
        assert_eq!(paiement_numero(&a, 2), Some(d(2026, 3, 31)));
        assert_eq!(paiement_numero(&a, 3), Some(d(2026, 4, 30)));
        a.rythme = Rythme::Semaine;
        assert_eq!(paiement_numero(&a, 1), Some(d(2026, 2, 7)));
    }

    #[test]
    fn rien_au_dela_de_9999() {
        let a = Abonnement::nouveau("a1", "x", 100, Rythme::An, d(9999, 6, 1));
        assert_eq!(prochaine_echeance(&a, d(9999, 7, 1)), None);
        let mut b = a.clone();
        assert!(!avancer(&mut b, d(9999, 7, 1)));
        assert_eq!(b, a);
    }
}
