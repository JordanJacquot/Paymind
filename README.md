# Paymind

Paymind aide à savoir combien coûtent tous ses abonnements et contrats, et quand ils se renouvellent.
On les ajoute depuis une grande liste toute prête (ou à la main) ; Paymind fait le bilan du mois et de l'année et
prévient avant chaque paiement. Tout reste sur le PC : rien n'est envoyé sur Internet.

Paymind fait partie de la suite Mindset : elle s'installe, se met à jour et s'affiche dans Mindset.
Le cadrage est dans [CADRAGE.md](CADRAGE.md), la maquette retenue dans `maquettes\proposition-f.html` (le portefeuille de cartes).

Les logos des marques restent la propriété de leurs marques ; ils servent seulement à reconnaître un abonnement.

## Plan

- [x] **1. Paymind apparaît dans Mindset** : une page vide s'ouvre dans l'onglet de Mindset, suit le thème et se ferme
  proprement. Vérifié par les essais automatiques (repris de Mindcare) et un essai dans un Mindset de test.
- [x] **2. Le calcul des comptes** : chaque rythme (semaine, mois, trimestre, an) ramené au mois et à l'année, la date du
  prochain paiement qui avance seule après un renouvellement automatique, les totaux par catégorie, la liste « Bientôt » ;
  les abonnements gardés sur le PC avec une copie de secours. Vérifié par des tests sur les cas pièges (le 31 dans un
  mois de 30 jours, le 29 février, le passage d'année).
- [x] **3. Le portefeuille** : l'accueil de la maquette F (cartes, total par mois et par an, « Bientôt ») ;
  « Ajoute ton premier abonnement » quand la liste est vide. Vérifié par des captures comparées à la maquette.
- [x] **4. La liste toute prête** : environ 80 abonnements avec leur logo officiel et leur prix habituel (streaming,
  musique, jeux dont Dofus et World of Warcraft, opérateurs, banques et assurances dont Crédit Mutuel, sport, presse,
  logiciels), la recherche et « Un abonnement à moi ». Vérifié : ajouter Netflix fait apparaître sa carte et change le total.
- [x] **5. La fiche** : prix, rythme, date, renouvellement automatique, pause, suppression ; le rappel au choix (jamais,
  le jour même, la veille par défaut, 3 jours avant, 7 jours avant).
- [x] **6. Les rappels et la carte dans Mindset** : une bulle près de l'horloge au moment choisi (montrée au démarrage
  suivant si le PC était éteint) ; la carte de Mindset montre le total du mois et le prochain paiement.
- [ ] **7. Première version 0.1.0** : publiée avec l'accord de Jordan et ajoutée à la liste des applis de Mindset.

Les prix de la liste toute prête sont datés : ils sont remis à jour à chaque nouvelle version.

## Construire et essayer

Paymind est écrite en Rust avec Tauri, sur le modèle de Mindcare (`rust\`) : `paymind-vie` (la vie avec Mindset),
`paymind-donnees` (dossier de données, journal, etat.json, réglages), `paymind-relais` (`Paymind.exe`) et
`paymind-appli` (`Mindset.exe`, la fenêtre et sa page). Rust 1.99.1 (`rust\rust-toolchain.toml`).

Dans `rust\` :

```
cargo test --locked --workspace
cargo test --locked -p paymind-appli --features essai
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Le dossier prêt à zipper et le zip (dans PowerShell, depuis la racine du dépôt, chemins complets) :

```
& '.\outils\construire-rust.ps1' -Sortie '<dossier>' [-Cible '<target-dir de cargo>'] [-SansConstruire]
& '.\outils\zip-rust.ps1' -Dossier '<dossier>\Paymind' -Zip '<dossier>\Paymind.zip' -VerifierImports 'D:\Projets\Mindset\outils\verifier-imports.ps1'
```

Les essais qui ouvrent une vraie fenêtre tournent **seulement sur un bureau Windows à part** (`outils\sur-bureau.ps1`),
avec la construction d'essai (`cargo build -p paymind-appli -p paymind-relais --features paymind-appli/essai`), jamais
livrée :

- `outils\essai-fenetre.ps1 -Exe <Mindset.exe d'essai> -Resultats <dossier>` : la fenêtre seule (cachée au départ,
  second lancement, signal d'affichage, quitter.demande, WM_CLOSE, relance au changement de thème) ;
- `outils\essai-dans-mindset.ps1 -Exe <Mindset.exe d'essai> -Relais <Paymind.exe> -Resultats <dossier>` : Paymind
  lancée par son relais, accrochée par le code d'accrochage de Mindset, capturée, puis fermée par « Fermer » et
  « Tout quitter » ;
- `outils\captures-portefeuille.ps1 -Exe <Mindset.exe d'essai> -Relais <Paymind.exe> -Resultats <dossier> [-Theme <thème>]
  [-Abonnements <abonnements.json d'essai>] [-Vues 'accueil,fiche:a3,ajouter,chercher:mutuel,valider:m:netflix'] [-Prefixe <nom>]` :
  les captures du portefeuille dans un panneau de Mindset de 1180 x 760, vue par vue (aucune erreur de la page) ;
- `D:\Projets\Mindset\outils\tester-contrat.ps1 -AppliConstruite <dossier>\Paymind -EntreeApps outils\entree-apps-rust.json` :
  tout le contrat avec le vrai code de Mindset, sur l'exe livré (aussi lancé par les tests de GitHub).

Exemple (chemins complets : les scripts les lisent depuis le dossier du processus, pas celui de PowerShell) :
`& 'D:\Projets\Paymind\outils\sur-bureau.ps1' -Script 'D:\Projets\Paymind\outils\essai-dans-mindset.ps1' -Sortie 'D:\Projets\Paymind\_test\essai.txt' -Arguments "..."`.
L'entrée de Paymind pour la liste des applis de Mindset est dans `outils\entree-apps-rust.json`.
