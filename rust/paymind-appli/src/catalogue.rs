//! La liste toute prête (étape 4 du plan) : `catalogue\catalogue.json` du dépôt, rangé dans l'exe à la construction
//! (`include_str!`), avec ses logos (`catalogue\logos`, recopiés dans `ui\logos` par build.rs : Tauri les range dans
//! l'exe avec les pages). Rien n'est téléchargé : Paymind ne va jamais sur Internet. La liste et ses prix changent
//! avec une nouvelle version de l'appli.
//!
//! Le fichier est lu une seule fois ([`catalogue`]) et vérifié entièrement ([`Catalogue::lire`]) : ids uniques et bien
//! écrits, catégories connues, prix plus grands que zéro, couleurs « #RRGGBB », noms de logos sans chemin, aucun tiret
//! long ni demi-cadratin. Les tests vérifient en plus que chaque logo existe. Un fichier refusé (impossible dans l'exe
//! livré : les tests le lisent) donnerait une liste vide, jamais un plantage.
//!
//! Ici aussi :
//! - la clé de recherche de chaque abonnement ([`cle_recherche`]) : sans accents ni majuscules, la page cherche
//!   dedans chaque mot tapé ([`correspond`], la même règle que ui\portefeuille.js) ;
//! - ce que la page dessine pour le logo d'une carte ou d'une pastille ([`logo_carte`], [`logo_rond`]), comme la
//!   maquette F et `catalogue\_apercu.html`.

use std::sync::OnceLock;

use paymind_moteur::abonnement::NOM_MAX;
use paymind_moteur::{PRIX_MAX, Rythme};
use serde_json::{Map, Value, json};

use crate::pont::{couleur_valable, id_valable};

/// Le fichier de la liste toute prête, rangé dans l'exe.
pub const TEXTE: &str = include_str!("../../../catalogue/catalogue.json");
/// Le dossier des logos, dans les pages (ui\logos : recopié par build.rs depuis catalogue\logos).
pub const DOSSIER_LOGOS: &str = "logos";
/// Le format de catalogue.json que cette version lit.
pub const FORMAT: u64 = 1;
/// La longueur la plus grande d'une offre (« Standard avec pub »), en caractères.
pub const OFFRE_MAX: usize = 120;
/// La longueur la plus grande d'un nom de catégorie.
pub const CATEGORIE_MAX: usize = 40;

/// Une catégorie de la liste : son id, son nom à l'écran, sa place, les deux couleurs des cartes sans logo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categorie {
    pub id: String,
    pub nom: String,
    pub ordre: u64,
    pub fond: String,
    pub fond_fin: String,
}

/// Un pictogramme simple, pour un abonnement sans marque (« Mutuelle santé »).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picto {
    Coeur,
    Voiture,
    Maison,
}

impl Picto {
    pub fn code(self) -> &'static str {
        match self {
            Picto::Coeur => "coeur",
            Picto::Voiture => "voiture",
            Picto::Maison => "maison",
        }
    }
}

/// Comment le logo se pose sur la carte (`affichage` de catalogue.json, voir catalogue\SOURCES.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affichage {
    /// Un logo d'une seule couleur, peint en blanc
    Blanc,
    /// Un logo en couleurs, sur une petite plaque blanche
    Plaque,
    /// Un logo en couleurs, posé tel quel
    Couleur,
    /// Pas de logo : le nom sur une plaque blanche, dans la couleur de la carte
    Texte,
    /// Un pictogramme simple
    Picto(Picto),
}

impl Affichage {
    fn depuis(t: &str) -> Option<Affichage> {
        Some(match t {
            "blanc" => Affichage::Blanc,
            "plaque" => Affichage::Plaque,
            "couleur" => Affichage::Couleur,
            "texte" => Affichage::Texte,
            "picto:coeur" => Affichage::Picto(Picto::Coeur),
            "picto:voiture" => Affichage::Picto(Picto::Voiture),
            "picto:maison" => Affichage::Picto(Picto::Maison),
            _ => return None,
        })
    }

    /// Le mot de la page (une classe CSS).
    pub fn code(self) -> &'static str {
        match self {
            Affichage::Blanc => "blanc",
            Affichage::Plaque => "plaque",
            Affichage::Couleur => "couleur",
            Affichage::Texte => "texte",
            Affichage::Picto(_) => "picto",
        }
    }

    /// Un fichier de logo est nécessaire
    fn avec_logo(self) -> bool {
        matches!(self, Affichage::Blanc | Affichage::Plaque | Affichage::Couleur)
    }
}

/// Un abonnement de la liste toute prête.
#[derive(Debug, Clone, PartialEq)]
pub struct Modele {
    pub id: String,
    pub nom: String,
    pub categorie: String,
    /// Le prix habituel, en centimes (plus grand que zéro)
    pub prix: u64,
    pub rythme: Rythme,
    /// L'offre de ce prix (« Standard avec pub »)
    pub offre: String,
    /// Le fichier du logo dans ui\logos (rien pour « texte » ou un pictogramme)
    pub logo: Option<String>,
    /// Largeur du logo divisée par sa hauteur
    pub ratio: f64,
    pub affichage: Affichage,
    /// Les deux couleurs du dégradé de la carte, et celle de son texte
    pub fond: String,
    pub fond_fin: String,
    pub texte: String,
    /// Un petit logo rond pour les pastilles (l'œuf de Dofus)
    pub pastille: Option<String>,
    /// Le prix est une moyenne ou change selon les cas
    pub a_verifier: bool,
}

/// La liste toute prête, vérifiée.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalogue {
    /// La date du relevé des prix (« 2026-11-15 »)
    pub releve: String,
    /// Les catégories, dans leur ordre
    pub categories: Vec<Categorie>,
    /// Les abonnements, dans l'ordre du fichier
    pub modeles: Vec<Modele>,
}

/// Un texte de l'écran accepté : sans tiret long ni demi-cadratin, sans chevron, sans caractère de contrôle.
pub fn texte_propre(t: &str) -> bool {
    !t.chars().any(|c| matches!(c, '\u{2013}' | '\u{2014}' | '<' | '>') || c.is_control())
}

/// Un nom de fichier de logo : des lettres minuscules, des chiffres et des tirets, puis .svg, .png ou .jpg (aucun
/// chemin).
pub fn fichier_valable(f: &str) -> bool {
    let Some((nom, ext)) = f.rsplit_once('.') else { return false };
    (1..=60).contains(&nom.len())
        && nom.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && matches!(ext, "svg" | "png" | "jpg")
}

/// Un id de la liste : 1 à 40 lettres minuscules, chiffres ou tirets.
fn id_liste(id: &str) -> bool {
    (1..=40).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && id_valable(id)
}

fn texte<'a>(o: &'a Map<String, Value>, champ: &str) -> Result<&'a str, String> {
    o.get(champ).and_then(Value::as_str).ok_or_else(|| format!("{champ} absent ou pas un texte"))
}

fn couleur(o: &Map<String, Value>, champ: &str) -> Result<String, String> {
    let c = texte(o, champ)?;
    if couleur_valable(c) { Ok(c.to_string()) } else { Err(format!("{champ} n'est pas une couleur #RRGGBB ({c})")) }
}

impl Catalogue {
    /// Lit et vérifie le texte de catalogue.json ; la première erreur trouvée est rendue.
    pub fn lire(t: &str) -> Result<Catalogue, String> {
        let v: Value =
            serde_json::from_str(t.trim_start_matches('\u{feff}')).map_err(|e| format!("pas du JSON ({e})"))?;
        let o = v.as_object().ok_or("pas un objet")?;
        if o.get("format").and_then(Value::as_u64) != Some(FORMAT) {
            return Err(format!("format différent de {FORMAT}"));
        }
        let releve = texte(o, "prix_releves_le")?.to_string();

        let mut categories = Vec::new();
        for (n, c) in o.get("categories").and_then(Value::as_array).ok_or("categories absent")?.iter().enumerate() {
            let c = c.as_object().ok_or(format!("catégorie {n} : pas un objet"))?;
            let id = texte(c, "id")?;
            let quoi = |e: String| format!("catégorie {id} : {e}");
            // (« moi » et « autres » sont les deux catégories de l'appli elle-même)
            if !id_liste(id) || id == "moi" || id == "autres" {
                return Err(quoi("id mal écrit ou réservé".into()));
            }
            if categories.iter().any(|x: &Categorie| x.id == id) {
                return Err(quoi("en double".into()));
            }
            let nom = texte(c, "nom").map_err(quoi)?;
            if nom.trim().is_empty() || nom.chars().count() > CATEGORIE_MAX || !texte_propre(nom) {
                return Err(quoi("nom vide, trop long ou avec un signe interdit".into()));
            }
            let ordre = c.get("ordre").and_then(Value::as_u64).ok_or_else(|| quoi("ordre absent".into()))?;
            if categories.iter().any(|x: &Categorie| x.ordre == ordre) {
                return Err(quoi("ordre en double".into()));
            }
            categories.push(Categorie {
                id: id.to_string(),
                nom: nom.to_string(),
                ordre,
                fond: couleur(c, "fond").map_err(quoi)?,
                fond_fin: couleur(c, "fond_fin").map_err(quoi)?,
            });
        }
        categories.sort_by_key(|c| c.ordre);

        let mut modeles: Vec<Modele> = Vec::new();
        for (n, a) in o.get("abonnements").and_then(Value::as_array).ok_or("abonnements absent")?.iter().enumerate() {
            let a = a.as_object().ok_or(format!("abonnement {n} : pas un objet"))?;
            let id = texte(a, "id")?;
            let quoi = |e: String| format!("abonnement {id} : {e}");
            if !id_liste(id) {
                return Err(quoi("id mal écrit".into()));
            }
            if modeles.iter().any(|m| m.id == id) {
                return Err(quoi("en double".into()));
            }
            let nom = texte(a, "nom").map_err(quoi)?;
            if nom.trim().is_empty() || nom.chars().count() > NOM_MAX || !texte_propre(nom) {
                return Err(quoi("nom vide, trop long ou avec un signe interdit".into()));
            }
            let categorie = texte(a, "categorie").map_err(quoi)?;
            if !categories.iter().any(|c| c.id == categorie) {
                return Err(quoi(format!("catégorie inconnue ({categorie})")));
            }
            let prix = a.get("prix_centimes").and_then(Value::as_u64).unwrap_or(0);
            if prix == 0 || prix > PRIX_MAX {
                return Err(quoi("prix absent, nul ou trop grand".into()));
            }
            let rythme =
                Rythme::depuis_code(texte(a, "rythme").map_err(quoi)?).ok_or_else(|| quoi("rythme inconnu".into()))?;
            let offre = texte(a, "offre").map_err(quoi)?;
            if offre.chars().count() > OFFRE_MAX || !texte_propre(offre) {
                return Err(quoi("offre trop longue ou avec un signe interdit".into()));
            }
            let affichage = Affichage::depuis(texte(a, "affichage").map_err(quoi)?)
                .ok_or_else(|| quoi("affichage inconnu".into()))?;
            let logo = texte(a, "logo").map_err(quoi)?;
            let logo = if logo.is_empty() {
                None
            } else if fichier_valable(logo) {
                Some(logo.to_string())
            } else {
                return Err(quoi(format!("nom de logo refusé ({logo})")));
            };
            if affichage.avec_logo() != logo.is_some() {
                return Err(quoi("logo et affichage ne vont pas ensemble".into()));
            }
            let ratio = a.get("logo_ratio").and_then(Value::as_f64).unwrap_or(1.0);
            if !(ratio.is_finite() && (0.1..=20.0).contains(&ratio)) {
                return Err(quoi("logo_ratio hors de 0,1 à 20".into()));
            }
            let pastille = match a.get("pastille") {
                None | Some(Value::Null) => None,
                Some(Value::String(p)) if fichier_valable(p) => Some(p.clone()),
                Some(_) => return Err(quoi("nom de pastille refusé".into())),
            };
            let a_verifier = match a.get("prix_a_verifier") {
                Some(Value::Bool(b)) => *b,
                _ => return Err(quoi("prix_a_verifier absent".into())),
            };
            modeles.push(Modele {
                id: id.to_string(),
                nom: nom.to_string(),
                categorie: categorie.to_string(),
                prix,
                rythme,
                offre: offre.to_string(),
                logo,
                ratio,
                affichage,
                fond: couleur(a, "fond").map_err(quoi)?,
                fond_fin: couleur(a, "fond_fin").map_err(quoi)?,
                texte: couleur(a, "texte").map_err(quoi)?,
                pastille,
                a_verifier,
            });
        }
        if modeles.is_empty() {
            return Err("aucun abonnement".into());
        }
        Ok(Catalogue { releve, categories, modeles })
    }

    /// Un abonnement de la liste, par son id.
    pub fn modele(&self, id: &str) -> Option<&Modele> {
        self.modeles.iter().find(|m| m.id == id)
    }

    /// Une catégorie de la liste, par son id.
    pub fn categorie(&self, id: &str) -> Option<&Categorie> {
        self.categories.iter().find(|c| c.id == id)
    }
}

/// La liste toute prête de l'exe, lue et vérifiée une seule fois.
pub fn catalogue() -> &'static Catalogue {
    static LU: OnceLock<Catalogue> = OnceLock::new();
    LU.get_or_init(|| Catalogue::lire(TEXTE).unwrap_or_default())
}

// ---------------------------------------------------------------------------------------------------------------
// La recherche
// ---------------------------------------------------------------------------------------------------------------

/// Une lettre sans son accent (les lettres du français et des noms de marques) ; les autres restent telles quelles.
fn sans_accent(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        _ => c,
    }
}

/// Un texte pour la recherche : en minuscules, sans accents, chaque suite de signes (espaces, apostrophes, « + »,
/// tirets...) devient une seule espace. « L'Équipe » donne « l equipe ». La page fait de même avec ce qui est tapé
/// (ui\portefeuille.js, `pourChercher`).
pub fn pour_chercher(t: &str) -> String {
    let mut s = String::new();
    for c in t.chars().flat_map(char::to_lowercase).map(sans_accent) {
        if c.is_alphanumeric() {
            s.push(c);
        } else if !s.is_empty() && !s.ends_with(' ') {
            s.push(' ');
        }
    }
    s.trim_end().to_string()
}

/// La clé de recherche d'un abonnement : son nom, ses initiales (« wow » pour World of Warcraft), son id, son offre
/// et sa catégorie.
pub fn cle_recherche(m: &Modele, categorie: &str) -> String {
    let nom = pour_chercher(&m.nom);
    let mots: Vec<&str> = nom.split(' ').filter(|x| !x.is_empty()).collect();
    let initiales: String =
        if mots.len() > 1 { mots.iter().filter_map(|x| x.chars().next()).collect() } else { String::new() };
    let morceaux = [nom.clone(), initiales, pour_chercher(&m.id), pour_chercher(&m.offre), pour_chercher(categorie)];
    morceaux.iter().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>().join(" ")
}

/// Une clé correspond à ce qui est tapé quand chaque mot tapé s'y trouve (« credit mutuel » trouve « Crédit Mutuel »).
/// Rien de tapé : tout correspond.
pub fn correspond(cle: &str, tape: &str) -> bool {
    pour_chercher(tape).split(' ').filter(|m| !m.is_empty()).all(|m| cle.contains(m))
}

// ---------------------------------------------------------------------------------------------------------------
// Les logos pour la page
// ---------------------------------------------------------------------------------------------------------------

/// La largeur d'une carte du portefeuille, en pixels : les tailles des logos sont données en centièmes de sa largeur
/// (`cqw` : elles suivent la carte, petite dans « Ajouter », grande dans la fiche).
const CARTE_PX: f64 = 212.0;

/// La taille d'un logo (largeur, hauteur) en centièmes de la carte : cette hauteur, sans dépasser cette largeur.
fn taille(ratio: f64, hauteur_px: f64, largeur_max_px: f64) -> (f64, f64) {
    let (mut l, mut h) = (hauteur_px * ratio, hauteur_px);
    if l > largeur_max_px {
        l = largeur_max_px;
        h = l / ratio;
    }
    let cqw = |px: f64| (px / CARTE_PX * 1000.0).round() / 10.0;
    (cqw(l), cqw(h))
}

/// L'adresse d'un logo dans les pages.
fn url(fichier: &str) -> String {
    format!("{DOSSIER_LOGOS}/{fichier}")
}

/// Le logo d'une carte, comme la maquette F : `{ forme, url, l, h }` (un logo blanc, en couleurs, ou sur sa plaque ;
/// tailles en centièmes de la carte), `{ forme: 'texte', texte, couleur }` (le nom sur une plaque blanche), ou
/// `{ forme: 'picto', picto }`.
pub fn logo_carte(m: &Modele) -> Value {
    match (m.affichage, &m.logo) {
        (Affichage::Picto(p), _) => json!({ "forme": "picto", "picto": p.code() }),
        (Affichage::Blanc | Affichage::Couleur | Affichage::Plaque, Some(f)) => {
            // (un logo blanc ou en couleurs fait 38 px de haut sur une carte de 212 ; sur sa plaque, 26)
            let (h, max) = if m.affichage == Affichage::Plaque { (26.0, 120.0) } else { (38.0, 120.0) };
            let (l, h) = taille(m.ratio, h, max);
            json!({ "forme": m.affichage.code(), "url": url(f), "l": l, "h": h })
        }
        _ => json!({ "forme": "texte", "texte": m.nom, "couleur": m.fond }),
    }
}

/// Au delà de ce rapport largeur / hauteur, un logo est illisible dans un rond de « Bientôt » : le rond montre alors
/// l'emblème de la marque (`pastille`), sinon ses initiales.
const RATIO_ROND: f64 = 1.6;

/// Le logo d'une pastille ronde de « Bientôt » (30 px) : la pastille propre de l'abonnement (l'œuf de Dofus, l'emblème
/// de Crédit Mutuel), posée comme son logo (en blanc, en couleurs ou sur fond blanc), sinon son logo s'il n'est pas
/// trop large pour un rond, ou son pictogramme ; rien sinon (la pastille montre alors ses initiales et son nom).
pub fn logo_rond(m: &Modele) -> Option<Value> {
    if let Some(p) = &m.pastille {
        let forme = match m.affichage {
            Affichage::Blanc | Affichage::Plaque => m.affichage.code(),
            _ => Affichage::Couleur.code(),
        };
        return Some(json!({ "forme": forme, "url": url(p) }));
    }
    match (m.affichage, &m.logo) {
        (Affichage::Picto(p), _) => Some(json!({ "forme": "picto", "picto": p.code() })),
        (Affichage::Blanc | Affichage::Couleur | Affichage::Plaque, Some(f)) if m.ratio <= RATIO_ROND => {
            Some(json!({ "forme": m.affichage.code(), "url": url(f) }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn racine() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
    }

    #[test]
    fn la_liste_de_l_exe_est_valable() {
        let c = Catalogue::lire(TEXTE).expect("catalogue.json refusé");
        assert_eq!(c.modeles.len(), 89);
        assert_eq!(c.categories.len(), 11);
        assert_eq!(&c, catalogue());
        // Dans l'ordre des catégories
        let ordres: Vec<u64> = c.categories.iter().map(|x| x.ordre).collect();
        assert!(ordres.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(c.categories[0].id, "films");
        // Ceux que le plan nomme
        for id in ["netflix", "dofus", "worldofwarcraft", "creditmutuel", "spotify", "free", "axa"] {
            assert!(c.modele(id).is_some(), "{id} manque");
        }
        let netflix = c.modele("netflix").unwrap();
        assert_eq!((netflix.prix, netflix.rythme, netflix.categorie.as_str()), (799, Rythme::Mois, "films"));
        assert_eq!(c.modele("dofus").unwrap().pastille.as_deref(), Some("dofus-oeuf.png"));
        // Chaque id unique, chaque prix plus grand que zéro, chaque catégorie connue (vérifiés par lire, revus ici)
        let mut ids: Vec<&str> = c.modeles.iter().map(|m| m.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), c.modeles.len());
        assert!(c.modeles.iter().all(|m| m.prix > 0 && c.categorie(&m.categorie).is_some()));
    }

    /// Chaque logo et chaque pastille existe, dans catalogue\logos et dans sa copie ui\logos (rangée dans l'exe),
    /// à l'identique.
    #[test]
    fn chaque_logo_existe() {
        let source = racine().join("catalogue").join("logos");
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join(DOSSIER_LOGOS);
        let c = catalogue();
        let mut n = 0;
        for f in c.modeles.iter().flat_map(|m| m.logo.iter().chain(m.pastille.iter())) {
            let a = std::fs::read(source.join(f)).unwrap_or_else(|_| panic!("{f} manque dans catalogue\\logos"));
            let b = std::fs::read(ui.join(f)).unwrap_or_else(|_| panic!("{f} manque dans ui\\logos (build.rs)"));
            assert!(!a.is_empty() && a == b, "{f} : la copie de ui\\logos diffère");
            n += 1;
        }
        assert!(n >= 76, "{n} logos");
    }

    /// Règle de la suite : aucun tiret long ni demi-cadratin dans ce que la page montre.
    #[test]
    fn aucun_tiret_long() {
        assert!(!TEXTE.contains(['\u{2013}', '\u{2014}']));
        for m in &catalogue().modeles {
            assert!(texte_propre(&m.nom) && texte_propre(&m.offre), "{}", m.id);
        }
    }

    /// Contre-épreuve : chaque erreur est refusée.
    #[test]
    fn listes_fausses_refusees() {
        type Changement<'a> = &'a dyn Fn(&mut Value);
        let bon = |f: Changement| {
            let mut v: Value = serde_json::from_str(TEXTE).unwrap();
            f(&mut v);
            Catalogue::lire(&v.to_string())
        };
        assert!(bon(&|_| {}).is_ok());
        let refuses: [(&str, Changement); 14] = [
            ("id en double", &|v| v["abonnements"][1]["id"] = json!("netflix")),
            ("catégorie inconnue", &|v| v["abonnements"][0]["categorie"] = json!("video")),
            ("prix nul", &|v| v["abonnements"][0]["prix_centimes"] = json!(0)),
            ("prix négatif", &|v| v["abonnements"][0]["prix_centimes"] = json!(-5)),
            ("prix trop grand", &|v| v["abonnements"][0]["prix_centimes"] = json!(PRIX_MAX + 1)),
            ("tiret long", &|v| v["abonnements"][0]["offre"] = json!("Standard \u{2014} avec pub")),
            ("demi-cadratin", &|v| v["abonnements"][0]["nom"] = json!("Net\u{2013}flix")),
            ("chemin de logo", &|v| v["abonnements"][0]["logo"] = json!("../../secret.svg")),
            ("logo sans extension", &|v| v["abonnements"][0]["logo"] = json!("netflix")),
            ("couleur", &|v| v["abonnements"][0]["fond"] = json!("red")),
            ("rythme", &|v| v["abonnements"][0]["rythme"] = json!("jour")),
            ("affichage", &|v| v["abonnements"][0]["affichage"] = json!("gros")),
            ("catégorie réservée", &|v| v["categories"][0]["id"] = json!("moi")),
            ("format", &|v| v["format"] = json!(2)),
        ];
        for (quoi, f) in refuses {
            assert!(bon(f).is_err(), "{quoi} accepté");
        }
        assert!(Catalogue::lire("pas du json").is_err());
        assert!(Catalogue::lire("[]").is_err());
    }

    #[test]
    fn recherche_sans_accents_ni_majuscules() {
        assert_eq!(pour_chercher("Crédit Mutuel"), "credit mutuel");
        assert_eq!(pour_chercher("  L'Équipe  "), "l equipe");
        assert_eq!(pour_chercher("Disney+"), "disney");
        assert_eq!(pour_chercher("ÇA VA"), "ca va");
        let c = catalogue();
        let trouve = |tape: &str| -> Vec<&str> {
            c.modeles
                .iter()
                .filter(|m| correspond(&cle_recherche(m, &c.categorie(&m.categorie).unwrap().nom), tape))
                .map(|m| m.id.as_str())
                .collect()
        };
        assert_eq!(trouve("credit mutuel"), ["creditmutuel"]);
        assert_eq!(trouve("CRÉDIT MUTUEL"), ["creditmutuel"]);
        assert_eq!(trouve("mutuel credit"), ["creditmutuel"]);
        assert!(trouve("mutuel").contains(&"creditmutuel"));
        assert!(trouve("wow").contains(&"worldofwarcraft"));
        assert!(trouve("World of").contains(&"worldofwarcraft"));
        assert_eq!(trouve("dofus"), ["dofus"]);
        assert!(trouve("netflix").contains(&"netflix"));
        assert!(trouve("Équipe").contains(&"lequipe"), "{:?}", trouve("Équipe"));
        assert!(trouve("zzz introuvable").is_empty());
        // Rien de tapé : tout
        assert_eq!(trouve("").len(), c.modeles.len());
        assert_eq!(trouve("  ").len(), c.modeles.len());
        // Les clés n'ont que des lettres sans accent, des chiffres et des espaces : la page, qui ôte les accents
        // de ce qui est tapé d'une autre façon (normalize('NFD')), trouve la même chose
        for m in &c.modeles {
            let k = cle_recherche(m, &c.categorie(&m.categorie).unwrap().nom);
            assert!(k.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b' '), "{k}");
        }
    }

    #[test]
    fn logos_comme_la_maquette() {
        let c = catalogue();
        // Netflix : blanc, 38 px de haut
        let n = logo_carte(c.modele("netflix").unwrap());
        assert_eq!(n["forme"], "blanc");
        assert_eq!(n["url"], "logos/netflix.svg");
        assert_eq!(n["h"], 17.9);
        // Crédit Mutuel : sur sa plaque, 120 px de large au plus
        let cm = logo_carte(c.modele("creditmutuel").unwrap());
        assert_eq!(cm["forme"], "plaque");
        assert_eq!(cm["l"], 56.6);
        // Dofus : en couleurs, et son œuf dans les pastilles
        let d = c.modele("dofus").unwrap();
        assert_eq!(logo_carte(d)["forme"], "couleur");
        assert_eq!(logo_rond(d).unwrap()["url"], "logos/dofus-oeuf.png");
        // Un pictogramme ; le nom sur une plaque
        let picto = c.modeles.iter().find(|m| matches!(m.affichage, Affichage::Picto(_))).unwrap();
        assert_eq!(logo_carte(picto)["forme"], "picto");
        let t = c.modeles.iter().find(|m| m.affichage == Affichage::Texte).unwrap();
        assert_eq!(logo_carte(t)["texte"], t.nom.as_str());
        // Un logo trop large pour un rond : l'emblème de la marque, posé comme son logo (Crédit Mutuel sur fond blanc,
        // Amazon Prime en blanc) ; sans emblème, rien (les initiales) ; un logo qui tient dans un rond : lui-même
        let rond = |id: &str| logo_rond(c.modele(id).unwrap());
        assert_eq!(rond("creditmutuel"), Some(json!({ "forme": "plaque", "url": "logos/creditmutuel-embleme.svg" })));
        assert_eq!(rond("amazonprime"), Some(json!({ "forme": "blanc", "url": "logos/amazon.svg" })));
        assert_eq!(rond("dofus"), Some(json!({ "forme": "couleur", "url": "logos/dofus-oeuf.png" })));
        assert_eq!(rond("lefigaro"), None);
        assert_eq!(rond("netflix"), Some(json!({ "forme": "blanc", "url": "logos/netflix.svg" })));
        for m in &c.modeles {
            let large = m.logo.is_some() && m.ratio > RATIO_ROND;
            match logo_rond(m) {
                Some(r) if large => {
                    assert!(m.pastille.is_some() && r["url"] == url(m.pastille.as_ref().unwrap()), "{}", m.id)
                }
                None => assert!(m.pastille.is_none() && (large || m.logo.is_none()), "{} : pas de rond", m.id),
                Some(_) => {}
            }
        }
        let t = c.modeles.iter().find(|m| m.affichage == Affichage::Texte).unwrap();
        assert!(logo_rond(t).is_none());
        // Chaque taille dans la carte
        for m in &c.modeles {
            let l = logo_carte(m);
            if let (Some(a), Some(b)) = (l["l"].as_f64(), l["h"].as_f64()) {
                assert!(a > 0.0 && a <= 57.0 && b > 0.0 && b <= 18.0, "{} : {a} x {b}", m.id);
            }
        }
    }
}
