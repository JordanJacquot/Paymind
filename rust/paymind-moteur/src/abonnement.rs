//! Un abonnement : ce que la fiche de la maquette F montre (nom, prix, rythme, prochain paiement, renouvellement
//! automatique, rappel, pause, note), plus le jour voulu (l'ancrage) et la catégorie.
//!
//! Le prix est en centimes (un entier) : 13,49 € est 1349. Aucun nombre à virgule n'entre dans les comptes.

use crate::date::Date;

/// Le prix le plus haut accepté : 999 999,99 €.
pub const PRIX_MAX: u64 = 99_999_999;
/// La longueur la plus grande d'un nom (en caractères).
pub const NOM_MAX: usize = 80;
/// La longueur la plus grande d'une note (en caractères).
pub const NOTE_MAX: usize = 2000;

/// Le rythme des paiements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rythme {
    Semaine,
    Mois,
    Trimestre,
    An,
}

impl Rythme {
    pub const TOUS: [Rythme; 4] = [Rythme::Semaine, Rythme::Mois, Rythme::Trimestre, Rythme::An];

    /// Le nombre de paiements dans une année : 52 semaines, 12 mois, 4 trimestres, 1 an.
    pub fn par_an(self) -> u64 {
        match self {
            Rythme::Semaine => 52,
            Rythme::Mois => 12,
            Rythme::Trimestre => 4,
            Rythme::An => 1,
        }
    }

    /// Le nombre de mois entre deux paiements (rien pour la semaine, qui compte en jours).
    pub fn mois(self) -> Option<i64> {
        match self {
            Rythme::Semaine => None,
            Rythme::Mois => Some(1),
            Rythme::Trimestre => Some(3),
            Rythme::An => Some(12),
        }
    }

    /// Le nom gardé dans abonnements.json.
    pub fn code(self) -> &'static str {
        match self {
            Rythme::Semaine => "semaine",
            Rythme::Mois => "mois",
            Rythme::Trimestre => "trimestre",
            Rythme::An => "an",
        }
    }

    pub fn depuis_code(t: &str) -> Option<Rythme> {
        Rythme::TOUS.into_iter().find(|r| r.code() == t)
    }

    /// Le mot de l'écran : « par mois ».
    pub fn par_texte(self) -> &'static str {
        match self {
            Rythme::Semaine => "par semaine",
            Rythme::Mois => "par mois",
            Rythme::Trimestre => "par trimestre",
            Rythme::An => "par an",
        }
    }
}

/// Quand prévenir avant un paiement (la veille par défaut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rappel {
    Jamais,
    JourMeme,
    #[default]
    Veille,
    TroisJours,
    SeptJours,
}

impl Rappel {
    pub const TOUS: [Rappel; 5] =
        [Rappel::Jamais, Rappel::JourMeme, Rappel::Veille, Rappel::TroisJours, Rappel::SeptJours];

    /// Le nombre de jours avant le paiement ; rien pour « jamais ».
    pub fn jours_avant(self) -> Option<i64> {
        match self {
            Rappel::Jamais => None,
            Rappel::JourMeme => Some(0),
            Rappel::Veille => Some(1),
            Rappel::TroisJours => Some(3),
            Rappel::SeptJours => Some(7),
        }
    }

    /// Le nom gardé dans abonnements.json.
    pub fn code(self) -> &'static str {
        match self {
            Rappel::Jamais => "jamais",
            Rappel::JourMeme => "jour-meme",
            Rappel::Veille => "veille",
            Rappel::TroisJours => "3-jours",
            Rappel::SeptJours => "7-jours",
        }
    }

    pub fn depuis_code(t: &str) -> Option<Rappel> {
        Rappel::TOUS.into_iter().find(|r| r.code() == t)
    }

    /// Le texte de la fiche : « Jamais », « Le jour même », « La veille », « 3 jours avant », « 7 jours avant ».
    pub fn texte(self) -> &'static str {
        match self {
            Rappel::Jamais => "Jamais",
            Rappel::JourMeme => "Le jour même",
            Rappel::Veille => "La veille",
            Rappel::TroisJours => "3 jours avant",
            Rappel::SeptJours => "7 jours avant",
        }
    }
}

/// Où en est un abonnement, un jour donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Statut {
    /// Il compte dans les totaux.
    Actif,
    /// Mis en pause : il ne compte pas, aucun rappel.
    EnPause,
    /// Sans renouvellement automatique, sa date est passée : il ne compte plus, jusqu'à ce qu'on le renouvelle
    /// (ou qu'on le supprime).
    ARenouveler,
}

/// Un abonnement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abonnement {
    /// Son identifiant, unique dans la liste (« a1 », « a2 »...).
    pub id: String,
    pub nom: String,
    /// L'identifiant de la liste toute prête (« netflix »), rien pour un abonnement « à moi ».
    pub catalogue: Option<String>,
    /// La catégorie (« video », « jeux », « moi »...), celle de la liste toute prête.
    pub categorie: String,
    /// Le prix d'un paiement, en centimes.
    pub prix: u64,
    pub rythme: Rythme,
    /// La date du prochain paiement (ou du dernier, s'il est à renouveler).
    pub date: Date,
    /// Le jour voulu (1 à 31) : un paiement du 31 revient au 31 après un mois de 30 jours. Sans effet pour la semaine.
    /// Choisir une nouvelle date dans la fiche change aussi le jour voulu ([`Abonnement::changer_date`]).
    pub ancrage: u32,
    /// Renouvellement automatique : la date avance seule après chaque paiement.
    pub auto: bool,
    pub rappel: Rappel,
    pub pause: bool,
    pub note: String,
}

impl Abonnement {
    /// Un abonnement neuf : renouvellement automatique, rappel la veille, jour voulu celui de la date, catégorie « moi ».
    pub fn nouveau(id: &str, nom: &str, prix: u64, rythme: Rythme, date: Date) -> Abonnement {
        Abonnement {
            id: id.to_string(),
            nom: nom.to_string(),
            catalogue: None,
            categorie: "moi".to_string(),
            prix,
            rythme,
            date,
            ancrage: date.jour(),
            auto: true,
            rappel: Rappel::default(),
            pause: false,
            note: String::new(),
        }
    }

    /// Ce qui ne va pas dans cet abonnement (lu dans un fichier) ; rien s'il est bon.
    pub fn probleme(&self) -> Option<String> {
        if self.id.trim().is_empty() {
            return Some("sans identifiant".into());
        }
        if self.nom.trim().is_empty() {
            return Some("sans nom".into());
        }
        if self.nom.chars().count() > NOM_MAX {
            return Some("nom trop long".into());
        }
        if self.note.chars().count() > NOTE_MAX {
            return Some("note trop longue".into());
        }
        if self.prix > PRIX_MAX {
            return Some("prix trop grand".into());
        }
        if !(1..=31).contains(&self.ancrage) {
            return Some("jour voulu hors de 1 à 31".into());
        }
        None
    }

    /// Où il en est ce jour-là. La date du jour même compte encore (le paiement est aujourd'hui).
    pub fn statut(&self, aujourdhui: Date) -> Statut {
        if self.pause {
            Statut::EnPause
        } else if !self.auto && self.date < aujourdhui {
            Statut::ARenouveler
        } else {
            Statut::Actif
        }
    }

    /// Ce qu'il coûte par an, en centimes (exact : aucun arrondi).
    pub fn par_an(&self) -> u64 {
        self.prix * self.rythme.par_an()
    }

    /// La date du prochain paiement choisie dans la fiche : elle devient la date gardée, et son jour devient le jour
    /// voulu (l'ancrage). Choisir le 31 janvier fait revenir les paiements au 31 (le 28 février, puis le 31 mars) ;
    /// choisir ensuite le 28 février les fait revenir au 28. Un abonnement à renouveler dont la nouvelle date n'est
    /// pas passée redevient actif.
    pub fn changer_date(&mut self, date: Date) {
        self.date = date;
        self.ancrage = date.jour();
    }

    /// Un nouveau rythme : la date gardée ne change pas. Depuis la semaine (où le jour voulu ne sert pas), le jour
    /// voulu devient celui de la date gardée ; sinon il reste le même (un paiement du 31 reste au 31).
    pub fn changer_rythme(&mut self, rythme: Rythme) {
        if self.rythme == Rythme::Semaine && rythme != Rythme::Semaine {
            self.ancrage = self.date.jour();
        }
        self.rythme = rythme;
    }
}

/// Un identifiant libre pour un nouvel abonnement : « a » suivi du plus grand numéro déjà pris, plus un.
pub fn id_libre(liste: &[Abonnement]) -> String {
    let max = liste.iter().filter_map(|a| a.id.strip_prefix('a').and_then(|n| n.parse::<u64>().ok())).max();
    format!("a{}", max.map_or(1, |m| m + 1))
}

/// Le total par an d'une liste, en centimes, sans arrondi (les abonnements qui ne sont pas actifs ce jour-là ne
/// comptent pas).
pub fn total_par_an(liste: &[Abonnement], aujourdhui: Date) -> u64 {
    liste.iter().filter(|a| a.statut(aujourdhui) == Statut::Actif).map(Abonnement::par_an).sum()
}

/// Le total par mois à partir du total par an : divisé par 12 et arrondi au centime le plus proche seulement à la fin
/// (un demi-centime est arrondi vers le haut).
pub fn par_mois(par_an: u64) -> u64 {
    (par_an + 6) / 12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_aller_retour() {
        for r in Rythme::TOUS {
            assert_eq!(Rythme::depuis_code(r.code()), Some(r));
        }
        for r in Rappel::TOUS {
            assert_eq!(Rappel::depuis_code(r.code()), Some(r));
        }
        assert_eq!(Rythme::depuis_code("Mois"), None);
        assert_eq!(Rappel::default(), Rappel::Veille);
    }

    #[test]
    fn identifiants_libres() {
        let d = Date::nouvelle(2026, 11, 15).unwrap();
        assert_eq!(id_libre(&[]), "a1");
        let l = vec![
            Abonnement::nouveau("a3", "x", 1, Rythme::Mois, d),
            Abonnement::nouveau("b9", "y", 1, Rythme::Mois, d),
        ];
        assert_eq!(id_libre(&l), "a4");
    }

    #[test]
    fn changer_la_date_change_le_jour_voulu() {
        let d = |a, m, j| Date::nouvelle(a, m, j).unwrap();
        let mut a = Abonnement::nouveau("a1", "x", 100, Rythme::Mois, d(2026, 11, 30));
        a.ancrage = 31;
        a.changer_date(d(2027, 1, 31));
        assert_eq!((a.date, a.ancrage), (d(2027, 1, 31), 31));
        assert_eq!(crate::echeances::paiement_numero(&a, 1), Some(d(2027, 2, 28)));
        assert_eq!(crate::echeances::paiement_numero(&a, 2), Some(d(2027, 3, 31)));
        a.changer_date(d(2027, 2, 28));
        assert_eq!(a.ancrage, 28);
        assert_eq!(crate::echeances::paiement_numero(&a, 1), Some(d(2027, 3, 28)));
        // Un nouveau rythme garde le jour voulu ; depuis la semaine, il prend celui de la date
        a.ancrage = 31;
        a.changer_rythme(Rythme::Trimestre);
        assert_eq!((a.rythme, a.ancrage), (Rythme::Trimestre, 31));
        a.rythme = Rythme::Semaine;
        a.changer_rythme(Rythme::Mois);
        assert_eq!((a.rythme, a.ancrage), (Rythme::Mois, 28));
    }

    #[test]
    fn arrondi_seulement_a_la_fin() {
        assert_eq!(par_mois(35_964), 2997);
        assert_eq!(par_mois(6), 1); // 0,5 centime : vers le haut
        assert_eq!(par_mois(5), 0);
        assert_eq!(par_mois(270_400), 22_533);
    }
}
