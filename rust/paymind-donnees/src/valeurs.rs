//! Les règles de PowerShell que les données suivent (reprises de `ps.rs` de mindcare-moteur, seulement ce qui sert ici) :
//! un champ se lit sans tenir compte des majuscules, vrai ou faux comme `if ($x)`, `-eq` sans tenir compte des
//! majuscules.

use serde_json::{Map, Value};

/// Un objet JSON : ses champs, dans l'ordre du fichier.
pub type Objet = Map<String, Value>;

/// Un champ d'un objet, sans tenir compte des majuscules (le nom exact d'abord).
pub fn champ<'a>(o: &'a Objet, nom: &str) -> Option<&'a Value> {
    o.get(nom).or_else(|| o.iter().find(|(k, _)| k.eq_ignore_ascii_case(nom)).map(|(_, v)| v))
}

/// Écrit un champ comme `$o[$nom] = ...` dans une table ordonnée : un champ du même nom (majuscules ou pas) est
/// remplacé à sa place, sinon il est ajouté à la fin.
pub fn mettre(o: &mut Objet, nom: &str, valeur: Value) {
    let cle = o.keys().find(|k| k.eq_ignore_ascii_case(nom)).cloned();
    o.insert(cle.unwrap_or_else(|| nom.to_string()), valeur);
}

/// Vrai ou faux, comme `if ($x)` en PowerShell : rien, un texte vide, 0 et une liste vide sont faux ; une liste d'un
/// seul élément vaut cet élément.
pub fn vrai(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => match a.as_slice() {
            [] => false,
            [x] => vrai(Some(x)),
            _ => true,
        },
        Some(Value::Object(_)) => true,
    }
}

/// `-eq` entre deux textes : sans tenir compte des majuscules.
pub fn egal(a: &str, b: &str) -> bool {
    a == b || a.chars().flat_map(char::to_lowercase).eq(b.chars().flat_map(char::to_lowercase))
}

/// La clé d'une table PowerShell (`@{}`) : sans tenir compte des majuscules.
pub fn cle(t: &str) -> String {
    t.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn comme_powershell() {
        assert!(!vrai(None) && !vrai(Some(&json!(""))) && !vrai(Some(&json!(0))) && !vrai(Some(&json!([]))));
        assert!(!vrai(Some(&json!([0]))) && vrai(Some(&json!([0, 0]))) && vrai(Some(&json!({}))));
        assert!(egal("Néon", "nÉON") && !egal("neon", "rubis"));
        let mut o = Objet::new();
        mettre(&mut o, "Theme", json!("neon"));
        mettre(&mut o, "theme", json!("rubis"));
        assert_eq!(o.len(), 1);
        assert_eq!(champ(&o, "THEME"), Some(&json!("rubis")));
    }
}
