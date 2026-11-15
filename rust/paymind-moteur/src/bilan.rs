//! Le bilan d'un jour : ce que la personne paie par mois et par an, par catégorie, combien d'abonnements comptent,
//! ce qui arrive bientôt et le prochain paiement (pour la carte de Mindset : « Netflix dans 3 jours »).
//!
//! Il se calcule sur la liste telle qu'elle est gardée : les dates passées des abonnements à renouvellement
//! automatique sont avancées ici, sans changer la liste (la liste elle-même avance avec [`crate::verifier`]).
//!
//! Les totaux sont exacts en centimes par an (prix × 52, 12, 4 ou 1) ; le total par mois est le total par an divisé
//! par 12, arrondi une seule fois à la fin (jamais abonnement par abonnement : le total ne dérive pas). Seuls les
//! abonnements actifs comptent (ni en pause, ni à renouveler).

use std::cmp::Reverse;

use crate::abonnement::{Abonnement, Statut, par_mois};
use crate::date::Date;
use crate::echeances::{paiements_entre, prochaine_echeance};

/// « Bientôt » : les paiements des 30 prochains jours, du jour 0 (aujourd'hui) au jour 30 compris, comme la carte de
/// Mindset qui dit « Netflix dans 30 jours » : un abonnement par mois ajouté aujourd'hui y paraît. Un abonnement payé
/// aujourd'hui n'y reparaît pas au jour 30 (le paiement du mois suivant : jamais deux fois le même abonnement à un
/// mois d'écart dans la liste).
pub const BIENTOT_JOURS: i64 = 30;

/// Le total d'une catégorie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalCategorie {
    pub categorie: String,
    /// En centimes, exact.
    pub par_an: u64,
    /// En centimes, arrondi à la fin (pour l'affichage de la catégorie seulement).
    pub par_mois: u64,
    /// Les abonnements actifs de la catégorie.
    pub nombre: usize,
}

/// Un paiement à venir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paiement {
    pub id: String,
    pub nom: String,
    pub date: Date,
    /// En centimes.
    pub prix: u64,
    /// Dans combien de jours (0 : aujourd'hui).
    pub dans_jours: i64,
}

/// Le bilan d'un jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bilan {
    pub aujourdhui: Date,
    /// En centimes, exact.
    pub par_an: u64,
    /// En centimes : le total par an divisé par 12, arrondi à la fin.
    pub par_mois: u64,
    /// Les catégories qui ont au moins un abonnement actif, de la plus chère à la moins chère (à égalité, dans l'ordre
    /// des noms).
    pub categories: Vec<TotalCategorie>,
    /// Tous les abonnements de la liste.
    pub total: usize,
    pub actifs: usize,
    pub en_pause: usize,
    pub a_renouveler: usize,
    /// Les paiements des 30 prochains jours (du jour 0 au jour 30, voir [`BIENTOT_JOURS`]), par date (puis par nom). Un abonnement par semaine y
    /// paraît à chacun de ses paiements.
    pub bientot: Vec<Paiement>,
    /// Le prochain paiement, même au-delà de 30 jours (rien si aucun abonnement actif).
    pub prochain: Option<Paiement>,
}

fn paiement(a: &Abonnement, date: Date, aujourdhui: Date) -> Paiement {
    Paiement { id: a.id.clone(), nom: a.nom.clone(), date, prix: a.prix, dans_jours: aujourdhui.jours_jusqua(date) }
}

/// Le bilan de la liste ce jour-là.
pub fn bilan(liste: &[Abonnement], aujourdhui: Date) -> Bilan {
    let mut b = Bilan {
        aujourdhui,
        par_an: 0,
        par_mois: 0,
        categories: Vec::new(),
        total: liste.len(),
        actifs: 0,
        en_pause: 0,
        a_renouveler: 0,
        bientot: Vec::new(),
        prochain: None,
    };
    let fin = aujourdhui.plus_jours(BIENTOT_JOURS).unwrap_or(aujourdhui);
    for a in liste {
        match a.statut(aujourdhui) {
            Statut::EnPause => {
                b.en_pause += 1;
                continue;
            }
            Statut::ARenouveler => {
                b.a_renouveler += 1;
                continue;
            }
            Statut::Actif => b.actifs += 1,
        }
        let an = a.par_an();
        b.par_an += an;
        match b.categories.iter_mut().find(|c| c.categorie == a.categorie) {
            Some(c) => {
                c.par_an += an;
                c.nombre += 1;
            }
            None => {
                b.categories.push(TotalCategorie { categorie: a.categorie.clone(), par_an: an, par_mois: 0, nombre: 1 })
            }
        }
        let mut dates = paiements_entre(a, aujourdhui, fin);
        // (payé aujourd'hui et de nouveau au jour 30 : seulement aujourd'hui)
        if dates.len() > 1 && dates.first() == Some(&aujourdhui) && dates.last() == Some(&fin) {
            dates.pop();
        }
        for d in dates {
            b.bientot.push(paiement(a, d, aujourdhui));
        }
        if let Some(d) = prochaine_echeance(a, aujourdhui) {
            let p = paiement(a, d, aujourdhui);
            if b.prochain.as_ref().is_none_or(|q| (p.date, &p.nom, &p.id) < (q.date, &q.nom, &q.id)) {
                b.prochain = Some(p);
            }
        }
    }
    b.par_mois = par_mois(b.par_an);
    for c in &mut b.categories {
        c.par_mois = par_mois(c.par_an);
    }
    b.categories.sort_by(|x, y| (Reverse(x.par_an), &x.categorie).cmp(&(Reverse(y.par_an), &y.categorie)));
    b.bientot.sort_by(|x, y| (x.date, &x.nom, &x.id).cmp(&(y.date, &y.nom, &y.id)));
    b
}
