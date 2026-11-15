//! Les données de Paymind réunies (repris de mindcare-donnees) : le dossier de données, le journal, les fichiers pris,
//! les réglages, les abonnements, les rappels et la carte de Mindset.

use std::path::{Path, PathBuf};

use crate::abonnements::{self, Abonnements};
use crate::carte::{EtatMindset, etat_carte};
use crate::fichiers::Fichiers;
use crate::journal::{Horloge, Journal};
use crate::moment::Moment;
use crate::rappels::{self, Bulle, Demande, Rappels, Verifie};
use crate::reglages::Reglages;

/// Les noms des fichiers du dossier de données.
pub const JOURNAL: &str = "journal.txt";
pub const REGLAGES: &str = "reglages.json";
pub const ETAT: &str = "etat.json";
pub const ABONNEMENTS: &str = abonnements::FICHIER;
pub const RAPPELS: &str = rappels::FICHIER;

/// Tout ce que Paymind garde dans son dossier de données.
#[derive(Clone)]
pub struct Donnees {
    dossier: PathBuf,
    pub fichiers: Fichiers,
    pub reglages: Reglages,
    pub abonnements: Abonnements,
    pub rappels: Rappels,
    pub etat: EtatMindset,
    /// La carte de Mindset est écrite (seulement pour une copie gérée par Mindset)
    pub carte_active: bool,
}

impl Donnees {
    /// Ouvre le dossier de données (créé s'il n'existe pas), raccourcit le journal, lit les réglages, les
    /// abonnements et les rappels.
    pub fn ouvrir(dossier: &Path, horloge: Horloge, carte_active: bool) -> Donnees {
        let _ = std::fs::create_dir_all(dossier);
        let journal = Journal::ouvrir(&dossier.join(JOURNAL), horloge);
        let mut fichiers = Fichiers::nouveau(journal);
        let reglages = Reglages::charger(&mut fichiers, &dossier.join(REGLAGES));
        let abonnements = Abonnements::charger(&mut fichiers, &dossier.join(ABONNEMENTS));
        let rappels = Rappels::charger(&mut fichiers, dossier);
        Donnees {
            dossier: dossier.to_path_buf(),
            fichiers,
            reglages,
            abonnements,
            rappels,
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

    /// La vérification des rappels (au démarrage, puis régulièrement) : une bulle mise en attente s'il y a des rappels
    /// dus depuis la dernière ; des dates avancées (renouvellement automatique) : la liste est enregistrée et la carte
    /// de Mindset réécrite. Une liste en lecture seule (version plus récente de Paymind) n'avance pas sur le disque :
    /// elle est avancée en mémoire seulement.
    pub fn verifier_rappels(&mut self, maintenant: Moment) -> Verifie {
        let v = self.rappels.verifier(&mut self.fichiers, &mut self.abonnements.liste, maintenant);
        if v.avances > 0 {
            self.fichiers
                .journal
                .ecrire(&format!("Rappels : {} date(s) avancée(s) (renouvellement automatique)", v.avances));
            if self.abonnements.lecture_seule().is_none() {
                let _ = self.abonnements.sauver(&mut self.fichiers);
            }
            self.ecrire_etat();
        }
        v
    }

    /// Envoie la bulle en attente à Mindset si c'est le moment ([`Rappels::envoyer`]).
    pub fn envoyer_bulle(&mut self, maintenant: Moment) -> Option<Bulle> {
        self.rappels.envoyer(&mut self.fichiers, maintenant)
    }

    /// Lit la demande d'affichage (un clic sur une bulle, un numéro de processus) ([`Rappels::demande`]).
    pub fn demande_affichage(&mut self) -> Demande {
        self.rappels.demande(&mut self.fichiers, &self.abonnements.liste)
    }
}
