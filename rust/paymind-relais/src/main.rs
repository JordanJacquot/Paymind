//! Paymind.exe : un petit relais (repris de mindcare-relais, lui-même porté de D:\Projets\Mindcare\outils\relais.cs).
//! Le vrai programme de Paymind s'appelle Mindset.exe (paymind-appli), pour que le Gestionnaire des tâches range toute
//! la suite sur une seule ligne « Mindset ». Paymind.exe est le champ `exe` de apps.json (et sert aux raccourcis et
//! aux anciens Mindset ; contrat-appli, partie 4) : il lance Mindset.exe de son dossier avec les mêmes arguments, puis
//! se ferme aussitôt. Si Mindset.exe manque : un message en français, rien d'autre.

// Pas de console : le relais n'a aucune fenêtre à lui (seulement le message s'il y a un problème).
#![windows_subsystem = "windows"]

mod ligne;
mod message;

use std::os::windows::process::CommandExt;
use std::process::Command;

/// `CREATE_NO_WINDOW` : jamais de console (règle de la suite pour tout programme lancé). Mindset.exe est un programme
/// graphique : Windows ignore ce drapeau pour lui, sa fenêtre s'ouvre comme d'habitude.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn main() {
    let dossier = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.to_path_buf()));
    let Some(dossier) = dossier else {
        message::montrer(ligne::MESSAGE_ABSENT, false);
        std::process::exit(1);
    };
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let r = ligne::relayer(&dossier, &arguments, |exe, ligne, travail| {
        let mut c = Command::new(exe);
        c.current_dir(travail).creation_flags(CREATE_NO_WINDOW);
        if !ligne.is_empty() {
            c.raw_arg(ligne);
        }
        // (le relais n'attend pas : il se ferme aussitôt, Mindset.exe continue seul)
        c.spawn().map(|_| ())
    });
    if let Err(p) = r {
        message::montrer(&p.texte(), matches!(p, ligne::Probleme::Echec(_)));
        std::process::exit(1);
    }
}
