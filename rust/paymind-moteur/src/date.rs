//! Une date du calendrier grégorien, calculée à la main en jours civils (aucune bibliothèque, aucun fuseau horaire :
//! Paymind ne compte que des jours). Les années vont de 1 à 9999, comme .NET.
//!
//! Le nombre de jours depuis le 1er janvier 1970 se calcule avec la méthode « days from civil » de Howard Hinnant
//! (domaine public) : les années commencent en mars, le 29 février tombe alors à la fin de l'année.

use std::fmt;

/// La première et la dernière année acceptées.
pub const ANNEE_MIN: i32 = 1;
pub const ANNEE_MAX: i32 = 9999;

/// Une année bissextile : divisible par 4, sauf les siècles, sauf ceux divisibles par 400 (1900 non, 2000 oui,
/// 2100 non).
pub fn bissextile(annee: i32) -> bool {
    (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0
}

/// Le nombre de jours d'un mois (1 à 12).
pub fn jours_du_mois(annee: i32, mois: u32) -> u32 {
    match mois {
        2 if bissextile(annee) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Une date (année, mois de 1 à 12, jour de 1 à 31). Rangées dans l'ordre du calendrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    annee: i32,
    mois: u32,
    jour: u32,
}

impl Date {
    /// Le 1er janvier 1970.
    pub const EPOQUE: Date = Date { annee: 1970, mois: 1, jour: 1 };

    /// La date, si elle existe (pas de 31 avril ni de 29 février 2026).
    pub fn nouvelle(annee: i32, mois: u32, jour: u32) -> Option<Date> {
        let ok = (ANNEE_MIN..=ANNEE_MAX).contains(&annee)
            && (1..=12).contains(&mois)
            && jour >= 1
            && jour <= jours_du_mois(annee, mois);
        ok.then_some(Date { annee, mois, jour })
    }

    pub fn annee(&self) -> i32 {
        self.annee
    }

    pub fn mois(&self) -> u32 {
        self.mois
    }

    pub fn jour(&self) -> u32 {
        self.jour
    }

    /// Le nombre de jours depuis le 1er janvier 1970 (négatif avant).
    pub fn jours_depuis_1970(&self) -> i64 {
        let y = i64::from(self.annee) - i64::from(self.mois <= 2);
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400); // 0 à 399
        let m = i64::from(self.mois);
        let mp = (m + 9) % 12; // mars = 0, février = 11
        let doy = (153 * mp + 2) / 5 + i64::from(self.jour) - 1; // 0 à 365
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // 0 à 146096
        era * 146_097 + doe - 719_468
    }

    /// La date de ce jour compté depuis le 1er janvier 1970, si elle est entre l'an 1 et l'an 9999.
    pub fn depuis_jours(jours: i64) -> Option<Date> {
        let z = jours + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let jour = doy - (153 * mp + 2) / 5 + 1;
        let mois = if mp < 10 { mp + 3 } else { mp - 9 };
        let annee = yoe + era * 400 + i64::from(mois <= 2);
        let annee = i32::try_from(annee).ok()?;
        Date::nouvelle(annee, u32::try_from(mois).ok()?, u32::try_from(jour).ok()?)
    }

    /// Cette date plus (ou moins) ce nombre de jours ; rien en dehors des années 1 à 9999.
    pub fn plus_jours(&self, n: i64) -> Option<Date> {
        Date::depuis_jours(self.jours_depuis_1970().checked_add(n)?)
    }

    /// Le nombre de jours de `self` à `autre` (positif si `autre` est après).
    pub fn jours_jusqua(&self, autre: Date) -> i64 {
        autre.jours_depuis_1970() - self.jours_depuis_1970()
    }

    /// Le jour de la semaine : 0 dimanche, 1 lundi... 6 samedi (le 1er janvier 1970 était un jeudi).
    pub fn jour_semaine(&self) -> u32 {
        // (toujours de 0 à 6)
        (self.jours_depuis_1970() + 4).rem_euclid(7) as u32
    }

    /// Le dernier jour du mois de cette date.
    pub fn fin_du_mois(&self) -> Date {
        Date { jour: jours_du_mois(self.annee, self.mois), ..*self }
    }

    /// Ajoute `n` mois et prend le jour voulu `ancrage` (1 à 31), ramené au dernier jour du mois s'il n'y existe pas :
    /// le 31 janvier plus un mois donne le 28 (ou 29) février, et le 31 mars avec un ancrage au 31.
    pub fn plus_mois(&self, n: i64, ancrage: u32) -> Option<Date> {
        let total = i64::from(self.annee) * 12 + i64::from(self.mois) - 1 + n;
        let annee = i32::try_from(total.div_euclid(12)).ok()?;
        let mois = u32::try_from(total.rem_euclid(12)).ok()? + 1;
        if !(ANNEE_MIN..=ANNEE_MAX).contains(&annee) {
            return None;
        }
        let jour = ancrage.clamp(1, 31).min(jours_du_mois(annee, mois));
        Date::nouvelle(annee, mois, jour)
    }

    /// Le nombre de mois entiers du calendrier de `self` à `autre` (sans regarder les jours).
    pub fn mois_jusqua(&self, autre: Date) -> i64 {
        (i64::from(autre.annee) - i64::from(self.annee)) * 12 + i64::from(autre.mois) - i64::from(self.mois)
    }

    /// « 2026-11-17 », comme dans abonnements.json.
    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.annee, self.mois, self.jour)
    }

    /// Lit « 2026-11-17 » (exactement ce format) ; rien si la date n'existe pas.
    pub fn lire_iso(t: &str) -> Option<Date> {
        let o = t.as_bytes();
        if o.len() != 10 || o[4] != b'-' || o[7] != b'-' {
            return None;
        }
        let chiffres = |a: usize, b: usize| -> Option<u32> {
            let p = &t[a..b];
            if p.bytes().all(|c| c.is_ascii_digit()) { p.parse().ok() } else { None }
        };
        Date::nouvelle(i32::try_from(chiffres(0, 4)?).ok()?, chiffres(5, 7)?, chiffres(8, 10)?)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.iso())
    }
}

/// Un instant : une date et la minute du jour (0 à 1439), en heure locale. Rangés dans l'ordre du temps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    pub date: Date,
    minute: u32,
}

impl Instant {
    /// L'instant de cette date à cette heure et cette minute (une heure au-delà de 23 h 59 est ramenée à 23 h 59).
    pub fn nouveau(date: Date, heure: u32, minute: u32) -> Instant {
        Instant { date, minute: (heure * 60 + minute).min(24 * 60 - 1) }
    }

    /// La minute du jour (0 à 1439).
    pub fn minute_du_jour(&self) -> u32 {
        self.minute
    }

    /// Les minutes depuis le 1er janvier 1970 à 0 h (pour un écart entre deux instants).
    pub fn minutes_depuis_1970(&self) -> i64 {
        self.date.jours_depuis_1970() * 1440 + i64::from(self.minute)
    }

    /// « 2026-11-15T09:00 » (rappels.json).
    pub fn iso(&self) -> String {
        format!("{}T{:02}:{:02}", self.date.iso(), self.minute / 60, self.minute % 60)
    }

    /// Lit « 2026-11-15T09:00 » (exactement ce format) ; rien si l'instant n'existe pas.
    pub fn lire_iso(t: &str) -> Option<Instant> {
        let o = t.as_bytes();
        if o.len() != 16 || o[10] != b'T' || o[13] != b':' || !t.is_ascii() {
            return None;
        }
        let date = Date::lire_iso(&t[..10])?;
        let chiffres =
            |p: &str| -> Option<u32> { if p.bytes().all(|c| c.is_ascii_digit()) { p.parse().ok() } else { None } };
        let (h, m) = (chiffres(&t[11..13])?, chiffres(&t[14..16])?);
        (h < 24 && m < 60).then(|| Instant::nouveau(date, h, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::nouvelle(a, m, j).unwrap()
    }

    #[test]
    fn annees_bissextiles() {
        assert!(!bissextile(1900) && bissextile(2000) && !bissextile(2100) && bissextile(2024) && !bissextile(2026));
        assert!(bissextile(2028) && bissextile(2400) && !bissextile(2200));
        assert!(Date::nouvelle(2000, 2, 29).is_some());
        assert!(Date::nouvelle(1900, 2, 29).is_none());
        assert!(Date::nouvelle(2100, 2, 29).is_none());
        assert!(Date::nouvelle(2026, 4, 31).is_none());
        assert!(Date::nouvelle(2026, 13, 1).is_none() && Date::nouvelle(2026, 0, 1).is_none());
        assert!(Date::nouvelle(0, 1, 1).is_none() && Date::nouvelle(10000, 1, 1).is_none());
    }

    #[test]
    fn jours_civils_connus() {
        assert_eq!(d(1970, 1, 1).jours_depuis_1970(), 0);
        assert_eq!(d(1969, 12, 31).jours_depuis_1970(), -1);
        assert_eq!(d(2000, 3, 1).jours_depuis_1970(), 11_017);
        // (le 15 novembre 2026 : 20 772 jours, un dimanche)
        assert_eq!(d(2026, 11, 15).jours_depuis_1970(), 20_772);
        assert_eq!(d(2026, 11, 15).jour_semaine(), 0);
        assert_eq!(d(2026, 11, 17).jour_semaine(), 2); // mardi
        assert_eq!(d(1, 1, 1).jour_semaine(), 1); // lundi (calendrier grégorien prolongé)
        assert_eq!(Date::depuis_jours(d(9999, 12, 31).jours_depuis_1970() + 1), None);
        assert_eq!(Date::depuis_jours(d(1, 1, 1).jours_depuis_1970() - 1), None);
    }

    #[test]
    fn chaque_jour_de_1899_a_2101_suit_le_precedent() {
        // (un calendrier tenu jour par jour, sans la formule, comparé à elle)
        let (mut a, mut m, mut j) = (1899, 1, 1);
        let mut n = d(1899, 1, 1).jours_depuis_1970();
        while a < 2102 {
            let x = d(a, m, j);
            assert_eq!(x.jours_depuis_1970(), n, "{x}");
            assert_eq!(Date::depuis_jours(n), Some(x));
            j += 1;
            if j > jours_du_mois(a, m) {
                j = 1;
                m += 1;
                if m > 12 {
                    m = 1;
                    a += 1;
                }
            }
            n += 1;
        }
    }

    #[test]
    fn mois_avec_jour_voulu() {
        assert_eq!(d(2026, 1, 31).plus_mois(1, 31), Some(d(2026, 2, 28)));
        assert_eq!(d(2028, 1, 31).plus_mois(1, 31), Some(d(2028, 2, 29)));
        assert_eq!(d(2026, 4, 30).plus_mois(1, 31), Some(d(2026, 5, 31)));
        assert_eq!(d(2026, 12, 15).plus_mois(1, 15), Some(d(2027, 1, 15)));
        assert_eq!(d(2026, 1, 15).plus_mois(-1, 15), Some(d(2025, 12, 15)));
        assert_eq!(d(9999, 12, 1).plus_mois(1, 1), None);
        assert_eq!(d(2026, 1, 1).mois_jusqua(d(2027, 3, 1)), 14);
    }

    #[test]
    fn format_iso() {
        assert_eq!(d(2026, 3, 5).iso(), "2026-03-05");
        assert_eq!(Date::lire_iso("2026-03-05"), Some(d(2026, 3, 5)));
        for faux in ["2026-02-29", "2026-3-05", "2026/03/05", "", "abcd-ef-gh", "2026-03-05T00:00", "+026-03-05"] {
            assert_eq!(Date::lire_iso(faux), None, "{faux}");
        }
    }

    #[test]
    fn instants_dans_l_ordre() {
        let a = Instant::nouveau(d(2026, 11, 15), 9, 0);
        let b = Instant::nouveau(d(2026, 11, 15), 9, 1);
        let c = Instant::nouveau(d(2026, 11, 16), 0, 0);
        assert!(a < b && b < c);
        assert_eq!(Instant::nouveau(d(2026, 11, 15), 30, 0).minute_du_jour(), 1439);
    }

    #[test]
    fn instants_ecrits_et_relus() {
        let a = Instant::nouveau(d(2026, 11, 15), 9, 5);
        assert_eq!(a.iso(), "2026-11-15T09:05");
        assert_eq!(Instant::lire_iso("2026-11-15T09:05"), Some(a));
        assert_eq!(Instant::nouveau(d(2026, 11, 16), 0, 0).minutes_depuis_1970() - a.minutes_depuis_1970(), 895);
        for faux in [
            "",
            "2026-11-15",
            "2026-11-15T9:05",
            "2026-11-15T24:00",
            "2026-11-15T09:60",
            "2026-02-30T09:00",
            "2026-11-15 09:05",
            "2026-11-15T09:0é",
            "2026-11-15T+9:05",
        ] {
            assert_eq!(Instant::lire_iso(faux), None, "{faux}");
        }
    }
}
