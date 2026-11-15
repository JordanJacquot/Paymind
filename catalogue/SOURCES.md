# Paymind : sources du catalogue

Prix relevés le **15/11/2026** (prix TTC en France). Chaque prix est un **prix habituel** que l'utilisateur corrige ensuite.
`prix_a_verifier: true` veut dire : moyenne nationale, prix qui dépend du client (région, club, profil), ou prix
trouvé seulement dans un article ou un comparateur, sans confirmation sur la page officielle.

Les logos restent la propriété de leurs marques. Ils servent seulement à reconnaître un abonnement dans la liste.

## Fichier `catalogue.json`

- `format` : 1. `prix_releves_le` : date du relevé.
- `categories` : `id`, `nom`, `ordre`, `fond` et `fond_fin` (le dégradé d'une carte sans logo de cette catégorie, par
  exemple un abonnement « à moi » rangé là).
- `abonnements` : `id`, `nom`, `categorie`, `prix_centimes`, `rythme` (`semaine`, `mois`, `trimestre`, `an`), `offre`,
  `logo` (fichier dans `logos/`, vide s'il n'y en a pas), `logo_ratio` (largeur / hauteur du logo), `affichage`,
  `fond` et `fond_fin` (dégradé de la carte, comme la maquette), `texte` (couleur du texte), `prix_a_verifier`,
  et pour Dofus `pastille` (l'oeuf, pour les petites pastilles rondes).
- `affichage` :
  - `blanc` : logo d'une seule couleur, à peindre en blanc sur la carte (icônes Simple Icons) ;
  - `plaque` : logo en couleurs, posé sur une petite plaque blanche ;
  - `couleur` : logo en couleurs posé directement sur la carte (Canal+, World of Warcraft, AXA, Dofus, Orange) ;
  - `texte` : pas de logo trouvé, le nom s'écrit sur une plaque blanche dans la couleur `fond` ;
  - `picto:coeur`, `picto:voiture`, `picto:maison` : abonnement sans marque, pictogramme simple (comme la maquette).
- Les SVG ont été recadrés : leur `viewBox` suit le contour réel du dessin (sans largeur ni hauteur fixes), donc ils
  remplissent la place qu'on leur donne. Les PNG et JPG ont perdu leurs marges vides.

## Prix

Sources : abréviations utilisées plus bas.
- **Off.** = page officielle de la marque.
- **Art.** = article ou comparateur récent (2026), adresse donnée.
- **Moy.** = moyenne nationale (pas de prix par marque publié).
- **Conn.** = connaissance, non vérifiée en ligne.

### Films et séries
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Netflix | 7,99 € / mois | Standard avec pub (sans pub : 14,99 €) | Off. help.netflix.com/fr/node/24926 | non |
| Amazon Prime | 6,99 € / mois | Au mois (69,90 € par an) | Art. selectra.info/telecom/tv/svod/prime-video | oui |
| Disney+ | 10,99 € / mois | Standard sans pub (avec pub : 6,99 €) | Art. selectra.info/telecom/tv/svod/disney-plus (23/09/2026) | oui |
| Canal+ | 24,99 € / mois | Sans engagement | Art. selectra.info/telecom/tv/svod/canal-plus (06/11/2026) | oui |
| Apple TV | 9,99 € / mois | Abonnement (ancien Apple TV+) | Art. journaldugeek.com 28/08/2026 (hausse aux États-Unis, pas encore en France) | oui |
| HBO Max | 10,99 € / mois | Standard sans pub | Art. iphonesoft.fr 08/06/2026, kulturegeek.fr | oui |
| Paramount+ | 7,99 € / mois | Standard | Art. selectra.info/telecom/tv/svod/paramount-plus (08/2026) | oui |
| Crunchyroll | 6,99 € / mois | Fan | Art. animotaku.fr (hausse du 02/02/2026) | oui |
| YouTube Premium | 12,99 € / mois | Individuel | Art. kulturegeek.fr 21/08/2026 | oui |
| ADN | 6,99 € / mois | 1 mois | Art. spliiit.com (page officielle non lue) | oui |

### Musique
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Spotify | 12,14 € / mois | Premium Personnel | Off. spotify.com/fr/premium | non |
| Deezer | 11,99 € / mois | Premium | Off. deezer.com/fr/offers | non |
| Apple Music | 11,99 € / mois | Individuel | Art. iphonesoft.fr 17/07/2026 (hausse, ancien prix 10,99) | oui |
| YouTube Music | 10,99 € / mois | Premium | Art. siecledigital.fr 09/02/2026 | oui |
| Amazon Music Unlimited | 10,99 € / mois | Sans Prime (avec Prime : 9,99) | Art. universfreebox, alloforfait (une source dit 12,14) | oui |
| Qobuz | 14,99 € / mois | Studio Solo | Off. try.qobuz.com (page d'une offre de 2025) | oui |

### Jeux vidéo
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Xbox Game Pass | 20,99 € / mois | Ultimate | Off. news.xbox.com/fr-fr 21/04/2026 | non |
| PlayStation Plus | 9,99 € / mois | Essential, 1 mois (71,99 € par an) | Art. iphonesoft.fr 18/05/2026 | oui |
| Nintendo Switch Online | 19,99 € / an | Individuel | Art. recharge.fr | oui |
| World of Warcraft | 12,99 € / mois | 1 mois | Art. recharge.fr 2026, breakflip | oui |
| Dofus | 5,99 € / mois | 1 mois | Art. dofus.jeuxonline.info/actualite/64729 | oui |
| Final Fantasy XIV | 10,99 € / mois | Standard (Entry : 8,99) | Conn., aucun prix en euros trouvé en ligne | oui |
| EA Play | 5,99 € / mois | 1 mois | Art. next-stage.fr, phonandroid.com | oui |
| Ubisoft+ | 17,99 € / mois | Premium (Classics : 7,99) | Art. tech-insider.org 2026 | oui |
| GeForce Now | 10,99 € / mois | Performance | Art. tech-insider.org | oui |
| Discord Nitro | 9,99 € / mois | Nitro | Art. eesel.ai 07/2026 | oui |
| Fortnite Crew | 11,99 € / mois | 1 mois | Art. xboxygen.com 09/2026 | oui |

### Téléphone et internet
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Orange | 42,99 € / mois | Livebox Classic fibre après la 1re année (29,99 avant) | Art. edcom.fr (prix du 04/11/2026) | oui |
| SFR | 27,99 € / mois | Box fibre Starter | Art. selectra.info, parrainduweb.fr 06/10/2026 (hausse annoncée pour les clients) | oui |
| Bouygues Telecom | 37,99 € / mois | Bbox Fit après la 1re année | Art. selectra.info 27/07/2026 | oui |
| Free | 39,99 € / mois | Freebox Pop après la 1re année | Art. selectra.info/telecom/fournisseurs/free/fibre | non |
| Free Mobile | 19,99 € / mois | Forfait Free 5G+ 350 Go | Art. selectra.info | non |
| Sosh | 9,99 € / mois | Forfait 20 Go | Art. jechange.fr/telecom/sosh | non |
| RED by SFR | 9,99 € / mois | Forfait 40 Go | Art. edcom.fr 11/2026 (hausses annoncées) | oui |
| B&You | 9,99 € / mois | Forfait 20 Go | Art. bbox-mag.fr 13/11/2026 (une autre source dit 10,99) | oui |

### Banque (toutes à vérifier : le prix dépend de la région et des options)
Valeur = coût annuel d'un usage courant (tenue de compte, carte à débit immédiat, 2 retraits par mois ailleurs,
petit découvert) divisé par 12. Source : moneyvox.fr/tarif-bancaire/banques-moins-cheres.php (1er novembre 2026).
| Banque | Prix | Remarque |
|---|---|---|
| Crédit Mutuel | 15,03 € / mois | 170 à 190 € par an selon la région |
| Crédit Agricole | 13,34 € / mois | 127 à 194 € par an selon la caisse |
| BNP Paribas | 14,33 € / mois | 171,92 € par an |
| Société Générale | 13,99 € / mois | 167,92 € par an |
| LCL | 14,74 € / mois | 176,90 € par an |
| Caisse d'Épargne | 14,65 € / mois | 160 à 192 € par an selon la région |
| La Banque Postale | 13,03 € / mois | 156,40 € par an |
| CIC | 14,92 € / mois | 179 € par an |
| Banque Populaire | 16,26 € / mois | 183 à 208 € par an selon la région |
| BoursoBank | 9,90 € / mois | Carte Metal (Conn.). L'offre de base est gratuite : le prix doit rester modifiable |
| Revolut | 3,99 € / mois | Revolut Plus (Conn.) |

### Assurances et santé (toutes à vérifier : moyennes nationales)
Aucun prix moyen par assureur n'est publié : la moyenne nationale est reprise pour chaque marque.
| Abonnement | Prix | Source |
|---|---|---|
| Assurance voiture (AXA, MACIF, MAAF, Groupama, Matmut, GMF, MMA, et la carte générique) | 700 € / an | Moy. lelynx.fr/assurance-auto/tarif (663 €), meilleurtaux.com (plus de 800 €), selectra.info |
| Assurance habitation, locataire (MAIF, Allianz, et la carte générique) | 144 € / an | Moy. lecomparateurassurance.com, baromètre octobre 2026. Propriétaire d'une maison : environ 350 € par an (moneyvox.fr) |
| Mutuelle santé, une personne (Alan, et la carte générique) | 75 € / mois | Moy. lecomparateurassurance.com, baromètre juin 2026 (57 € avant 35 ans, 97 € tous âges) |

### Sport
Les salles prélèvent souvent toutes les 4 semaines (13 fois par an). Le catalogue n'a pas ce rythme : le prix a été
ramené au mois (prix des 4 semaines x 13 / 12), et le prix réel est écrit dans l'offre.
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Basic-Fit | 27,07 € / mois | Comfort, 24,99 € toutes les 4 semaines | Art. parrainduweb.fr, basic-fit.com | non |
| Fitness Park | 32,50 € / mois | Classic, environ 30 € toutes les 4 semaines | Art. parrainduweb.fr (prix selon le club) | oui |
| L'Orange Bleue | 35,10 € / mois | Environ 32,40 € toutes les 4 semaines (29,90 à 34,90) | Art. resilier.fr, masalledesport.com | oui |
| Neoness | 19,99 € / mois | Abonnement de base (15 à 20 €) | Art. vitepromo.com | oui |
| Strava | 59,99 € / an | À l'année | Art. parrainduweb.fr, reference-trail.fr | non |

### Presse et lecture
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Le Monde | 11,99 € / mois | Numérique, après la promotion | Art. subger.com (10,99 à 12,99 ailleurs) | oui |
| Le Figaro | 17,99 € / mois | Premium numérique, après la promotion | Art. dealabs (une autre source dit 14,90) | oui |
| L'Équipe | 12,99 € / mois | Premium numérique | Art. spliiit.com | oui |
| Le Parisien | 7,99 € / mois | Numérique | Art. fr.igraal.com | oui |
| Mediapart | 12,00 € / mois | Accès complet (120 € par an) | Art. spliiit.com | oui |
| Kindle Unlimited | 9,99 € / mois | 1 mois (nouveau nom : « Abonnement Kindle ») | Art. blog-nouvelles-technologies.fr | oui |
| Audible | 9,95 € / mois | Premium Plus (Standard : 5,99) | Off. audible.fr/ep/memberbenefits | non |

### Logiciels et stockage
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Microsoft 365 | 99 € / an | Personnel (Famille : 129 €) | Art. it-connect.fr | non |
| Google One | 1,99 € / mois | 100 Go | Art. spliiit.com | non |
| iCloud+ | 0,99 € / mois | 50 Go (200 Go : 2,99) | Off. support.apple.com/fr-fr/HT5879 | non |
| Dropbox | 11,99 € / mois | Plus, 2 To | Art. quechoisir.org | oui |
| Adobe Creative Cloud | 23,87 € / mois | Photographie 1 To | Art. phototrend.fr 10/2026 | oui |
| ChatGPT | 22,99 € / mois | Plus | Art. jechange.fr 10/2026 | non |
| NordVPN | 12,99 € / mois | Basique sans engagement (environ 3,09 à 3,49 € par mois sur 2 ans) | Art. monpetitforfait.com | oui |
| Canva | 12,00 € / mois | Pro (110 € par an) | Art. siecledigital.fr 30/03/2026 | oui |
| Duolingo | 110,99 € / an | Super | Art. spliiit.com (94,99 à 122,99 selon les sources) | oui |

### Courses et livraison
| Abonnement | Prix | Offre | Source | À vérifier |
|---|---|---|---|---|
| Uber One | 5,99 € / mois | 1 mois (59,99 € par an) | Off. uber.com/fr/fr/u/uber-one | non |
| Deliveroo Plus | 2,99 € / mois | Silver (Gold : 5,99) | Art. parrainduweb.fr | oui |
| Cdiscount à volonté | 29 € / an | Livraison gratuite dès 10 € | Art. phonandroid.com | oui |

### Maison et énergie (toutes à vérifier : dépend du logement et de la consommation)
| Abonnement | Prix | Offre | Source |
|---|---|---|---|
| EDF | 106 € / mois | Tarif bleu, foyer de 2 personnes (une personne seule : 69 €) | Moy. hellowatt.fr, fournisseur-energie.com (2026) |
| Engie | 149 € / mois | Gaz avec chauffage (11 200 kWh par an) | Moy. selectra.info, été 2026 |
| TotalEnergies | 100 € / mois | Électricité, foyer de 2 personnes | Moy. EDF reprise |
| Octopus Energy | 100 € / mois | Électricité, foyer de 2 personnes | Moy. EDF reprise |
| Veolia (eau) | 43,40 € / mois | Eau et assainissement, 120 m³ par an | Moy. kelwatt.fr, eau.selectra.info (Eaufrance) |

## Logos

Récupérés le 15/11/2026. « SI » = Simple Icons (licence CC0), via cdn.jsdelivr.net/npm/simple-icons.
« WC » = Wikimedia Commons (licence indiquée sur la page du fichier). Les SVG ont été recadrés (viewBox), sans
autre changement, sauf `orange.svg` (couleur orange ajoutée au carré).

| Fichier | Source | Licence |
|---|---|---|
| netflix, amazonprime, appletv, paramountplus, crunchyroll, youtubepremium (icône YouTube), spotify, deezer, applemusic, youtubemusic, xboxgamepass (icône Xbox), playstationplus (icône PlayStation), nintendoswitchonline (icône Nintendo Switch), eaplay (icône EA), ubisoftplus (icône Ubisoft), orange, revolut, strava, lequipe, audible, icloud, dropbox, adobe (icône Creative Cloud), chatgpt (icône OpenAI), nordvpn, canva, duolingo, uberone (icône Uber), deliveroo | SI (repris de `maquettes/logos`) | CC0 |
| hbomax, amazonmusic, discordnitro (icône Discord), fortnitecrew (icône Fortnite) | SI | CC0 |
| disneyplus, canalplus, worldofwarcraft, sfr, free, freemobile (logo Free), sosh, bouyguestelecom, creditmutuel, axa, maif, groupama, allianz, lemonde, microsoft365 | WC (repris de `maquettes/logos`) | domaine public (logo simple) selon Commons |
| dofus.png, dofus-oeuf.png | fr.wikipedia (repris de `maquettes/logos`) | logo du jeu, propriété d'Ankama |
| adn.png | WC « ADN Logo 2016.png » | domaine public |
| qobuz.svg | WC « Qobuz logo light.svg » | domaine public |
| ffxiv.svg | WC « Final Fantasy XIV Online logo.svg » | domaine public |
| geforcenow.png | WC « GeForce Now logo (2022).svg » (vignette 500 px) | domaine public |
| redbysfr.svg | fr.wikipedia « Logo_Red_By_SFR_2016.svg » | logo simple hébergé sur fr.wikipedia |
| byou.jpg | WC « B&YOU logo.jpeg » (vignette 500 px) | domaine public |
| creditagricole.svg | WC « Crédit Agricole 2020 logo.svg » | domaine public |
| bnpparibas.svg | WC « BNP Paribas.svg » (le symbole seul) | domaine public |
| societegenerale.svg | WC « Logo-SG-Société-Générale.svg » | domaine public |
| lcl.svg | WC « LCL banque logo.svg » | domaine public |
| labanquepostale.svg | WC « Logo La Banque postale 2022.svg » | domaine public |
| cic.svg | WC « Crédit Industriel et Commercial Logo.svg » | domaine public |
| banquepopulaire.png | WC « Banque Populaire landscape 2018 logo.svg » (vignette 500 px) | domaine public |
| maaf.jpg | WC « Logo de MAAF.jpg » | CC BY-SA 4.0 |
| alan.svg | WC « Alan-logo-green.svg » | CC BY-SA 4.0 |
| basicfit.png | WC « Basic-Fit logo.png » | CC BY-SA 4.0 |
| lefigaro.png | WC « Le Figaro.svg » (vignette 500 px) | domaine public |
| leparisien.svg | WC « Le Parisien logo.svg » | domaine public |
| mediapart.svg | WC « Mediapart wordmark.svg » | domaine public |
| kindleunlimited.png | WC « Amazon Kindle 2024.svg » (vignette 500 px) | domaine public |
| googleone.svg | WC « Google One logo.svg » | domaine public |
| cdiscount.png | WC « Logo CDiscount 2024.png » (vignette 500 px) | CC0 |
| edf.png | WC « Électricité de France logo.svg » (vignette 500 px) | domaine public |
| engie.svg | WC « Logo-engie.svg » | domaine public |
| totalenergies.svg | WC « TotalEnergies wordmark (2021-present).svg » | domaine public |
| veolia.svg | WC « Veolia logo.svg » | domaine public |
| octopus.svg | WC « Octopus Energy logo (2019).svg » | domaine public |

Les fichiers marqués « vignette 500 px » sont des PNG : Wikimedia a refusé de servir l'original (trop de demandes)
et demande d'utiliser ses vignettes.

**Sans logo** (plaque de texte aux couleurs de la marque) : Caisse d'Épargne, BoursoBank, MACIF, Matmut, GMF, MMA,
Fitness Park, L'Orange Bleue, Neoness. Aucun fichier sûr trouvé sur Commons (ou recherche bloquée par Wikimedia).
**Sans marque** (pictogramme) : Mutuelle santé, Assurance voiture, Assurance habitation.
