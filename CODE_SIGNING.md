# Code signing policy

*Version française plus bas : [Politique de signature du code](#politique-de-signature-du-code).*

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

> **Status:** the application to SignPath Foundation is in progress. Until it is accepted, the releases are not
> Authenticode signed. Mindset only installs or updates Paymind from a `Paymind.zip` that carries a valid RSA
> signature of the maintainer (`Paymind.zip.sig`).

## What is signed

- `Mindset.exe` (the Paymind program, named like every program of the suite) and `Paymind.exe` (a small relay that
  starts it), both built from this repository (`rust\paymind-appli` and `rust\paymind-relais`).
- `version.json`, the third file of `Paymind.zip`, is data (the name and version of the app), not a program, and is
  not signed.
- `fichiers\Paymind.ico`, the fourth and last file of `Paymind.zip`, is an image (the icon of Paymind, the same as in
  the programs, shown as its logo by the 3D console of Mindset), not a program, and is not signed.
- The page of the app, the catalogue of subscriptions (`catalogue\catalogue.json`) and its logos are stored inside
  `Mindset.exe`: there is no other file. The program includes open source libraries: Tauri and the other Rust
  libraries listed in `rust\Cargo.lock` (licenses compatible with MIT, checked at each build by `cargo deny`), and
  three.js (MIT license, `rust\paymind-appli\ui\three\LICENCE-three.js.txt`).

## How a release is built and signed

1. The version number is written in `rust\paymind-appli\Cargo.toml`, committed, and the version is tagged in this
   repository (`v<version>`).
2. The [Release workflow](.github/workflows/release.yml) builds `Mindset.exe` and `Paymind.exe` on a GitHub hosted
   runner (`outils\construire-rust.ps1`: `cargo build --release --locked`), from the source of that commit only, makes
   `Paymind.zip` (`outils\zip-rust.ps1`, checked by `outils\verifier-zip-rust.ps1`), attests it (GitHub build
   provenance: this repository, this workflow, this commit) and uploads the `Paymind` folder as a workflow artifact.
3. The workflow sends a signing request to SignPath. SignPath checks that the artifact comes from this workflow and
   this repository. **Every signing request is approved manually** by an approver (see below).
4. The maintainer downloads the zip produced by the workflow, checks its attestation, adds the detached RSA signature
   `Paymind.zip.sig` (Mindset uses it to verify installations and updates; it does not change any file inside the
   zip) and publishes the Release (`outils\publier.ps1`).

## Team and roles

| Role | Members |
|---|---|
| Committers and reviewers | [Jordan Jacquot](https://github.com/JordanJacquot) |
| Approvers | [Jordan Jacquot](https://github.com/JordanJacquot) |

- Committers can change the code of this repository. Changes proposed by anyone else (pull requests) are reviewed
  by a committer before they are merged.
- Every signing request is approved by an approver.
- All team members use multi-factor authentication on GitHub and on SignPath.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user
or the person installing or operating it.

Paymind has no account, no advertising, no telemetry and no statistics. It never connects to the Internet: it sends
nothing and downloads nothing, not even after a click. The page of the app can only load the files stored in the
program (its content security policy forbids any other address). Paymind does not update itself: it is installed and
updated by Mindset (see [Mindset's own policy](https://github.com/JordanJacquot/Mindset/blob/main/CODE_SIGNING.md)).

What stays on your PC:
- Your subscriptions and contracts (name, price, rhythm, next payment date, reminder, note) are saved in
  `%LOCALAPPDATA%\Mindset\Paymind` (or `X:\Mindset\Donnees\Paymind` when Mindset is installed on another drive `X:`),
  with a backup copy, and are never sent anywhere.
- The reminders are shown by Mindset near the clock: Paymind only writes a small file in its data folder that Mindset
  reads. The card of Paymind in Mindset (total of the month, next payment) is written the same way.
- When Paymind is opened without Mindset installed, it shows a message with the address of Mindset's download page,
  as text only: it never opens it.

The catalogue of subscriptions shows the logos of their brands. These logos are trademarks of their owners; they are
only shown to help you recognize a subscription in the list, and Paymind is not linked to these brands. Their sources
and licenses are listed in [catalogue/SOURCES.md](catalogue/SOURCES.md). The prices of the catalogue are usual prices,
that you can change.

## System changes and uninstallation

- Paymind changes nothing in Windows: no administrator rights, no service, no scheduled task, nothing written in the
  registry. It only reads, in the registry, where Mindset is installed (`InstallLocation` of Mindset's uninstall key).
- Its files: the program in Mindset's programs folder, its data (subscriptions, reminders, settings, journal) in
  `%LOCALAPPDATA%\Mindset\Paymind` (or `X:\Mindset\Donnees\Paymind`), and the cache of the page (WebView2) in Mindset's
  data folder, `WebView2-Tauri\Paymind` (or `ecran-webview` in Paymind's data folder when that one cannot be written).
- Paymind is installed, updated and uninstalled by Mindset: "Désinstaller" on the card of Paymind in Mindset removes
  the program; its data folder is kept, so that a new installation finds your subscriptions. The uninstaller of
  Mindset (`Desinstaller Mindset.exe`, also in "Installed apps" of Windows) removes Paymind with Mindset, and its data
  folder too when you choose to delete the data.

---

# Politique de signature du code

Signature gratuite du code fournie par [SignPath.io](https://about.signpath.io), certificat de la
[SignPath Foundation](https://signpath.org).

> **Où on en est :** la demande auprès de la SignPath Foundation est en cours. En attendant, les versions ne sont pas
> signées (Authenticode). Mindset n'installe ou ne met à jour Paymind qu'à partir d'un `Paymind.zip` qui porte une
> signature RSA valide du responsable (`Paymind.zip.sig`).

## Ce qui est signé

- `Mindset.exe` (le programme de Paymind, au même nom que chaque programme de la suite) et `Paymind.exe` (un petit
  relais qui le lance), tous deux construits depuis ce dépôt (`rust\paymind-appli` et `rust\paymind-relais`).
- `version.json`, le troisième fichier de `Paymind.zip`, est une donnée (le nom et la version de l'appli), pas un
  programme, et n'est pas signé.
- `fichiers\Paymind.ico`, le quatrième et dernier fichier de `Paymind.zip`, est une image (l'icône de Paymind, la même
  que dans les programmes, que la console 3D de Mindset affiche comme logo), pas un programme, et n'est pas signée.
- La page de l'appli, la liste toute prête des abonnements (`catalogue\catalogue.json`) et ses logos sont rangés dans
  `Mindset.exe` : il n'y a aucun autre fichier. Le programme contient des bibliothèques libres : Tauri et les autres
  bibliothèques Rust listées dans `rust\Cargo.lock` (licences compatibles avec la licence MIT, vérifiées à chaque
  construction par `cargo deny`), et three.js (licence MIT, `rust\paymind-appli\ui\three\LICENCE-three.js.txt`).

## Comment une version est construite et signée

1. Le numéro de version est écrit dans `rust\paymind-appli\Cargo.toml`, commité, et la version reçoit son étiquette
   dans ce dépôt (`v<version>`).
2. Le [workflow Release](.github/workflows/release.yml) construit `Mindset.exe` et `Paymind.exe` sur une machine de
   GitHub (`outils\construire-rust.ps1` : `cargo build --release --locked`), seulement depuis le code de ce commit,
   fait `Paymind.zip` (`outils\zip-rust.ps1`, vérifié par `outils\verifier-zip-rust.ps1`), l'atteste (attestation de
   construction GitHub : ce dépôt, ce workflow, ce commit) et dépose le dossier `Paymind` comme artefact.
3. Le workflow envoie une demande de signature à SignPath, qui vérifie que l'artefact vient bien de ce workflow et de
   ce dépôt. **Chaque demande est approuvée à la main** par un approbateur (voir plus bas).
4. Le responsable récupère le zip construit par le workflow, vérifie son attestation, ajoute la signature RSA séparée
   `Paymind.zip.sig` (Mindset s'en sert pour vérifier les installations et les mises à jour ; elle ne change aucun
   fichier du zip) et publie la version (`outils\publier.ps1`).

## Équipe et rôles

| Rôle | Membres |
|---|---|
| Auteurs et relecteurs | [Jordan Jacquot](https://github.com/JordanJacquot) |
| Approbateurs | [Jordan Jacquot](https://github.com/JordanJacquot) |

- Les auteurs peuvent modifier le code de ce dépôt. Les changements proposés par quelqu'un d'autre (pull requests)
  sont relus par un auteur avant d'être acceptés.
- Chaque demande de signature est approuvée par un approbateur.
- Tous les membres de l'équipe utilisent la double authentification sur GitHub et sur SignPath.

## Confidentialité

Ce programme n'envoie aucune information vers d'autres systèmes en réseau, sauf si l'utilisateur, ou la personne qui
l'installe ou s'en sert, le demande.

Paymind n'a ni compte, ni publicité, ni télémétrie, ni statistiques. Elle ne se connecte jamais à Internet : elle
n'envoie rien et ne télécharge rien, même après un clic. La page de l'appli ne peut charger que les fichiers rangés
dans le programme (sa politique de sécurité du contenu interdit toute autre adresse). Paymind ne se met pas à jour
elle-même : elle est installée et mise à jour par Mindset (voir
[la politique de Mindset](https://github.com/JordanJacquot/Mindset/blob/main/CODE_SIGNING.md)).

Ce qui reste sur ton PC :
- Tes abonnements et contrats (nom, prix, rythme, date du prochain paiement, rappel, note) sont gardés dans
  `%LOCALAPPDATA%\Mindset\Paymind` (ou `X:\Mindset\Donnees\Paymind` quand Mindset est installé sur un autre disque
  `X:`), avec une copie de secours, et ne sont jamais envoyés nulle part.
- Les rappels sont montrés par Mindset près de l'horloge : Paymind écrit seulement un petit fichier dans son dossier
  de données, que Mindset lit. La carte de Paymind dans Mindset (total du mois, prochain paiement) est écrite de la
  même façon.
- Quand Paymind est ouverte sans que Mindset soit installé, elle montre un message avec l'adresse de la page de
  téléchargement de Mindset, seulement en texte : elle ne l'ouvre jamais.

La liste toute prête montre les logos des marques. Ces logos sont des marques de leurs propriétaires ; ils sont
seulement montrés pour t'aider à reconnaître un abonnement dans la liste, et Paymind n'est pas liée à ces marques.
Leurs sources et leurs licences sont dans [catalogue/SOURCES.md](catalogue/SOURCES.md). Les prix de la liste sont des
prix habituels, que tu peux changer.

## Ce qui change sur le PC, et la désinstallation

- Paymind ne change rien dans Windows : pas de droits administrateur, pas de service, pas de tâche planifiée, rien
  d'écrit dans le registre. Elle lit seulement dans le registre où Mindset est installé (`InstallLocation` de la clé
  de désinstallation de Mindset).
- Ses fichiers : le programme dans le dossier des programmes de Mindset, ses données (abonnements, rappels, réglages,
  journal) dans `%LOCALAPPDATA%\Mindset\Paymind` (ou `X:\Mindset\Donnees\Paymind`), et le cache de la page (WebView2)
  dans le dossier de données de Mindset, `WebView2-Tauri\Paymind` (ou `ecran-webview` dans le dossier de données de
  Paymind quand celui-là ne peut pas être écrit).
- Paymind est installée, mise à jour et désinstallée par Mindset : « Désinstaller » sur la carte de Paymind dans
  Mindset supprime le programme ; son dossier de données est gardé, pour qu'une nouvelle installation retrouve tes
  abonnements. Le désinstalleur de Mindset (`Desinstaller Mindset.exe`, aussi dans « Applications installées » de
  Windows) supprime Paymind avec Mindset, et aussi son dossier de données quand tu choisis de supprimer les données.
