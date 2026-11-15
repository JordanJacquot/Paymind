//! La liste des abonnements, gardée sur le PC dans abonnements.json (dans le dossier de données de l'appli :
//! `%LOCALAPPDATA%\Mindset\Paymind` en vrai, le dossier d'essai dans les tests). Le calcul est dans paymind-moteur ;
//! ici, seulement le fichier.
//!
//! ```json
//! { "format": 1, "abonnements": [ { "id": "a1", "nom": "Netflix", "catalogue": "netflix", "categorie": "video",
//!   "prix": 1349, "rythme": "mois", "date": "2026-11-18", "ancrage": 18, "auto": true, "rappel": "veille",
//!   "pause": false, "note": "" } ] }
//! ```
//!
//! Le prix est en centimes. `rythme` : semaine, mois, trimestre, an. `rappel` : jamais, jour-meme, veille, 3-jours,
//! 7-jours. `catalogue` : rien (null) pour un abonnement « à moi ».
//!
//! Rien ne se perd :
//! - écriture : à côté puis remplacé d'un coup, l'ancienne version gardée dans « abonnements.json.bak » ([`Fichiers`],
//!   `json_sur`) ;
//! - un fichier abîmé (vide, coupé, pas du JSON) est mis de côté (« .abime-<date> », jamais effacé) et la copie
//!   « .bak » est reprise ; de même pour un JSON qui n'est pas une liste d'abonnements (sans « format » ni
//!   « abonnements ») ;
//! - un abonnement illisible dans un bon fichier (date qui n'existe pas, prix négatif...) est laissé de côté, et tout
//!   le fichier lu est d'abord recopié à côté (« .incomplet-<date> ») : l'enregistrement suivant ne le fait pas
//!   disparaître pour de bon ;
//! - un fichier d'une version plus récente de Paymind (« format » plus grand) est lu comme il peut, mais n'est jamais
//!   réécrit (lecture seule) : revenir à une ancienne Paymind ne casse pas la liste de la nouvelle ;
//! - un fichier absent donne une liste vide.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use paymind_moteur::{Abonnement, Date, Rappel, Rythme};
use serde_json::{Map, Value, json};

use crate::fichiers::Fichiers;
use crate::valeurs::Objet;

/// Le nom du fichier, dans le dossier de données.
pub const FICHIER: &str = "abonnements.json";
/// Le numéro de format écrit par cette version.
pub const FORMAT: u64 = 1;
/// Pourquoi la liste est en lecture seule quand elle vient d'une version plus récente de Paymind.
pub const PLUS_RECENTE: &str = "la liste vient d'une version plus récente de Paymind";

/// Un abonnement en JSON.
pub fn abonnement_json(a: &Abonnement) -> Value {
    let mut o = Map::new();
    o.insert("id".into(), json!(a.id));
    o.insert("nom".into(), json!(a.nom));
    o.insert("catalogue".into(), a.catalogue.as_ref().map_or(Value::Null, |c| json!(c)));
    o.insert("categorie".into(), json!(a.categorie));
    o.insert("prix".into(), json!(a.prix));
    o.insert("rythme".into(), json!(a.rythme.code()));
    o.insert("date".into(), json!(a.date.iso()));
    o.insert("ancrage".into(), json!(a.ancrage));
    o.insert("auto".into(), json!(a.auto));
    o.insert("rappel".into(), json!(a.rappel.code()));
    o.insert("pause".into(), json!(a.pause));
    o.insert("note".into(), json!(a.note));
    Value::Object(o)
}

/// Tout le fichier en JSON.
pub fn liste_json(liste: &[Abonnement]) -> Value {
    let mut o = Map::new();
    o.insert("format".into(), json!(FORMAT));
    o.insert("abonnements".into(), Value::Array(liste.iter().map(abonnement_json).collect()));
    Value::Object(o)
}

fn texte<'a>(o: &'a Objet, nom: &str) -> Result<Option<&'a str>, String> {
    match o.get(nom) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("« {nom} » n'est pas un texte")),
    }
}

fn oui_non(o: &Objet, nom: &str, defaut: bool) -> Result<bool, String> {
    match o.get(nom) {
        None | Some(Value::Null) => Ok(defaut),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("« {nom} » n'est ni vrai ni faux")),
    }
}

/// Un abonnement lu dans le fichier, ou pourquoi il ne peut pas l'être.
pub fn abonnement_depuis(v: &Value) -> Result<Abonnement, String> {
    let Value::Object(o) = v else { return Err("ce n'est pas un objet".into()) };
    let id = texte(o, "id")?.ok_or("sans identifiant")?;
    let nom = texte(o, "nom")?.ok_or("sans nom")?;
    let prix = o.get("prix").and_then(Value::as_u64).ok_or("prix absent, négatif ou à virgule")?;
    let rythme = texte(o, "rythme")?.and_then(Rythme::depuis_code).ok_or("rythme inconnu")?;
    let date = texte(o, "date")?.and_then(Date::lire_iso).ok_or("date absente ou qui n'existe pas")?;
    let ancrage = match o.get("ancrage") {
        None | Some(Value::Null) => date.jour(),
        Some(x) => x.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or("jour voulu illisible")?,
    };
    let rappel = match texte(o, "rappel")? {
        None => Rappel::default(),
        Some(t) => Rappel::depuis_code(t).ok_or("rappel inconnu")?,
    };
    let a = Abonnement {
        id: id.to_string(),
        nom: nom.to_string(),
        catalogue: texte(o, "catalogue")?.filter(|c| !c.is_empty()).map(str::to_string),
        categorie: texte(o, "categorie")?.filter(|c| !c.is_empty()).unwrap_or("moi").to_string(),
        prix,
        rythme,
        date,
        ancrage,
        auto: oui_non(o, "auto", true)?,
        rappel,
        pause: oui_non(o, "pause", false)?,
        note: texte(o, "note")?.unwrap_or("").to_string(),
    };
    match a.probleme() {
        Some(p) => Err(p),
        None => Ok(a),
    }
}

/// Ce qu'a donné la lecture du contenu d'un fichier.
#[derive(Debug, Clone, PartialEq)]
pub struct Lu {
    pub format: u64,
    pub liste: Vec<Abonnement>,
    /// Les abonnements laissés de côté (leur numéro dans le fichier, à partir de 1, et pourquoi).
    pub ignores: Vec<(usize, String)>,
}

/// Le contenu du fichier ; une erreur si ce n'est pas une liste d'abonnements de Paymind.
pub fn lire_contenu(o: &Objet) -> Result<Lu, String> {
    let format = o
        .get("format")
        .and_then(Value::as_u64)
        .filter(|&f| f >= 1)
        .ok_or("pas de numéro de format : ce n'est pas une liste d'abonnements")?;
    let Some(Value::Array(l)) = o.get("abonnements") else {
        return Err("pas de liste « abonnements »".into());
    };
    let mut lu = Lu { format, liste: Vec::new(), ignores: Vec::new() };
    for (i, v) in l.iter().enumerate() {
        match abonnement_depuis(v) {
            Ok(a) if lu.liste.iter().any(|b| b.id == a.id) => lu.ignores.push((i + 1, "identifiant en double".into())),
            Ok(a) => lu.liste.push(a),
            Err(e) => lu.ignores.push((i + 1, e)),
        }
    }
    Ok(lu)
}

/// `<fichier><fin>`.
fn a_cote(fichier: &Path, fin: &str) -> PathBuf {
    let mut s = fichier.as_os_str().to_owned();
    s.push(fin);
    PathBuf::from(s)
}

/// Met un fichier de côté sous un nom libre `<fichier><fin>-<date>` (puis `-2`, `-3`...), jamais par-dessus un autre :
/// une copie de ces octets, ou le fichier lui-même déplacé (`octets` vide). Le nom choisi.
fn mettre_de_cote(fichier: &Path, fin: &str, horodatage: &str, octets: Option<&[u8]>) -> io::Result<PathBuf> {
    for n in 1u32..1000 {
        let p = if n == 1 {
            a_cote(fichier, &format!("{fin}-{horodatage}"))
        } else {
            a_cote(fichier, &format!("{fin}-{horodatage}-{n}"))
        };
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&p) {
            Ok(mut f) => {
                let r = match octets {
                    Some(o) => f.write_all(o).and_then(|()| f.sync_all()),
                    None => {
                        drop(f);
                        std::fs::rename(fichier, &p)
                    }
                };
                return match r {
                    Ok(()) => Ok(p),
                    Err(e) => {
                        let _ = std::fs::remove_file(&p);
                        Err(e)
                    }
                };
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("aucun nom libre pour mettre le fichier de côté"))
}

/// Le nom d'un fichier (pour le journal).
fn nom(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// La liste des abonnements et son fichier.
#[derive(Debug, Clone)]
pub struct Abonnements {
    fichier: PathBuf,
    pub liste: Vec<Abonnement>,
    /// Pourquoi le fichier ne doit pas être réécrit (une version plus récente de Paymind, un fichier qu'on n'a pas pu
    /// mettre à l'abri) ; rien : on peut enregistrer.
    lecture_seule: Option<String>,
}

impl Abonnements {
    /// Une liste vide pour ce fichier (rien n'est lu).
    pub fn vide(fichier: &Path) -> Abonnements {
        Abonnements { fichier: fichier.to_path_buf(), liste: Vec::new(), lecture_seule: None }
    }

    pub fn fichier(&self) -> &Path {
        &self.fichier
    }

    /// La liste vient d'une version plus récente de Paymind (elle n'est jamais réécrite par celle-ci).
    pub fn plus_recente(&self) -> bool {
        self.lecture_seule.as_deref() == Some(PLUS_RECENTE)
    }

    /// Pourquoi la liste ne peut pas être enregistrée pour l'instant ; rien si elle peut l'être.
    pub fn lecture_seule(&self) -> Option<&str> {
        self.lecture_seule.as_deref()
    }

    /// Lit abonnements.json (absent : liste vide ; abîmé : sa copie de secours).
    pub fn charger(fichiers: &mut Fichiers, fichier: &Path) -> Abonnements {
        let mut a = Abonnements::vide(fichier);
        let horodatage = fichiers.journal.maintenant().compacte();
        // (Fichiers lit le fichier, ou sa copie de secours s'il est absent, abîmé ou pris)
        let mut contenu = fichiers.lire(fichier);
        let mut lu = None;
        for tour in 0..2 {
            let Some(o) = contenu.take() else { break };
            let raison = match lire_contenu(&o) {
                Ok(l) => {
                    lu = Some((l, o));
                    break;
                }
                Err(r) => r,
            };
            // Du JSON, mais pas une liste d'abonnements : le fichier lui-même est mis de côté et sa copie de secours
            // lue ; si elle ne vaut pas mieux (ou si le contenu venait déjà d'elle), elle est mise de côté aussi.
            let cible = if tour == 0 && fichier.is_file() && !fichiers.est_pris(fichier) {
                fichier.to_path_buf()
            } else {
                a_cote(fichier, ".bak")
            };
            fichiers.journal.ecrire(&format!("{} illisible ({raison})", nom(&cible)));
            if !cible.is_file() {
                break;
            }
            match mettre_de_cote(&cible, ".abime", &horodatage, None) {
                Ok(p) => fichiers.journal.ecrire(&format!("{} mis de côté dans {}", nom(&cible), nom(&p))),
                Err(e) => {
                    fichiers.journal.ecrire(&format!("{} pas pu être mis de côté : {e}", nom(&cible)));
                    a.lecture_seule = Some(format!("{} n'a pas pu être mis de côté", nom(&cible)));
                    break;
                }
            }
            if cible.as_path() != fichier {
                break;
            }
            contenu = fichiers.lire(fichier);
        }
        let Some((lu, brut)) = lu else { return a };
        if lu.format > FORMAT {
            fichiers.journal.ecrire(&format!(
                "{FICHIER} vient d'une version plus récente de Paymind (format {}) : lu sans jamais être réécrit",
                lu.format
            ));
            a.lecture_seule = Some(PLUS_RECENTE.into());
        }
        if !lu.ignores.is_empty() {
            for (n, r) in &lu.ignores {
                fichiers.journal.ecrire(&format!("{FICHIER} : abonnement numéro {n} laissé de côté ({r})"));
            }
            let octets = serde_json::to_string_pretty(&Value::Object(brut)).unwrap_or_default();
            match mettre_de_cote(fichier, ".incomplet", &horodatage, Some(octets.as_bytes())) {
                Ok(p) => fichiers.journal.ecrire(&format!("{FICHIER} recopié tel quel dans {}", nom(&p))),
                Err(e) => {
                    fichiers.journal.ecrire(&format!("{FICHIER} pas pu être recopié de côté : {e}"));
                    a.lecture_seule = Some(format!("{FICHIER} n'a pas pu être recopié de côté"));
                }
            }
        }
        a.liste = lu.liste;
        a
    }

    /// Enregistre la liste (à côté puis remplacé d'un coup ; l'ancienne version reste dans « .bak »). Refusé en
    /// lecture seule, ou si un autre programme garde le fichier ouvert.
    pub fn sauver(&mut self, fichiers: &mut Fichiers) -> Result<(), String> {
        if let Some(r) = &self.lecture_seule {
            return Err(format!("{FICHIER} n'est pas enregistré : {r}"));
        }
        let r = fichiers.sauver(&self.fichier, &liste_json(&self.liste));
        if let Err(e) = &r {
            fichiers.journal.ecrire(&format!("Abonnements non enregistrés : {e}"));
        }
        r
    }
}
