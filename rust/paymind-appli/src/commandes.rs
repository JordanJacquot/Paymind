//! Les commandes que la page de Paymind peut appeler : les trois seules, chacune déclarée dans le manifeste de build.rs
//! et autorisée à la fenêtre « principale » seulement (capabilities\principale.json). Elles ne lisent ni n'écrivent
//! rien sur le disque et ne lancent rien : elles passent l'état à la page (le kit d'écrans de la suite), notent ce
//! qu'elle voit, et donnent ses messages vérifiés au receveur de l'appli. Reprises de mindcare-appli.

use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::Value;
use tauri::State;
use tauri::ipc::Channel;

use crate::pont::{self, Habillage, MessagePage};

/// L'état de départ du kit ([`crate::ecran::etat`]), et les couleurs du thème s'il y en a (les variables CSS de
/// `suite_commun::themes::Theme::variables_css` : `P`, `S`, `T`, `fond`, `fond2` ; une variable au nom ou à la couleur
/// mal écrits n'est pas passée, la page vérifie aussi). Le thème, les couleurs d'un pack et ses images s'ajoutent par
/// [`Page::habiller`].
pub fn etat_page(version: &str, variables: &[(&str, String)]) -> Value {
    let mut e = crate::ecran::etat(version);
    let couleurs = pont::couleurs_json(variables);
    if !couleurs.is_empty() {
        e["couleurs"] = Value::Object(couleurs);
    }
    e
}

/// Le journal de l'appli, vu d'ici (une ligne de texte)
type Journal = Box<dyn Fn(&str) + Send + Sync>;

/// Ce que l'appli fait d'un message de la page ([`Page::recevoir`]). Appelé pendant la commande `action`, sur le fil
/// où Tauri traite les commandes synchrones : le fil de la fenêtre.
pub type Receveur = Arc<dyn Fn(MessagePage) + Send + Sync>;

/// L'état partagé entre l'appli et sa page : le dernier état connu, l'habillage du thème, le canal ouvert par la page
/// (un seul : une page rechargée reprend sa place), ce que la page a dit de sa visibilité, et le receveur de ses
/// messages.
pub struct Page {
    etat: Mutex<Value>,
    habillage: Mutex<Option<Habillage>>,
    canal: Mutex<Option<Channel<Value>>>,
    visible: Mutex<Option<bool>>,
    receveur: Mutex<Option<Receveur>>,
    journal: Journal,
}

/// Un verrou, même si un autre fil s'est arrêté en le tenant (la valeur gardée reste bonne : un simple
/// remplacement)
fn tenir<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Page {
    pub fn new(etat: Value, journal: impl Fn(&str) + Send + Sync + 'static) -> Page {
        Page {
            etat: Mutex::new(etat),
            habillage: Mutex::new(None),
            canal: Mutex::new(None),
            visible: Mutex::new(None),
            receveur: Mutex::new(None),
            journal: Box::new(journal),
        }
    }

    /// Le dernier état connu, habillé du thème (ce que la page reçoit)
    pub fn etat(&self) -> Value {
        let mut e = tenir(&self.etat).clone();
        if let Some(h) = tenir(&self.habillage).as_ref() {
            h.poser(&mut e);
        }
        e
    }

    /// Envoie à la page, si elle suit déjà
    fn envoyer(&self, message: Value) {
        if let Some(canal) = tenir(&self.canal).as_ref()
            && let Err(e) = canal.send(message)
        {
            (self.journal)(&format!("Page : état non envoyé ({e})"));
        }
    }

    /// Un nouvel état : gardé, et envoyé à la page si elle suit déjà
    pub fn changer(&self, etat: Value) {
        *tenir(&self.etat) = etat;
        self.envoyer(self.etat());
    }

    /// Le thème pour l'écran (thème de la suite, couleurs et images d'un pack), ajouté à chaque état envoyé, comme
    /// `Send-Vue` ; l'état est renvoyé tout de suite.
    pub fn habiller(&self, habillage: Habillage) {
        *tenir(&self.habillage) = Some(habillage);
        self.envoyer(self.etat());
    }

    /// Un ordre au kit, sans changer l'état (kit.js : `{ kit: 'message', texte, bravo }`, `{ kit: 'aller', ecran }`...).
    /// Autre chose qu'un ordre n'est pas envoyé.
    pub fn ordre(&self, ordre: Value) {
        if ordre.get("kit").is_some_and(Value::is_string) {
            self.envoyer(ordre);
        }
    }

    /// Le receveur des messages de la page (le programme le pose) ; sans lui, chaque message est noté au journal.
    pub fn recevoir(&self, receveur: impl Fn(MessagePage) + Send + Sync + 'static) {
        *tenir(&self.receveur) = Some(Arc::new(receveur));
    }

    /// Ce que la page a dit en dernier : visible (`Some(true)`), cachée, ou rien encore
    pub fn visible(&self) -> Option<bool> {
        *tenir(&self.visible)
    }
}

/// Les lignes du journal quand la page change de visibilité
pub const JOURNAL_CACHEE: &str = "Écran : caché, en pause";
pub const JOURNAL_VISIBLE: &str = "Écran : de nouveau à l'écran";

/// La page commence à suivre l'état de l'appli : il lui est envoyé tout de suite, puis à chaque changement
/// ([`Page::changer`]) ; les ordres passent par le même canal ([`Page::ordre`]).
#[tauri::command]
pub fn suivre_page(canal: Channel<Value>, page: State<'_, Page>) -> Result<(), String> {
    canal.send(page.etat()).map_err(|e| format!("état non envoyé ({e})"))?;
    *tenir(&page.canal) = Some(canal);
    Ok(())
}

/// La page est devenue visible ou cachée (document.visibilityState) : noté au journal quand cela change. Cachée,
/// WebView2 met la page en pause (la veille, `SetIsVisible`, décidée par la vie de l'appli).
#[tauri::command]
pub fn noter_visibilite(visible: bool, page: State<'_, Page>) {
    let avant = tenir(&page.visible).replace(visible);
    if avant != Some(visible) {
        (page.journal)(if visible { JOURNAL_VISIBLE } else { JOURNAL_CACHEE });
    }
}

/// Un message du kit (un geste, l'écran montré, « Retour » depuis l'accueil, une erreur, une réponse) : vérifié
/// ([`pont::lire_message`] : taille, type, champs), puis donné au receveur de l'appli. Une erreur de la page va
/// toujours au journal. Un message refusé est noté au journal, et la page reçoit seulement « refusé ».
#[tauri::command]
pub fn action(message: Value, page: State<'_, Page>) -> Result<(), String> {
    let m = match pont::lire_message(&message) {
        Ok(m) => m,
        Err(r) => {
            (page.journal)(&format!("Écran : message refusé ({})", r.raison()));
            return Err("refusé".into());
        }
    };
    if matches!(m, MessagePage::Erreur { .. }) {
        (page.journal)(&pont::ligne_journal(&m));
        return Ok(());
    }
    // (copié hors du verrou : le receveur peut changer l'état, ou se remplacer)
    let receveur = tenir(&page.receveur).clone();
    match receveur {
        Some(r) => r(m),
        None => (page.journal)(&pont::ligne_journal(&m)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn etat_page_avec_et_sans_theme() {
        let theme = suite_commun::themes::Theme::neon();
        let e = etat_page("0.1.0", &theme.variables_css());
        assert_eq!(e["app"], json!({ "nom": "Paymind", "version": "0.1.0", "logo": "logo.svg" }));
        assert_eq!(
            e["couleurs"],
            json!({ "P": "#00E5FF", "S": "#B04BFF", "T": "#FF2EB5", "fond": "#06050F", "fond2": "#0C0920" })
        );
        let e = etat_page("0.1.0", &[]);
        assert!(e.get("couleurs").is_none());
    }

    #[test]
    fn habillage_ajoute_a_chaque_etat() {
        let p = Page::new(json!({ "app": { "nom": "Paymind" } }), |_| {});
        assert!(p.etat().get("theme").is_none());
        p.habiller(Habillage::depuis(&suite_commun::themes::charger(Some("rubis"), &[]), false));
        assert_eq!(p.etat()["theme"], "rubis");
        p.changer(json!({ "app": { "nom": "Paymind" }, "ecran": "abonnements" }));
        assert_eq!(p.etat()["theme"], "rubis");
        assert_eq!(p.etat()["ecran"], "abonnements");
    }
}
