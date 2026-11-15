//! Le calcul des comptes de Paymind (étape 2 du plan), sans écran, sans Windows et sans fichier :
//!
//! - [`date`] : une date du calendrier (jours civils, calculés à la main), et un instant (une date et une minute) ;
//! - [`abonnement`] : un abonnement (prix en centimes, rythme, prochain paiement, jour voulu, rappel, pause) ;
//! - [`echeances`] : les dates qui avancent après un renouvellement automatique, le 31 et le 29 février ;
//! - [`bilan`] : le total par mois et par an, par catégorie, « Bientôt », le prochain paiement ;
//! - [`rappels`] : les rappels dus entre deux instants (ceux manqués quand le PC était éteint compris) ;
//! - [`textes`] : les montants et les dates écrits pour l'écran, en mots simples.
//!
//! L'heure qu'il est est toujours donnée par l'appelant : ce crate ne lit jamais l'horloge.

pub mod abonnement;
pub mod bilan;
pub mod date;
pub mod echeances;
pub mod rappels;
pub mod textes;

pub use abonnement::{Abonnement, PRIX_MAX, Rappel, Rythme, Statut, id_libre, par_mois, total_par_an};
pub use bilan::{BIENTOT_JOURS, Bilan, Paiement, TotalCategorie, bilan};
pub use date::{Date, Instant};
pub use echeances::{avancer, avancer_tout, paiements_entre, prochaine_echeance, renouveler};
pub use rappels::{HEURE_RAPPEL, RappelDu, Verification, instant_du_rappel, rappels_dus, verifier};
