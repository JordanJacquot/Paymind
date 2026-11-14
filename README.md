# Paymind

Paymind aide à savoir combien coûtent tous ses abonnements et contrats, et quand ils se renouvellent.
On les ajoute depuis une grande liste toute prête (ou à la main) ; Paymind fait le bilan du mois et de l'année et
prévient avant chaque paiement. Tout reste sur le PC : rien n'est envoyé sur Internet.

Paymind fait partie de la suite Mindset : elle s'installe, se met à jour et s'affiche dans Mindset.
Le cadrage est dans [CADRAGE.md](CADRAGE.md), la maquette retenue dans `maquettes\proposition-f.html` (le portefeuille de cartes).

Les logos des marques restent la propriété de leurs marques ; ils servent seulement à reconnaître un abonnement.

## Plan

- [ ] **1. Paymind apparaît dans Mindset** : une page vide s'ouvre dans l'onglet de Mindset, suit le thème et se ferme
  proprement. Vérifié par les essais automatiques (repris de Mindcare) et un essai dans un Mindset de test.
- [ ] **2. Le calcul des comptes** : chaque rythme (semaine, mois, trimestre, an) ramené au mois et à l'année, la date du
  prochain paiement qui avance seule après un renouvellement automatique, les totaux par catégorie, la liste « Bientôt » ;
  les abonnements gardés sur le PC avec une copie de secours. Vérifié par des tests sur les cas pièges (le 31 dans un
  mois de 30 jours, le 29 février, le passage d'année).
- [ ] **3. Le portefeuille** : l'accueil de la maquette F (cartes, total par mois et par an, « Bientôt ») ;
  « Ajoute ton premier abonnement » quand la liste est vide. Vérifié par des captures comparées à la maquette.
- [ ] **4. La liste toute prête** : environ 80 abonnements avec leur logo officiel et leur prix habituel (streaming,
  musique, jeux dont Dofus et World of Warcraft, opérateurs, banques et assurances dont Crédit Mutuel, sport, presse,
  logiciels), la recherche et « Un abonnement à moi ». Vérifié : ajouter Netflix fait apparaître sa carte et change le total.
- [ ] **5. La fiche** : prix, rythme, date, renouvellement automatique, pause, suppression ; le rappel au choix (jamais,
  le jour même, la veille par défaut, 3 jours avant, 7 jours avant).
- [ ] **6. Les rappels et la carte dans Mindset** : une bulle près de l'horloge au moment choisi (montrée au démarrage
  suivant si le PC était éteint) ; la carte de Mindset montre le total du mois et le prochain paiement.
- [ ] **7. Première version 0.1.0** : publiée avec l'accord de Jordan et ajoutée à la liste des applis de Mindset.

Les prix de la liste toute prête sont datés : ils sont remis à jour à chaque nouvelle version.
