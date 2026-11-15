//! Les rappels : une bulle avant un paiement, au moment choisi (le jour même, la veille, 3 ou 7 jours avant).
//!
//! Un rappel est dû à [`HEURE_RAPPEL`] le jour voulu (la veille d'un paiement du 1er décembre : le 30 novembre à 9 h).
//! L'appli garde l'instant de sa dernière vérification (étape 6) et demande les rappels dus depuis : un rappel tombé
//! pendant que le PC était éteint sort au démarrage suivant.
//!
//! - Sans doublon : les rappels dus sont ceux de l'intervalle (dernière vérification, maintenant], ouvert au début et
//!   fermé à la fin ; deux vérifications qui se suivent ne rendent jamais le même rappel. Pour un premier lancement,
//!   l'appelant prend « maintenant » comme dernière vérification (aucun rappel du passé).
//! - Au plus un rappel par abonnement à la fois : après des mois de PC éteint, seul le plus récent sort (pas une
//!   bulle par mois manqué).
//! - Aucun rappel pour un abonnement en pause ou réglé sur « jamais ».
//! - Les rappels se calculent sur les dates gardées, AVANT de les avancer : [`verifier`] fait les deux dans cet ordre.

use crate::abonnement::{Abonnement, Rappel};
use crate::date::{Date, Instant};
use crate::echeances::{avancer_tout, paiement_numero};

/// L'heure où un rappel devient dû, le jour voulu.
pub const HEURE_RAPPEL: u32 = 9;

/// Un rappel dû.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RappelDu {
    pub id: String,
    pub nom: String,
    /// En centimes.
    pub prix: u64,
    /// Le paiement dont il prévient.
    pub paiement: Date,
    /// L'instant où il est devenu dû.
    pub du: Instant,
    pub rappel: Rappel,
    /// Renouvellement automatique (sinon : il faudra le renouveler soi-même).
    pub auto: bool,
}

impl RappelDu {
    /// Une clé qui le désigne (l'abonnement et la date du paiement), si l'appelant veut se souvenir de ceux montrés.
    pub fn cle(&self) -> String {
        format!("{}@{}", self.id, self.paiement.iso())
    }
}

/// L'instant où le rappel d'un paiement devient dû ; rien pour « jamais ».
pub fn instant_du_rappel(rappel: Rappel, paiement: Date) -> Option<Instant> {
    let jours = rappel.jours_avant()?;
    Some(Instant::nouveau(paiement.plus_jours(-jours)?, HEURE_RAPPEL, 0))
}

/// Les rappels dus après `depuis` et jusqu'à `jusqua` (compris), par instant puis par nom ; au plus un par abonnement
/// (le plus récent). Rien si `jusqua` n'est pas après `depuis` (l'horloge a reculé).
pub fn rappels_dus(liste: &[Abonnement], depuis: Instant, jusqua: Instant) -> Vec<RappelDu> {
    let mut dus = Vec::new();
    if jusqua <= depuis {
        return dus;
    }
    for a in liste {
        if a.pause || a.rappel == Rappel::Jamais {
            continue;
        }
        let mut dernier = None;
        // (sans renouvellement automatique, un seul paiement : celui de la date gardée)
        let derniere_k = if a.auto { i64::MAX } else { 0 };
        let mut k = 0;
        while k <= derniere_k {
            let Some(p) = paiement_numero(a, k) else { break };
            let Some(du) = instant_du_rappel(a.rappel, p) else { break };
            if du > jusqua {
                break;
            }
            if du > depuis {
                dernier = Some((p, du));
            }
            k += 1;
        }
        if let Some((paiement, du)) = dernier {
            dus.push(RappelDu {
                id: a.id.clone(),
                nom: a.nom.clone(),
                prix: a.prix,
                paiement,
                du,
                rappel: a.rappel,
                auto: a.auto,
            });
        }
    }
    dus.sort_by(|x, y| (x.du, &x.nom, &x.id).cmp(&(y.du, &y.nom, &y.id)));
    dus
}

/// Ce qu'a donné une vérification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    /// Les rappels à montrer.
    pub rappels: Vec<RappelDu>,
    /// Le nombre d'abonnements dont la date a avancé (la liste est à enregistrer s'il n'est pas nul).
    pub avances: usize,
}

/// La vérification de l'appli (au démarrage, puis de temps en temps) : les rappels dus depuis la dernière vérification,
/// calculés sur les dates gardées, puis les dates passées avancées (renouvellement automatique).
pub fn verifier(liste: &mut [Abonnement], depuis: Instant, maintenant: Instant) -> Verification {
    let rappels = rappels_dus(liste, depuis, maintenant);
    let avances = avancer_tout(liste, maintenant.date);
    Verification { rappels, avances }
}
