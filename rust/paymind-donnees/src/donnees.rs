//! Les données de Paymind réunies (repris de mindcare-donnees) : le dossier de données, le journal, les fichiers pris,
//! les réglages, les abonnements et la carte de Mindset.

use std::path::{Path, PathBuf};

use crate::abonnements::{self, Abonnements};
use crate::carte::{EtatMindset, etat_carte};
use crate::fichiers::Fichiers;
use crate::journal::{Horloge, Journal};
use crate::reglages::Reglages;

/// Les noms des fichiers du dossier de données.
pub const JOURNAL: &str = "journal.txt";
pub const REGLAGES: &str = "reglages.json";
pub const ETAT: &str = "etat.json";
pub const ABONNEMENTS: &str = abonnements::FICHIER;

/// Tout ce que Paymind garde dans son dossier de données.
#[derive(Clone)]
pub struct Donnees {
    dossier: PathBuf,
    pub fichiers: Fichiers,
    pub reglages: Reglages,
    pub abonnements: Abonnements,
    pub etat: EtatMindset,
    /// La carte de Mindset est écrite (seulement pour une copie gérée par Mindset)
    pub carte_active: bool,
}

impl Donnees {
    /// Ouvre le dossier de données (créé s'il n'existe pas), raccourcit le journal, lit les réglages et les
    /// abonnements.
    pub fn ouvrir(dossier: &Path, horloge: Horloge, carte_active: bool) -> Donnees {
        let _ = std::fs::create_dir_all(dossier);
        let journal = Journal::ouvrir(&dossier.join(JOURNAL), horloge);
        let mut fichiers = Fichiers::nouveau(journal);
        let reglages = Reglages::charger(&mut fichiers, &dossier.join(REGLAGES));
        let abonnements = Abonnements::charger(&mut fichiers, &dossier.join(ABONNEMENTS));
        Donnees {
            dossier: dossier.to_path_buf(),
            fichiers,
            reglages,
            abonnements,
            etat: EtatMindset::nouveau(&dossier.join(ETAT)),
            carte_active,
        }
    }

    /// Le dossier de données.
    pub fn dossier(&self) -> &Path {
        &self.dossier
    }

    /// Le journal.
    pub fn journal(&self) -> &Journal {
        &self.fichiers.journal
    }

    /// `Write-EtatMindset` : la carte de Mindset (le total du mois, le prochain paiement, le nombre d'abonnements),
    /// écrite seulement si elle change.
    pub fn ecrire_etat(&mut self) {
        if !self.carte_active {
            return;
        }
        let aujourdhui = self.fichiers.journal.maintenant().date();
        let carte = etat_carte(&self.abonnements.liste, aujourdhui);
        self.etat.ecrire(&mut self.fichiers, &carte);
    }
}
