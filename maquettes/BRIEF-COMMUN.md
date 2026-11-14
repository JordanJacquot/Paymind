# Brief commun : maquettes Paymind, 2e tour

## Ce que Jordan a dit du 1er tour
Les 3 premières maquettes (proposition-a/b/c.html dans ce dossier : inventaire de jeu de rôle, bibliothèque de console, cockpit) ne lui plaisent pas :
- l'ORGANISATION des écrans ne lui va pas ;
- ce n'est PAS ASSEZ BEAU NI MODERNE (ça fait maquette, pas une vraie appli soignée) ;
- il veut les LOGOS OFFICIELS des marques.
Il veut GARDER le style de la suite, mais en mieux fait. Regarde les captures capture-a.png, capture-b.png, capture-c.png pour voir ce qu'il faut dépasser : trop de petits textes, trop de cadres partout, tout au même niveau.

## L'appli
Paymind : noter à la main ses abonnements et prélèvements, savoir combien on paie par mois et par an, et quand chacun se renouvelle. Fiche : D:\Projets\Paymind\CADRAGE.md. Utilisateurs PAS à l'aise avec un PC : peu de mots, en français simple, gros éléments, une chose importante par zone. Aucun tiret long (ni — ni –) dans les textes. Couleur de l'appli : vert #22E07A.
Trois choses à montrer : 1) le bilan (total par mois et par an, ce qui arrive bientôt, la liste) ; 2) ajouter un abonnement depuis une grande liste toute prête (logos, prix habituel déjà rempli qu'on corrige, recherche, catégories ; plus « Un abonnement à moi ») ; 3) la fiche d'un abonnement (prix, rythme semaine/mois/trimestre/an, prochain paiement, renouvellement automatique oui/non, rappel 7 jours avant, mettre en pause, supprimer).

## Le style de la suite
Direction « console d'un futur lointain » : fond très sombre, bords de plasma lumineux, pages en module déplié. Pour voir le niveau de finition attendu, lis le prototype validé de la suite (Nevermind) avec l'outil Artifact, action "read", url https://claude.ai/artifact/TbSh5EJHtvb5vsQntFEHFw, et C:\Users\death\.claude\skills\nouvelle-appli\kit\ui\kit.css. Reprends sa palette, ses formes et sa police (Segoe UI ou celle du kit), en CSS recopié dans ta page.
Règles de Jordan : JAMAIS de lueur sur les chiffres ni les textes (textes nets ; la lueur pour les bords et petits détails). Un anneau seulement pour une vraie proportion. Aucune donnée que l'appli ne pourrait pas avoir.
« Moderne et soigné » veut dire : beaucoup d'air, une hiérarchie nette (un grand chiffre, le reste discret), des logos grands et propres, des coins et espacements réguliers, peu de cadres, des transitions douces (respecter prefers-reduced-motion).

## Les logos officiels
Dans D:\Projets\Paymind\maquettes\logos\ (un .svg par marque). Ceux de Simple Icons sont monochromes (un seul tracé) : colore-les avec la couleur officielle de la marque (ex. Netflix #E50914, Spotify #1DB954, Prime Video #1F2B3E ou #00A8E1, YouTube #FF0000, PlayStation #0070D1, Xbox #107C10, Orange #FF7900, Deezer #A238FF...) ou en blanc sur une pastille à la couleur de la marque. Ceux de Wikimedia (disneyplus, canalplus, worldofwarcraft, sfr, free, sosh, allianz, groupama, lemonde, microsoft365, google, axa, maif, bouyguestelecom) sont déjà en couleur. Recopie le SVG dans la page (en ligne), en le nettoyant si besoin (enlève les métadonnées, garde le viewBox). Dofus n'a pas de logo : une pastille avec « D » soignée, en attendant. Mutuelle, assurance voiture ou habitation sans marque : un pictogramme simple.
Données d'exemple (une dizaine) : Netflix 13,49 €/mois, Spotify 11,12 €/mois, Prime Video 6,99 €/mois, Disney+ 9,99 €/mois, World of Warcraft 12,99 €/mois, Dofus 5,99 €/mois, Free (box) 29,99 €/mois, Orange (mobile) 15,99 €/mois, AXA assurance voiture 540 € par an, MAIF assurance maison 14,50 €/mois, mutuelle 32 €/mois, Basic-Fit (pictogramme) 29,99 € toutes les 4 semaines ou par mois. Marquées discrètement « Exemple ». Prix au format français (13,49 €). Aujourd'hui = samedi 14 novembre 2026.

## La page : une VRAIE PETITE APPLI CLIQUABLE
Une seule fenêtre d'appli d'environ 1280x760, centrée, dans laquelle on clique : l'accueil, un clic sur un abonnement ouvre sa fiche, le bouton Ajouter ouvre le catalogue, la recherche filtre, ajouter depuis le catalogue ajoute vraiment à la liste et recalcule le total, la fiche modifie vraiment (prix, rythme) et le total suit. Retour en arrière visible. Au-dessus de la fenêtre, une ligne : « Proposition X : <nom>. Clique pour essayer. » Le lien #bilan, #ajouter, #fiche ouvre directement l'écran (sert aux captures).
Contraintes de publication (page web privée) : pas de <!doctype>, <html>, <head>, <body> ; commence par <title>Paymind proposition X</title>, puis <style>, puis le contenu ; tout le CSS, le JS et les SVG dans la page ; aucune ressource externe (pas de police Google non plus) ; fond sombre fixé par la page (color-scheme: dark) ; sur téléphone la fenêtre passe en une colonne sans déborder sur le côté. Taille de la page : moins de 1 Mo.

## Vérifier avant de rendre
Capture de chaque écran (#bilan, #ajouter, #fiche) et REGARDE-les (outil Read), corrige (texte coupé, chevauchement, logo déformé ou illisible sur le fond sombre, lueur sur un chiffre), recapture. Depuis l'outil Bash, avec Edge sans interface (aucune fenêtre) :
"/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" --headless=new --disable-gpu --hide-scrollbars --user-data-dir="D:\Projets\Paymind\maquettes\_edge-X" --window-size=1400,900 --screenshot="D:\Projets\Paymind\maquettes\capture-X-bilan.png" "file:///D:/Projets/Paymind/maquettes/proposition-X.html#bilan"
Supprime ton dossier _edge-X à la fin.

## Règles absolues (le PC de Jordan)
- Aucune fenêtre ne s'ouvre sur l'écran de Jordan : jamais powershell.exe, python, python3, py ni pip depuis Bash, même pas --version. Edge seulement avec --headless=new.
- Ne crée ou modifie QUE ta page proposition-X.html et tes captures capture-X-*.png dans D:\Projets\Paymind\maquettes\ (ne touche pas aux logos ni aux autres pages). Pas de git. Rien d'autre sur le PC.
- Avant chaque capture : dans l'outil PowerShell, `Get-Process bf6, dofus*, RocketLeague, FC*, FIFA*, 'League of Legends' -ErrorAction SilentlyContinue` (Dofus toléré ; un autre jeu : arrête et dis-le).

Rends : le chemin de ta page, les 3 captures, et en 5 lignes ce que montre ta proposition et ce que tu as corrigé après avoir regardé.
