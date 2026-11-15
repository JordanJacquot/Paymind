//! Les rappels montrés près de l'horloge (étape 6 du plan) : rappels.json, les bulles demandées à Mindset
//! (notification.demande) et le clic sur une bulle (afficher.demande), contrat-appli partie 5.
//!
//! ```json
//! { "format": 1, "verifie": "2026-11-15T09:00",
//!   "attente": [ { "id": "0a1b2c3d4e5f", "titre": "...", "texte": "...", "cible": "fiche:a1" } ],
//!   "envoyees": [ { "id": "...", "titre": "...", "texte": "...", "cible": "portefeuille" } ] }
//! ```
//!
//! - `verifie` : l'instant de la dernière vérification. Les rappels dus sont ceux d'après lui jusqu'à maintenant
//!   (paymind-moteur, `rappels_dus`) : un rappel tombé pendant que le PC était éteint sort au démarrage suivant.
//!   Fichier absent ou illisible (premier lancement) : maintenant, aucune bulle pour le passé.
//! - Jamais deux fois la même bulle, même après un redémarrage ou un plantage : une vérification qui trouve des rappels
//!   n'est retenue que si rappels.json est enregistré (avec la bulle en attente et le nouvel instant) ; une bulle sort
//!   de l'attente et est enregistrée comme envoyée AVANT que notification.demande soit écrit. Au pire (un plantage
//!   entre les deux, quelques millisecondes), une bulle est perdue, jamais montrée deux fois.
//! - Plusieurs rappels dus en même temps : une seule bulle qui les regroupe (« 3 rappels de paiement »), qui ouvre le
//!   portefeuille ; un seul : sa fiche.
//! - Les règles d'écriture de Nevermind (contrat, partie 5) : une bulle à la fois (la précédente pas encore lue par
//!   Mindset : on attend), 6 s d'écart au moins, écrite à côté puis renommée, abandonnée si Mindset ne l'a pas lue en
//!   5 minutes (il n'est pas ouvert).
//! - afficher.demande : « notif:<id> » n'est accepté que pour une bulle que Paymind a donnée elle-même (les 30
//!   dernières, gardées dans rappels.json : un clic après une relance marche encore) ; un numéro de processus demande
//!   d'afficher la fenêtre ; tout le reste est ignoré. Le fichier est toujours supprimé, rien n'est exécuté.
//!
//! L'heure est toujours donnée par l'appelant (les tests rejouent des jours en un instant).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};

use paymind_moteur::textes::bulle_rappels;
use paymind_moteur::{Abonnement, Instant, avancer_tout, rappels_dus};
use serde_json::{Value, json};

use crate::fichiers::Fichiers;
use crate::moment::Moment;
use crate::valeurs::{Objet, champ};

/// Le fichier des rappels, dans le dossier de données.
pub const FICHIER: &str = "rappels.json";
/// La bulle demandée à Mindset (contrat-appli, partie 5).
pub const NOTIFICATION: &str = "notification.demande";
/// La demande d'affichage (le champ `afficher_fichier` de l'entrée de Paymind) : un numéro de processus, ou
/// « notif:<id> » après un clic sur une bulle.
pub const AFFICHER: &str = "afficher.demande";
/// Le numéro de format écrit par cette version.
pub const FORMAT: u64 = 1;
/// L'écart entre deux bulles, en secondes.
pub const ECART: i64 = 6;
/// Une bulle que Mindset n'a pas lue en 5 minutes est abandonnée.
pub const ABANDON: i64 = 5 * 60;
/// Les bulles envoyées dont on se souvient (pour le clic).
pub const GARDEES: usize = 30;
/// rappels.json est réécrit au moins tous les quarts d'heure, même sans bulle (l'instant de la vérification).
pub const NOTER_TOUTES: i64 = 15;
/// Une dernière vérification plus de 2 jours dans le futur : l'horloge était fausse, on repart de maintenant.
pub const FUTUR_MAX_JOURS: i64 = 2;
/// Une demande d'affichage plus grosse est supprimée sans être lue.
const AFFICHER_MAX: u64 = 64;

/// Ce qu'une bulle ouvre quand on clique dessus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cible {
    /// La fiche de cet abonnement
    Fiche(String),
    /// Le portefeuille (plusieurs rappels)
    Portefeuille,
}

impl Cible {
    fn code(&self) -> String {
        match self {
            Cible::Fiche(id) => format!("fiche:{id}"),
            Cible::Portefeuille => "portefeuille".into(),
        }
    }

    fn depuis_code(t: &str) -> Option<Cible> {
        if t == "portefeuille" {
            return Some(Cible::Portefeuille);
        }
        let id = t.strip_prefix("fiche:")?;
        id_abonnement_valable(id).then(|| Cible::Fiche(id.to_string()))
    }
}

/// Un identifiant d'abonnement acceptable dans une cible : 1 à 40 lettres ou chiffres ASCII, `-` ou `_`.
fn id_abonnement_valable(id: &str) -> bool {
    (1..=40).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Un numéro de bulle : 12 chiffres hexadécimaux en minuscules (ce que Mindset accepte).
pub fn id_bulle_valable(id: &str) -> bool {
    id.len() == 12 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Un numéro de bulle neuf, au hasard (48 bits : la graine au hasard de la bibliothèque standard).
fn nouvel_id(graine: i64) -> String {
    let mut h = RandomState::new().build_hasher();
    h.write_i64(graine);
    h.write_u32(std::process::id());
    format!("{:012x}", h.finish() & 0xffff_ffff_ffff)
}

/// Une bulle près de l'horloge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bulle {
    pub id: String,
    pub titre: String,
    pub texte: String,
    pub cible: Cible,
}

impl Bulle {
    fn json(&self) -> Value {
        json!({ "id": self.id, "titre": self.titre, "texte": self.texte, "cible": self.cible.code() })
    }

    fn depuis(v: &Value) -> Option<Bulle> {
        let o = v.as_object()?;
        let t = |n: &str| champ(o, n).and_then(Value::as_str).map(str::to_string);
        let b =
            Bulle { id: t("id")?, titre: t("titre")?, texte: t("texte")?, cible: Cible::depuis_code(&t("cible")?)? };
        id_bulle_valable(&b.id).then_some(b)
    }

    /// Ce que Mindset lit dans notification.demande : `{ titre, texte, id }`.
    pub fn notification(&self) -> String {
        json!({ "titre": self.titre, "texte": self.texte, "id": self.id }).to_string()
    }
}

/// Ce qu'a donné une vérification.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verifie {
    /// La bulle mise en attente (une seule, même pour plusieurs rappels)
    pub bulle: Option<Bulle>,
    /// Le nombre d'abonnements dont la date a avancé (renouvellement automatique) : la liste est à enregistrer
    pub avances: usize,
}

/// Ce que demandait afficher.demande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demande {
    /// Rien à faire (pas de fichier, une bulle inconnue, un contenu illisible)
    Rien,
    /// Un numéro de processus : montrer la fenêtre (dans Mindset si elle est gérée)
    Montrer,
    /// Un clic sur une de nos bulles : ouvrir ce qu'elle annonçait
    Ouvrir(Cible),
}

/// Les rappels : rappels.json, l'attente des bulles, les bulles envoyées.
#[derive(Debug, Clone)]
pub struct Rappels {
    dossier: PathBuf,
    /// La dernière vérification ; rien : premier lancement (aucune bulle pour le passé)
    verifie: Option<Instant>,
    /// L'instant gardé dans rappels.json la dernière fois qu'il a été enregistré
    note: Option<Instant>,
    attente: Vec<Bulle>,
    envoyees: Vec<Bulle>,
    /// Le moment (en secondes) où la dernière bulle a été écrite
    envoi: Option<i64>,
}

impl Rappels {
    /// Lit rappels.json dans ce dossier de données (absent ou illisible : premier lancement).
    pub fn charger(fichiers: &mut Fichiers, dossier: &Path) -> Rappels {
        let mut r = Rappels {
            dossier: dossier.to_path_buf(),
            verifie: None,
            note: None,
            attente: Vec::new(),
            envoyees: Vec::new(),
            envoi: None,
        };
        let Some(o) = fichiers.lire(&dossier.join(FICHIER)) else { return r };
        let liste = |o: &Objet, n: &str| -> Vec<Bulle> {
            champ(o, n)
                .and_then(Value::as_array)
                .map(|l| l.iter().filter_map(Bulle::depuis).collect())
                .unwrap_or_default()
        };
        r.verifie = champ(&o, "verifie").and_then(Value::as_str).and_then(Instant::lire_iso);
        if r.verifie.is_none() {
            fichiers.journal.ecrire("rappels.json sans dernière vérification lisible : repris de maintenant");
        }
        r.note = r.verifie;
        r.attente = liste(&o, "attente");
        r.envoyees = liste(&o, "envoyees");
        r
    }

    /// L'instant de la dernière vérification (rien avant la première).
    pub fn verifie(&self) -> Option<Instant> {
        self.verifie
    }

    /// Les bulles qui attendent d'être montrées.
    pub fn attente(&self) -> &[Bulle] {
        &self.attente
    }

    /// Les dernières bulles envoyées (les plus récentes à la fin).
    pub fn envoyees(&self) -> &[Bulle] {
        &self.envoyees
    }

    fn fichier(&self) -> PathBuf {
        self.dossier.join(FICHIER)
    }

    fn sauver(&mut self, fichiers: &mut Fichiers) -> Result<(), String> {
        let v = json!({
            "format": FORMAT,
            "verifie": self.verifie.map(|i| i.iso()),
            "attente": self.attente.iter().map(Bulle::json).collect::<Vec<_>>(),
            "envoyees": self.envoyees.iter().map(Bulle::json).collect::<Vec<_>>(),
        });
        let r = fichiers.sauver(&self.fichier(), &v);
        match &r {
            Ok(()) => self.note = self.verifie,
            Err(e) => fichiers.journal.ecrire(&format!("Rappels non enregistrés : {e}")),
        }
        r
    }

    /// La vérification (au démarrage, puis régulièrement) : les rappels dus depuis la dernière, en une seule bulle mise
    /// en attente, calculés sur les dates gardées ; puis les dates passées avancées (renouvellement automatique). La
    /// liste n'est pas enregistrée ici (l'appelant le fait si `avances` n'est pas nul).
    ///
    /// Des rappels trouvés ne sont retenus que si rappels.json a pu être enregistré ; sinon rien ne change (ni la
    /// liste, ni l'instant) et la vérification suivante les retrouve.
    pub fn verifier(&mut self, fichiers: &mut Fichiers, liste: &mut [Abonnement], maintenant: Moment) -> Verifie {
        let jusqua = maintenant.instant();
        let Some(depuis) = self.verifie else {
            // Premier lancement : rien du passé
            self.verifie = Some(jusqua);
            fichiers.journal.ecrire("Rappels : premier lancement, aucune bulle pour le passé");
            let _ = self.sauver(fichiers);
            return Verifie { bulle: None, avances: avancer_tout(liste, jusqua.date) };
        };
        if jusqua <= depuis {
            // L'horloge a reculé (ou la même minute) : rien de dû. Très loin dans le futur : l'horloge était fausse.
            if depuis.date.jours_jusqua(jusqua.date) < -FUTUR_MAX_JOURS {
                fichiers.journal.ecrire(&format!(
                    "Rappels : dernière vérification dans le futur ({}), reprise de maintenant",
                    depuis.iso()
                ));
                self.verifie = Some(jusqua);
                let _ = self.sauver(fichiers);
            }
            return Verifie { bulle: None, avances: avancer_tout(liste, jusqua.date) };
        }
        let dus = rappels_dus(liste, depuis, jusqua);
        let bulle = bulle_rappels(&dus, jusqua.date).map(|(titre, texte)| Bulle {
            id: nouvel_id(maintenant.secondes()),
            titre,
            texte,
            cible: match dus.as_slice() {
                [r] => Cible::Fiche(r.id.clone()),
                _ => Cible::Portefeuille,
            },
        });
        self.verifie = Some(jusqua);
        if let Some(b) = &bulle {
            self.attente.push(b.clone());
        }
        let ancien = self.note.map_or(i64::MAX, |n| jusqua.minutes_depuis_1970() - n.minutes_depuis_1970());
        if (bulle.is_some() || !(0..NOTER_TOUTES).contains(&ancien))
            && self.sauver(fichiers).is_err()
            && bulle.is_some()
        {
            // (pas enregistré : rien n'est retenu, la vérification suivante les retrouve)
            self.attente.pop();
            self.verifie = Some(depuis);
            return Verifie::default();
        }
        if let Some(b) = &bulle {
            let noms: Vec<&str> = dus.iter().map(|r| r.id.as_str()).collect();
            fichiers.journal.ecrire(&format!("Rappels : bulle {} en attente ({})", b.id, noms.join(", ")));
        }
        Verifie { bulle, avances: avancer_tout(liste, jusqua.date) }
    }

    /// Envoie la première bulle en attente à Mindset, si c'est le moment : la précédente a été lue, 6 s ont passé.
    /// Une bulle que Mindset n'a pas lue en 5 minutes est abandonnée. Rend la bulle envoyée.
    pub fn envoyer(&mut self, fichiers: &mut Fichiers, maintenant: Moment) -> Option<Bulle> {
        let s = maintenant.secondes();
        let notif = self.dossier.join(NOTIFICATION);
        if notif.exists() {
            match self.envoi {
                // (laissée par une copie d'avant : comptée de maintenant)
                None => self.envoi = Some(s),
                Some(e) if s - e > ABANDON || s < e => {
                    if std::fs::remove_file(&notif).is_ok() {
                        fichiers.journal.ecrire("Bulle abandonnée (Mindset ne l'a pas lue en 5 minutes)");
                    }
                    self.envoi = Some(s);
                }
                Some(_) => {}
            }
            return None;
        }
        if self.attente.is_empty() || self.envoi.is_some_and(|e| (0..ECART).contains(&(s - e))) {
            return None;
        }
        // Notée envoyée AVANT d'être écrite : jamais deux fois, même si l'appli plante juste après
        let b = self.attente.remove(0);
        self.envoyees.push(b.clone());
        if self.envoyees.len() > GARDEES {
            self.envoyees.remove(0);
        }
        if self.sauver(fichiers).is_err() {
            self.envoyees.retain(|x| x.id != b.id);
            self.attente.insert(0, b);
            return None;
        }
        self.envoi = Some(s);
        match ecrire_a_cote(&notif, &b.notification()) {
            Ok(()) => {
                fichiers.journal.ecrire(&format!("Bulle envoyée ({}) : {} | {}", b.id, b.titre, b.texte));
                Some(b)
            }
            Err(e) => {
                // (pas écrite : remise en attente, réessayée dans 6 s)
                fichiers.journal.ecrire(&format!("Bulle pas envoyée : {e}"));
                self.envoyees.retain(|x| x.id != b.id);
                self.attente.insert(0, b);
                let _ = self.sauver(fichiers);
                None
            }
        }
    }

    /// Lit afficher.demande (toujours supprimé ensuite, rien n'est exécuté) : un clic sur une de nos bulles (la fiche
    /// d'un abonnement qui n'existe plus devient le portefeuille), un numéro de processus, ou rien.
    pub fn demande(&mut self, fichiers: &mut Fichiers, liste: &[Abonnement]) -> Demande {
        let f = self.dossier.join(AFFICHER);
        let Ok(m) = std::fs::metadata(&f) else { return Demande::Rien };
        if !m.is_file() {
            return Demande::Rien;
        }
        let octets = if m.len() < AFFICHER_MAX { std::fs::read(&f).ok() } else { None };
        if let Err(e) = std::fs::remove_file(&f) {
            fichiers.journal.ecrire(&format!("Demande d'affichage pas supprimée : {e}"));
        }
        let Some(octets) = octets else {
            fichiers.journal.ecrire("Demande d'affichage trop grosse ou illisible, ignorée");
            return Demande::Rien;
        };
        let texte = String::from_utf8_lossy(&octets);
        let t = texte.trim_start_matches('\u{feff}').trim();
        if let Some(id) = t.strip_prefix("notif:")
            && id_bulle_valable(id)
        {
            let Some(b) = self.envoyees.iter().rev().find(|b| b.id == id) else {
                fichiers.journal.ecrire(&format!("Bulle inconnue ignorée ({id})"));
                return Demande::Rien;
            };
            fichiers.journal.ecrire(&format!("Bulle ouverte : {id}"));
            return Demande::Ouvrir(match &b.cible {
                Cible::Fiche(a) if liste.iter().any(|x| &x.id == a) => b.cible.clone(),
                _ => Cible::Portefeuille,
            });
        }
        if (1..=10).contains(&t.len()) && t.bytes().all(|c| c.is_ascii_digit()) {
            fichiers.journal.ecrire(&format!("Demande d'affichage ({t})"));
            return Demande::Montrer;
        }
        fichiers.journal.ecrire("Demande d'affichage illisible, ignorée");
        Demande::Rien
    }
}

/// Écrit un fichier à côté (« .ecriture ») puis le renomme : Mindset ne lit jamais un fichier à moitié écrit. UTF-8
/// sans BOM.
fn ecrire_a_cote(fichier: &Path, texte: &str) -> std::io::Result<()> {
    let mut a_cote = fichier.as_os_str().to_owned();
    a_cote.push(".ecriture");
    let a_cote = PathBuf::from(a_cote);
    std::fs::write(&a_cote, texte.as_bytes())?;
    std::fs::rename(&a_cote, fichier).inspect_err(|_| {
        let _ = std::fs::remove_file(&a_cote);
    })
}
