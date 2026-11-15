//! Un moment en heure locale (donné par l'appelant : ce crate ne lit jamais l'horloge de Windows), écrit comme
//! `Get-Date -Format s` (le journal, etat.json). Repris de mindcare-donnees. Les dates des abonnements sont celles de
//! paymind-moteur ([`Moment::date`], [`Moment::instant`]).

use std::fmt;

/// Un moment en heure locale, à la seconde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment {
    pub annee: i32,
    pub mois: u32,
    pub jour: u32,
    pub heure: u32,
    pub minute: u32,
    pub seconde: u32,
}

/// Une année bissextile (calendrier grégorien).
fn bissextile(a: i32) -> bool {
    (a % 4 == 0 && a % 100 != 0) || a % 400 == 0
}

/// Le nombre de jours d'un mois.
fn jours_du_mois(a: i32, m: u32) -> u32 {
    match m {
        2 if bissextile(a) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Moment {
    /// Un moment, s'il existe (1er janvier de l'an 1 au 31 décembre 9999, comme .NET).
    pub fn nouveau(annee: i32, mois: u32, jour: u32, heure: u32, minute: u32, seconde: u32) -> Option<Moment> {
        let ok = (1..=9999).contains(&annee)
            && (1..=12).contains(&mois)
            && jour >= 1
            && jour <= jours_du_mois(annee, mois)
            && heure < 24
            && minute < 60
            && seconde < 60;
        ok.then_some(Moment { annee, mois, jour, heure, minute, seconde })
    }

    /// Le jour de ce moment, pour le calcul des comptes (un moment fait à la main qui n'existe pas est ramené au 1er
    /// de son mois, ou au 1er janvier 1970).
    pub fn date(&self) -> paymind_moteur::Date {
        use paymind_moteur::Date;
        Date::nouvelle(self.annee, self.mois, self.jour)
            .or_else(|| Date::nouvelle(self.annee, self.mois, 1))
            .unwrap_or(Date::EPOQUE)
    }

    /// L'instant de ce moment (à la minute), pour les rappels.
    pub fn instant(&self) -> paymind_moteur::Instant {
        paymind_moteur::Instant::nouveau(self.date(), self.heure, self.minute)
    }

    /// `Get-Date -Format 'yyyyMMdd-HHmmss'` : « 20261114-140500 » (le nom d'un fichier mis de côté).
    pub fn compacte(&self) -> String {
        format!(
            "{:04}{:02}{:02}-{:02}{:02}{:02}",
            self.annee, self.mois, self.jour, self.heure, self.minute, self.seconde
        )
    }

    /// `Get-Date -Format s` : « 2026-11-14T14:05:00 ».
    pub fn iso(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.annee, self.mois, self.jour, self.heure, self.minute, self.seconde
        )
    }
}

impl fmt::Display for Moment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.iso())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moments_valables_ou_non() {
        assert_eq!(Moment::nouveau(2026, 11, 14, 9, 5, 0).unwrap().iso(), "2026-11-14T09:05:00");
        assert!(Moment::nouveau(2028, 2, 29, 0, 0, 0).is_some());
        assert!(Moment::nouveau(2026, 2, 29, 0, 0, 0).is_none());
        assert!(Moment::nouveau(2026, 4, 31, 0, 0, 0).is_none());
        assert!(Moment::nouveau(2026, 13, 1, 0, 0, 0).is_none());
        assert!(Moment::nouveau(2026, 1, 1, 24, 0, 0).is_none());
    }
}
