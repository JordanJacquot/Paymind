// Le portefeuille de Paymind (étapes 3 et 4) : l'accueil de la maquette F validée par Jordan
// (maquettes\proposition-f.html), et son écran « Ajouter un abonnement » (la liste toute prête). Le kit (kit.js)
// l'appelle à la place de ses tuiles (bloc accueil.perso de l'état, voir « perso » en tête de kit.js) ; il garde
// l'en-tête, le bas, les flèches, A et B (Entrée et Échap), les sons et la page « Bientôt ».
//
// Rien n'est calculé ici : Rust (src\portefeuille.rs, src\ajout.rs) envoie les totaux, les dates et les montants
// déjà écrits en français, les logos et leurs tailles. Tout texte reçu est échappé (esc du kit), chaque couleur,
// chaque id et chaque taille vérifiés, chaque image passée par urlSure du kit. Ce fichier ne parle jamais à Rust
// lui-même : les gestes passent par le kit (action), et Rust vérifie tout de nouveau.
//
// - une carte (ou une pastille de « Bientôt ») ouvre sa fiche : la carte s'agrandit, la fiche apparaît dessous ;
//   Retour, Échap ou B la referment. Depuis l'étape 5, la fiche se change (src\fiche.rs) : le prix (Entrée, ou en
//   quittant le champ), le rythme, la date (un petit calendrier d'un mois, gros jours à cliquer, à la place de la
//   colonne de droite : pas le champ date du navigateur), le renouvellement automatique, le rappel, une note, le nom
//   d'un abonnement à moi, la pause, « C'est renouvelé » et la suppression (« Supprimer Netflix ? » puis « Oui,
//   supprimer » : jamais au premier clic). Chaque geste part vers Rust par le kit (action 'fiche-...', valeur
//   { id, ... }) ; Rust vérifie, enregistre et renvoie le portefeuille : la fiche ouverte est redessinée sur place ;
//   les flèches restent dans la fiche ;
// - une carte de la liste toute prête montre le logo officiel de sa marque (blanc, en couleurs, sur sa plaque, son
//   nom sur une plaque, ou un pictogramme) ; une carte « à moi », une plaque avec ses initiales ;
// - « Ajouter » montre la liste toute prête, par catégorie, avec une recherche (sans accents ni majuscules, dans la
//   clé préparée par Rust) et « Un abonnement à moi ». Choisir un abonnement l'ajoute : Rust l'enregistre, renvoie
//   le portefeuille et l'ordre { kit: 'accueil', ajoute: id } ; on revient alors sur la nouvelle carte ;
// - la liste vide : un seul grand bouton, « Ajoute ton premier abonnement ».
(() => {
  'use strict';
  const reduit = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  const couleurSure = (c, defaut) => (typeof c === 'string' && /^#[0-9a-f]{6}$/i.test(c) ? c : defaut);
  const idSur = (t) => typeof t === 'string' && /^[\w:.-]{1,60}$/.test(t);
  const nombreSur = (n, max) => (typeof n === 'number' && Number.isFinite(n) && n > 0 && n <= max ? n : 0);
  const svg = (corps, w = 2) => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${corps}</svg>`;
  const PLUS = svg('<path d="M12 5v14M5 12h14"/>', 2.2);
  const RETOUR = svg('<path d="M15 5l-7 7 7 7"/>');
  const LOUPE = svg('<circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/>');
  const COCHE = svg('<path d="M5 12l5 5L20 7"/>', 3);
  const AGENDA = svg('<rect x="3.5" y="5" width="17" height="15" rx="2.5"/><path d="M3.5 10h17M8 3v4M16 3v4"/>');
  const POUBELLE = svg('<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3"/>');
  const PAUSE = '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1"/><rect x="14" y="5" width="4" height="14" rx="1"/></svg>';
  const LECTURE = '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z"/></svg>';
  const GAUCHE = svg('<path d="M15 5l-7 7 7 7"/>', 2.4);
  const DROITE = svg('<path d="M9 5l7 7-7 7"/>', 2.4);
  // Les pictogrammes des abonnements sans marque (catalogue\_apercu.html)
  const PICTOS = {
    coeur: svg('<path d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"/><path d="M12 10v5M9.5 12.5h5"/>'),
    voiture: svg('<path d="M5 16V11l2-5h10l2 5v5"/><path d="M3 16h18v3H3z"/><circle cx="7.5" cy="19" r="1.5"/><circle cx="16.5" cy="19" r="1.5"/><path d="M5 11h14"/>'),
    maison: svg('<path d="M3 11l9-7 9 7"/><path d="M5 10v10h14V10"/><path d="M10 20v-6h4v6"/>'),
  };
  const FORMES = new Set(['blanc', 'couleur', 'plaque']);

  let P = null;          // le dernier portefeuille reçu
  let O = null;          // les outils du kit (esc, urlSure, son...)
  let hote = null;       // #ecran
  let ouverte = null;    // l'id de la carte dont la fiche est ouverte
  let retourK = null;    // l'élément à reprendre en refermant la fiche
  let dernierK = null;   // le dernier élément choisi sur l'accueil (pour y revenir depuis une page)
  const cartes = new Map();
  // L'écran « Ajouter » : la vue montrée, ce qui est tapé dans la recherche, « Un abonnement à moi » ouvert et ses
  // champs (gardés quand l'état revient), la carte à montrer après un ajout, l'ajout en cours (un double clic
  // n'ajoute pas deux fois)
  let vue = 'portefeuille';
  let cherche = '';
  let moiOuvert = false;
  let moi = { nom: '', prix: '', rythme: 'mois', categorie: 'moi' };
  let aMontrer = null;
  let envoiJusqua = 0;
  // La fiche : le calendrier ouvert ({ id, a, m } : l'année et le mois montrés, 0 à 11) et la fiche où « Supprimer
  // Netflix ? » est demandé (gardés quand l'état revient, oubliés en refermant la fiche)
  let cal = null;
  let confirme = null;

  const esc = (t) => O.esc(t);
  const styleCouleurs = (x) => `--c1:${couleurSure(x.c1, '#5A5A6E')};--c2:${couleurSure(x.c2, '#17171F')}`;

  // ---------------------------------------------------------------- les logos
  // Le logo d'une carte (comme la maquette F), ou la plaque de ses initiales
  function logoCarte(x) {
    const l = x.logo && typeof x.logo === 'object' ? x.logo : null, ini = esc(x.initiales || '?');
    if (l?.forme === 'picto' && PICTOS[l.picto]) return `<span class="c-logo c-picto">${PICTOS[l.picto]}</span>`;
    if (l?.forme === 'texte' && typeof l.texte === 'string') return `<span class="c-logo c-plaque c-nomlogo"><b style="color:${couleurSure(l.couleur, '#333333')}">${esc(l.texte)}</b></span>`;
    const u = l && FORMES.has(l.forme) ? O.urlSure(l.url || '') : '';
    if (u) {
      const lg = nombreSur(l.l, 70), ht = nombreSur(l.h, 30);
      const taille = lg && ht ? ` style="width:${lg}cqw;height:${ht}cqw"` : '';
      return `<span class="c-logo lg-${l.forme}"><img src="${u}" alt="" data-logo="${ini}"${taille}></span>`;
    }
    return `<span class="c-logo c-plaque"><b>${ini}</b></span>`;
  }

  // Le rond d'une pastille de « Bientôt » : le logo (s'il tient dans un rond), un pictogramme, ou les initiales
  function rondHTML(x) {
    const l = x.logo && typeof x.logo === 'object' ? x.logo : null;
    if (l?.forme === 'picto' && PICTOS[l.picto]) return `<span class="rond-c r-picto" style="${styleCouleurs(x)}">${PICTOS[l.picto]}</span>`;
    const u = l && FORMES.has(l.forme) ? O.urlSure(l.url || '') : '';
    if (u) return `<span class="rond-c r-${l.forme}" style="${styleCouleurs(x)}"><img src="${u}" alt="" data-logo="${esc(x.initiales || '?')}"></span>`;
    return `<span class="rond-c" style="${styleCouleurs(x)}"><i>${esc(x.initiales || '?')}</i></span>`;
  }
  const rondAvecLogo = (x) => !!(x.logo && FORMES.has(x.logo.forme) && O.urlSure(x.logo.url || ''));

  // Un logo qui ne se charge pas : la plaque des initiales à sa place
  function secoursLogos(racine) {
    racine.querySelectorAll('img[data-logo]').forEach((img) => img.addEventListener('error', () => {
      const parent = img.parentElement, carte = parent.classList.contains('c-logo'), plaque = document.createElement(carte ? 'b' : 'i');
      plaque.textContent = img.dataset.logo || '?';
      parent.className = carte ? 'c-logo c-plaque' : 'rond-c';
      parent.replaceChildren(plaque);
    }, { once: true }));
  }

  // ---------------------------------------------------------------- le portefeuille
  function carteHTML(x, grande) {
    const st = x.statut === 'pause' || x.statut === 'renouveler' ? ` ${x.statut}` : '';
    const etq = x.badge ? `<span class="c-badge${st}">${esc(x.badge)}</span>` : x.etiquette ? `<span>${esc(x.etiquette)}</span>` : '';
    const dedans = `<span class="c-haut">${logoCarte(x)}<span class="c-etq">${etq}</span></span>
      <span class="c-bas"><span class="c-gauche"><b class="c-nom">${esc(x.nom)}</b><span class="c-proch">${esc(x.ligne)}</span></span><span class="c-prix"><small>${esc(x.rythme)}</small>${esc(x.prix)}</span></span>
      <i class="c-reflet"></i>`;
    if (grande) return `<div class="carte grande${st}" style="${styleCouleurs(x)}" aria-hidden="true">${dedans}</div>`;
    const aide = [x.nom, `${x.prix} ${x.rythme}`, x.badge, x.ligne].filter(Boolean).join(', ');
    return `<button type="button" class="carte f${st}" data-perso="carte" data-id="${esc(x.id)}" data-k="c:${esc(x.id)}" style="${styleCouleurs(x)}" aria-label="${esc(aide)}">${dedans}</button>`;
  }

  // (« Ajouter » montre la liste toute prête, dessinée ici : data-perso, pas une page du kit)
  const ajouterHTML = (p, grand) => (p.catalogue
    ? `<button type="button" class="ajouter${grand ? ' grand' : ''} f" data-perso="ajouter" data-k="ajouter">${PLUS}${grand ? `<b>${esc(p.titre)}</b>` : 'Ajouter'}</button>`
    : '');

  function videHTML(p) {
    return `<div class="pf-vide"><div class="pf-jour">${esc(p.jour)}</div>${ajouterHTML(p, true)}
      <p>${esc(p.texte)}</p><p class="pf-note">${esc(p.note)}</p></div>`;
  }

  function pleinHTML(p) {
    const t = p.total || {}, b = p.bientot || {};
    const groupes = (p.groupes || []).map((g) => {
      const c = (g.cartes || []).filter((x) => idSur(x.id)).map((x) => { cartes.set(x.id, x); return carteHTML(x); }).join('');
      return c ? `<section class="groupe-c"><h2>${esc(g.nom)}${g.total ? `<span>${esc(g.total)}</span>` : ''}</h2><div class="c-rang">${c}</div></section>` : '';
    }).join('');
    // (une pastille par abonnement : son rond, sa date, son nom quand le rond n'a pas de logo)
    const pastilles = (b.elements || []).filter((x) => idSur(x.id) && cartes.has(x.id)).map((x) =>
      `<button type="button" class="pas f" data-perso="carte" data-id="${esc(x.id)}" data-k="b:${esc(x.id)}" title="${esc(x.aide)}" aria-label="${esc(x.aide)}">${rondHTML(x)}<b>${esc(x.date)}</b>${rondAvecLogo(x) ? '' : `<span class="pas-nom">${esc(x.nom)}</span>`}</button>`).join('');
    const tout = b.tout && idSur(b.tout.id) ? `<button type="button" class="pas tout f" data-action="${esc(b.tout.id)}" data-k="b:tout">${esc(b.tout.texte)}</button>` : '';
    return `<div class="pf-in">
      <div class="bilan">
        <div class="total"><div class="pf-jour">${esc(p.jour)}</div><div class="gros">${esc(t.mois)}<small>${esc(t.par)}</small></div>
          <div class="an">soit <b>${esc(t.an)}</b> par an</div>${t.compte ? `<div class="compte">${esc(t.compte)}</div>` : ''}</div>
        <div class="bientot"><span>${esc(b.titre || 'Bientôt')}</span>${pastilles || `<p class="rien">${esc(b.vide)}</p>`}${tout}</div>
      </div>
      <div class="groupes">${groupes}<section class="groupe-c"><h2>&nbsp;</h2><div class="c-rang">${ajouterHTML(p)}</div></section></div>
    </div>`;
  }

  // ---------------------------------------------------------------- la fiche (modifiable depuis l'étape 5)
  const ISO = /^\d{4}-\d{2}-\d{2}$/;
  const deux = (n) => String(n).padStart(2, '0');
  const textesFiche = () => P.communs?.fiche || {};

  // Des pastilles à choisir (le rythme, le rappel) : une seule allumée
  function pastillesHTML(liste, choisi, quoi, petit, aide) {
    const b = (Array.isArray(liste) ? liste : []).filter((o) => Array.isArray(o) && idSur(o[0])).map((o) =>
      `<button type="button" class="f${o[0] === choisi ? ' oui' : ''}" role="radio" aria-checked="${o[0] === choisi}" data-perso="${quoi}" data-v="${esc(o[0])}" data-k="${quoi}:${esc(o[0])}">${esc(o[1])}</button>`).join('');
    return `<div class="choix-f${petit ? ' petit' : ''}" role="radiogroup" aria-label="${esc(aide)}">${b}</div>`;
  }

  // La colonne de droite : la date (un bouton qui ouvre le calendrier), « C'est renouvelé », le renouvellement
  // automatique, le rappel, puis la pause et la suppression (ou la question « Supprimer Netflix ? »)
  function droiteHTML(x) {
    const f = x.fiche || {}, t = textesFiche(), passee = x.statut === 'renouveler';
    const dateTexte = `${esc(f.date)}${f.quand ? `<small>${esc(f.quand)}</small>` : ''}`;
    const date = ISO.test(f.date_iso || '')
      ? `<button type="button" class="date-b f${passee ? ' passee' : ''}" data-perso="f-date" data-k="f-date" aria-label="${esc(`${t.changer_date} : ${f.date}`)}" title="${esc(t.changer_date)}">${AGENDA}<span>${dateTexte}</span></button>`
      : `<div class="date">${dateTexte}</div>`;
    const r = f.renouvele && typeof f.renouvele === 'object' ? f.renouvele : null;
    const renouv = r ? `<div class="renouv"><button type="button" class="btn-f plein f" data-perso="f-renouvele" data-k="f-renouvele">${COCHE}${esc(r.texte)}</button><small>${esc(r.aide)}</small></div>` : '';
    const bas = confirme === x.id
      ? `<div class="f-suppr" role="alert"><p><b>${esc(f.question)}</b><small>${esc(t.suppr_aide)}</small></p>
          <div class="f-btns"><button type="button" class="btn-f danger sur f" data-perso="f-suppr-oui" data-k="f-suppr-oui">${POUBELLE}${esc(t.suppr_oui)}</button><button type="button" class="btn-f f" data-perso="f-suppr-non" data-k="f-suppr-non">${esc(t.suppr_non)}</button></div></div>`
      : `<div class="f-btns"><button type="button" class="btn-f f" data-perso="f-pause" data-k="f-pause">${f.pause ? LECTURE : PAUSE}${esc(f.pause_texte)}</button><button type="button" class="btn-f danger f" data-perso="f-suppr" data-k="f-suppr">${POUBELLE}${esc(t.supprimer)}</button></div>`;
    return `<div class="rangf"><div class="txt">${esc(f.titre_date)}${f.etat ? `<small>${esc(f.etat)}</small>` : ''}</div>${date}</div>
      ${renouv}
      <div class="rangf"><div class="txt">${esc(t.auto)}<small>${esc(f.auto_texte)}</small></div><button type="button" class="bascule-f f${f.auto ? ' oui' : ''}" role="switch" aria-checked="${!!f.auto}" aria-label="${esc(t.auto)}" data-perso="f-auto" data-k="f-auto"></button></div>
      <div class="rangf rappel"><div class="txt">${esc(t.prevenir)}<small>${esc(f.rappel_texte)}</small></div></div>
      ${pastillesHTML(P.communs?.rappels, f.rappel, 'f-rappel', true, t.prevenir)}
      ${bas}`;
  }

  // Le calendrier d'un mois, à la place de la colonne de droite : les jours d'aujourd'hui à dans 3 ans se cliquent
  // (bornes données par Rust, qui vérifie de nouveau), le jour choisi et aujourd'hui sont marqués
  function calendrierHTML(x) {
    const f = x.fiche || {}, k = textesFiche().calendrier || {}, mois = Array.isArray(k.mois) ? k.mois : [];
    const min = ISO.test(k.min || '') ? k.min : '0000-00-00', max = ISO.test(k.max || '') ? k.max : '9999-12-31';
    const { a, m } = cal, cle = `${a}-${deux(m + 1)}`;
    const decale = (new Date(Date.UTC(a, m, 1)).getUTCDay() + 6) % 7, n = new Date(Date.UTC(a, m + 1, 0)).getUTCDate();
    let jours = '<span class="vide"></span>'.repeat(decale);
    for (let j = 1; j <= n; j += 1) {
      const d = `${cle}-${deux(j)}`, cls = `${d === f.date_iso ? ' oui' : ''}${d === min ? ' auj' : ''}`;
      jours += d >= min && d <= max
        ? `<button type="button" class="cal-d f${cls}" data-perso="f-jour" data-v="${d}" data-k="f-jour:${d}" aria-label="${j} ${esc(mois[m] || '')} ${a}">${j}</button>`
        : `<span class="cal-d non${cls}">${j}</span>`;
    }
    const fleche = (sens, ok, icone, aide) => (ok
      ? `<button type="button" class="cal-fl f" data-perso="f-mois" data-v="${sens}" data-k="f-mois:${sens < 0 ? 'avant' : 'apres'}" aria-label="${esc(aide)}" title="${esc(aide)}">${icone}</button>`
      : `<span class="cal-fl non">${icone}</span>`);
    return `<div class="cal" role="group" aria-label="${esc(k.titre)}">
      <div class="cal-titre">${esc(k.titre)}</div>
      <div class="cal-tete">${fleche(-1, cle > min.slice(0, 7), GAUCHE, k.avant)}<b>${esc(mois[m] || '')} ${a}</b>${fleche(1, cle < max.slice(0, 7), DROITE, k.apres)}</div>
      <div class="cal-grille">${(Array.isArray(k.jours) ? k.jours : []).map((j) => `<span class="cal-j">${esc(j)}</span>`).join('')}${jours}</div>
      <div class="f-btns"><button type="button" class="btn-f f" data-perso="f-cal-annuler" data-k="f-cal-annuler">${esc(k.annuler)}</button></div>
    </div>`;
  }

  // La fiche, sous la carte agrandie (les places de la fiche de la maquette F)
  function sceneHTML(x) {
    const f = x.fiche || {}, t = textesFiche();
    const nomMax = nombreSur(t.nom_max, 200) || 80, noteMax = nombreSur(t.note_max, 2000) || 200;
    const titre = f.nom_modifiable
      ? `<input class="f" id="fNom" data-perso="f-nom" data-k="f-nom" maxlength="${nomMax}" autocomplete="off" spellcheck="false" aria-label="${esc(t.nom)}" value="${esc(x.nom)}">`
      : esc(x.nom);
    return `<div class="scene-in">
      <button type="button" class="fermer-f f" data-perso="fermer" data-k="fermer">${RETOUR}Retour</button>
      ${carteHTML(x, true)}
      <div class="fiche">
        <div class="f-col">
          <div class="f-titre">${titre}<em>${esc(f.categorie)}</em></div>
          <span class="lbl">${esc(t.je_paie)}</span>
          <div class="prix-ligne"><label class="prix-champ"><input class="f" id="fPrix" data-perso="f-prix" data-k="f-prix" inputmode="decimal" maxlength="14" autocomplete="off" spellcheck="false" aria-label="${esc(t.je_paie)}" aria-describedby="fErr" value="${esc(f.prix_saisie)}"><span>€</span></label><p class="f-err" id="fErr" role="alert"></p></div>
          <span class="lbl">${esc(t.tous_les)}</span>
          ${pastillesHTML(P.communs?.rythmes, f.rythme, 'f-rythme', false, t.tous_les)}
          <div class="calc"><div>${esc(t.par_mois)}<b>${esc(f.mois)}</b></div><div>${esc(t.par_an)}<b>${esc(f.an)}</b></div></div>
          <label class="note-champ"><span class="lbl">${esc(t.note)}</span><input class="f" id="fNote" data-perso="f-note" data-k="f-note" maxlength="${noteMax}" autocomplete="off" placeholder="${esc(t.note_indice)}" value="${esc(f.note)}"></label>
        </div>
        <div class="f-col">${cal && cal.id === x.id ? calendrierHTML(x) : droiteHTML(x)}</div>
      </div></div>`;
  }

  // ---------------------------------------------------------------- « Ajouter un abonnement » (la liste toute prête)
  // Ce qui est tapé, comme la clé de Rust (src\catalogue.rs, pour_chercher) : minuscules, sans accents, chaque suite
  // de signes devient une espace. Chaque mot tapé doit se trouver dans la clé.
  const pourChercher = (t) => String(t || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, ' ').trim();
  const correspond = (cle, mots) => mots.every((m) => cle.includes(m));

  function miniHTML(x, c) {
    const aide = [`Ajouter ${x.nom}`, `${x.prix} ${x.rythme}`, x.offre].filter(Boolean).join(', ');
    const verif = x.verifier ? `<span class="m-verif" title="${esc(c.verifier_aide)}">${esc(c.verifier)}</span>` : '';
    return `<button type="button" class="mini f${x.deja > 0 ? ' a' : ''}" data-perso="modele" data-id="${esc(x.id)}" data-k="m:${esc(x.id)}" data-cle="${esc(x.cle || '')}" aria-label="${esc(aide)}">
      <span class="carte" style="${styleCouleurs(x)}"><span class="c-haut">${logoCarte(x)}<span class="c-etq">${verif}</span></span><i class="c-reflet"></i></span>
      <span class="deja">${COCHE}${esc(c.deja)}</span>
      <span class="leg"><b>${esc(x.nom)}</b><span class="px">${esc(x.prix)}</span></span>
      <span class="leg2"><span>${esc(x.offre)}</span><span>${esc(x.rythme)}</span></span></button>`;
  }

  function moiHTML(c) {
    const m = c.moi || {};
    if (!moiOuvert) return `<button type="button" class="perso-b f" data-perso="moi" data-k="a:moi">${PLUS}${esc(m.bouton)}</button>`;
    const choix = (liste, choisi, quoi) => (Array.isArray(liste) ? liste : []).filter((o) => Array.isArray(o) && idSur(o[0]))
      .map((o) => `<button type="button" class="f${o[0] === choisi ? ' oui' : ''}" data-perso="${quoi}" data-v="${esc(o[0])}" data-k="a:${quoi}:${esc(o[0])}" aria-pressed="${o[0] === choisi}">${esc(o[1])}</button>`).join('');
    const max = nombreSur(m.nom_max, 200) || 80;
    return `<div class="moi-f">
      <div class="moi-ligne">
        <label class="champ-m"><span>${esc(m.nom)}</span><input class="f" id="moiNom" data-perso="moi-champ" data-k="a:moi-nom" maxlength="${max}" autocomplete="off" spellcheck="false" placeholder="${esc(m.nom_indice)}" value="${esc(moi.nom)}"></label>
        <label class="champ-m prix"><span>${esc(m.prix)}</span><span class="avec-euro"><input class="f" id="moiPrix" data-perso="moi-champ" data-k="a:moi-prix" maxlength="14" inputmode="decimal" autocomplete="off" placeholder="${esc(m.prix_indice)}" value="${esc(moi.prix)}"><em>€</em></span></label>
      </div>
      <span class="lbl">${esc(m.rythme)}</span><div class="choix-m">${choix(m.rythmes, moi.rythme, 'moi-rythme')}</div>
      <span class="lbl">${esc(m.categorie)}</span><div class="choix-m cats">${choix(m.categories, moi.categorie, 'moi-cat')}</div>
      <p class="moi-err" id="moiErr" role="alert"></p>
      <div class="moi-btns"><button type="button" class="btn-m plein f" data-perso="moi-ok" data-k="a:moi-ok">${PLUS}${esc(m.valider)}</button><button type="button" class="btn-m f" data-perso="moi-annuler" data-k="a:moi-annuler">${esc(m.annuler)}</button></div>
    </div>`;
  }

  function catalogueHTML(c) {
    const rayons = (c.groupes || []).map((g) => {
      const el = (g.elements || []).filter((x) => idSur(x.id)).map((x) => miniHTML(x, c)).join('');
      return el ? `<section class="cat-sec" data-groupe="${esc(g.id)}"><h2>${esc(g.nom)}</h2><div class="cat-rang">${el}</div></section>` : '';
    }).join('');
    return `<div class="cat-in">
      <div class="cat-tete">
        <button type="button" class="retour-c f" data-perso="retour" data-k="a:retour">${RETOUR}${esc(c.retour)}<span class="compte-c">${esc(c.compte)}</span></button>
        <h1>${esc(c.titre)}</h1>
        <label class="recherche">${LOUPE}<input class="f" id="pfCherche" data-perso="chercher" data-k="a:chercher" maxlength="60" autocomplete="off" spellcheck="false" placeholder="${esc(c.chercher)}" aria-label="${esc(c.chercher)}"></label>
      </div>
      <p class="rien-c" id="pfRien" hidden></p>
      ${rayons}
      <section class="cat-sec moi-sec"><h2>${esc(c.moi?.titre)}</h2><div class="cat-rang">${moiHTML(c)}</div></section>
      <p class="cat-note">${esc(c.note)}</p>
    </div>`;
  }

  // La recherche : seuls les abonnements dont la clé contient chaque mot tapé restent ; une catégorie vide disparaît
  function filtrer() {
    const pf = hote?.querySelector('#pf'); if (!pf || vue !== 'catalogue') return;
    const mots = pourChercher(cherche).split(' ').filter(Boolean);
    let vus = 0;
    pf.querySelectorAll('.cat-sec[data-groupe]').forEach((s) => {
      let n = 0;
      s.querySelectorAll('.mini').forEach((m) => { const ok = correspond(m.dataset.cle || '', mots); m.hidden = !ok; if (ok) n += 1; });
      s.hidden = n === 0; vus += n;
    });
    const rien = pf.querySelector('#pfRien'), c = P.catalogue || {};
    rien.hidden = !(mots.length && !vus);
    rien.textContent = `${c.rien || ''} « ${cherche.trim()} ».`;
  }

  // ---------------------------------------------------------------- dessiner
  function dessinerVue(sansRetenue) {
    const garde = sansRetenue ? 0 : (hote.querySelector('#pf')?.scrollTop || 0);
    cartes.clear(); ouverte = null;
    const cat = vue === 'catalogue' && P.catalogue;
    if (!cat) vue = 'portefeuille';
    hote.innerHTML = `<div class="pf${O.anime ? ' entree' : ''}${cat ? ' pf-ajout' : ''}" id="pf" data-vue="${vue}">${cat ? catalogueHTML(P.catalogue) : P.vide ? videHTML(P) : pleinHTML(P)}</div>
      <div class="scene-pf" id="pfScene" role="dialog" aria-modal="true" aria-hidden="true"></div>`;
    O.anime = false;
    const pf = hote.querySelector('#pf');
    pf.scrollTop = garde;
    secoursLogos(pf);
    hote.querySelector('#pfScene').addEventListener('click', (e) => { if (e.target.id === 'pfScene' || e.target.classList.contains('scene-in')) { O.son?.retour?.(); fermer(); } });
    if (cat) {
      const champ = pf.querySelector('#pfCherche');
      champ.value = cherche;
      champ.addEventListener('input', () => { cherche = champ.value; filtrer(); });
      pf.querySelector('#moiNom')?.addEventListener('input', (e) => { moi.nom = e.target.value; });
      pf.querySelector('#moiPrix')?.addEventListener('input', (e) => { moi.prix = e.target.value; });
      filtrer();
    }
    return pf;
  }

  // Le portefeuille après un ajout : la nouvelle carte est montrée, choisie, et saute un peu
  function montrerNouvelle(pf) {
    const id = aMontrer; aMontrer = null;
    const el = pf.querySelector(`[data-k="c:${CSS.escape(id)}"]`); if (!el) return false;
    el.focus({ preventScroll: true });
    el.scrollIntoView({ block: 'center', behavior: reduit() ? 'auto' : 'smooth' });
    el.classList.add('nouvelle');
    return true;
  }

  function redessiner(k) {
    if (!hote || !P || !O) return;
    const pf = dessinerVue(true);
    if (vue === 'portefeuille' && aMontrer && montrerNouvelle(pf)) return;
    const cible = k && pf.querySelector(`[data-k="${CSS.escape(k)}"]`);
    (cible || pf.querySelector('.f'))?.focus({ preventScroll: true });
    if (cible) cible.scrollIntoView({ block: 'nearest' });
  }

  function ouvrirCatalogue(texte) {
    if (!P?.catalogue) return;
    fermer();
    vue = 'catalogue'; moiOuvert = false;
    if (typeof texte === 'string') cherche = texte.slice(0, 60);
    O.anime = !reduit();
    redessiner(typeof texte === 'string' ? 'a:chercher' : 'a:chercher');
  }
  function quitterCatalogue() {
    vue = 'portefeuille'; moiOuvert = false; cherche = '';
    O.anime = !reduit();
    redessiner('ajouter');
  }

  // « Un abonnement à moi » : le nom et le prix d'abord vérifiés ici (Rust vérifie tout de nouveau)
  function envoyerMoi() {
    const m = P.catalogue?.moi || {}, err = hote.querySelector('#moiErr');
    const nom = moi.nom.trim(), prix = moi.prix.replace(/[\s€]/g, '');
    const dire = (t, k) => { if (err) err.textContent = t || ''; hote.querySelector(`[data-k="${k}"]`)?.focus(); O.son?.retour?.(); };
    if (!nom) return dire(m.erreur_nom, 'a:moi-nom');
    if (!/^\d{1,9}([.,]\d{1,2})?$/.test(prix) || !/[1-9]/.test(prix)) return dire(m.erreur_prix, 'a:moi-prix');
    if (err) err.textContent = '';
    envoyer(m.action, { nom: nom.slice(0, 200), prix: moi.prix.trim().slice(0, 20), rythme: moi.rythme, categorie: moi.categorie });
  }

  // Un ajout part vers Rust (une seule fois, même sur un double clic)
  function envoyer(action, valeur) {
    if (!idSur(action) || Date.now() < envoiJusqua) return;
    envoiJusqua = Date.now() + 1500;
    O.son?.ok?.();
    O.action(action, valeur);
  }

  // ---------------------------------------------------------------- la fiche
  const scene = () => hote?.querySelector('#pfScene');
  const estOuverte = () => !!(ouverte && scene()?.classList.contains('ouverte'));

  // La fiche remplie (à l'ouverture, et chaque fois qu'elle change) ; puis l'élément `k` choisi, s'il y est
  function remplirScene(x, k) {
    const sc = scene();
    sc.innerHTML = sceneHTML(x);
    secoursLogos(sc);
    // (un champ tapé part en le quittant aussi, s'il a changé)
    sc.querySelectorAll('input[data-perso]').forEach((i) => i.addEventListener('change', () => champ(i, false)));
    sc.querySelector('#fPrix')?.addEventListener('input', (e) => { e.target.closest('.prix-champ')?.classList.remove('faux'); });
    const cible = (k && sc.querySelector(`[data-k="${CSS.escape(k)}"]`)) || null;
    cible?.focus({ preventScroll: true });
    cible?.scrollIntoView({ block: 'nearest' });
    return cible;
  }
  // La fiche ouverte redessinée sur place (le calendrier, la question « Supprimer ? »)
  function majScene(k) {
    const x = ouverte && cartes.get(ouverte), sc = scene(); if (!x || !sc) return;
    const haut = sc.scrollTop;
    sc.classList.add('sans-entree');
    const cible = remplirScene(x, k);
    if (!cible) { sc.scrollTop = haut; sc.querySelector('.fermer-f')?.focus({ preventScroll: true }); }
  }

  // Ouvre la fiche d'une carte ; `garde` : la même fiche redessinée (l'état a changé) : pas d'animation, la même
  // place dans la fiche, l'élément `k` choisi de nouveau
  function ouvrir(id, sansAnim, garde, k) {
    const x = cartes.get(id), sc = scene(), pf = hote?.querySelector('#pf');
    if (!x || !sc || !pf) return;
    if (!estOuverte() && !garde) retourK = document.activeElement?.closest?.('#ecran [data-k]')?.dataset.k || `c:${id}`;
    hote.querySelectorAll('.carte.cache').forEach((c) => c.classList.remove('cache'));
    ouverte = id;
    sc.classList.toggle('sans-entree', !!garde); pf.classList.toggle('sans-anim', !!garde);
    const cible = remplirScene(x, garde ? k : null);
    sc.setAttribute('aria-hidden', 'false'); sc.setAttribute('aria-label', x.nom || '');
    pf.classList.add('estompe'); pf.inert = true;
    sc.classList.remove('part'); sc.classList.add('ouverte'); sc.scrollTop = garde ? garde.haut : 0;
    if (garde) {
      hote.querySelector(`#pf .carte[data-id="${CSS.escape(id)}"]`)?.classList.add('cache');
      if (!cible) sc.querySelector('.fermer-f')?.focus({ preventScroll: true });
      return;
    }
    const source = pf.querySelector(`.carte[data-id="${CSS.escape(id)}"]`), grande = sc.querySelector('.carte.grande');
    if (source && grande && !sansAnim && !reduit()) {
      // (la carte quitte sa place et vient au-dessus de la fiche)
      const s = source.getBoundingClientRect(), g = grande.getBoundingClientRect();
      grande.style.transformOrigin = '0 0';
      grande.style.transform = `translate(${s.left - g.left}px, ${s.top - g.top}px) scale(${s.width / g.width})`;
      source.classList.add('cache');
      grande.getBoundingClientRect();
      grande.classList.add('anim'); grande.style.transform = '';
    } else if (source) source.classList.add('cache');
    sc.querySelector('.fermer-f')?.focus({ preventScroll: true });
  }

  function fermer() {
    if (!estOuverte()) { ouverte = null; return false; }
    const id = ouverte, sc = scene(), pf = hote.querySelector('#pf');
    ouverte = null; cal = null; confirme = null;
    pf.classList.remove('estompe', 'sans-anim'); pf.inert = false;
    sc.classList.remove('ouverte', 'sans-entree'); sc.classList.add('part'); sc.setAttribute('aria-hidden', 'true');
    const source = pf.querySelector(`.carte[data-id="${CSS.escape(id)}"]`), grande = sc.querySelector('.carte.grande');
    // (à la fin, la carte revenue à sa place est de nouveau choisie : on reprend là où on était)
    const r = retourK && hote.querySelector(`#pf [data-k="${CSS.escape(retourK)}"]`);
    const fin = () => {
      if (ouverte) return;
      sc.classList.remove('part'); sc.innerHTML = ''; source?.classList.remove('cache');
      if (!document.activeElement || document.activeElement === document.body || !document.activeElement.isConnected) (r || source)?.focus({ preventScroll: true });
    };
    if (source && grande && !reduit()) {
      // (la carte retourne à sa place)
      source.scrollIntoView({ block: 'nearest' });
      const s = source.getBoundingClientRect(), g = grande.getBoundingClientRect();
      grande.style.transformOrigin = '0 0'; grande.classList.add('anim');
      grande.style.transform = `translate(${s.left - g.left}px, ${s.top - g.top}px) scale(${s.width / g.width})`;
      setTimeout(fin, 530);
    } else fin();
    // (tout de suite si elle se voit déjà : une pastille de « Bientôt »)
    if (r && !r.classList.contains('cache')) r.focus({ preventScroll: true });
    return true;
  }

  // ---------------------------------------------------------------- les gestes de la fiche
  // Un geste part vers Rust (action du kit) pour la fiche ouverte : { id, ...champs }. Rust vérifie tout, enregistre
  // et renvoie le portefeuille (la fiche est alors redessinée), ou dit non dans la bulle et renvoie l'état d'avant.
  function geste(action, champs) {
    if (!idSur(ouverte)) return;
    O.action(action, { id: ouverte, ...champs });
  }

  // Le prix, le nom ou la note tapés : envoyés sur Entrée (`entree`) ou en quittant le champ, s'ils ont changé. Le
  // prix est d'abord vérifié ici (un chiffre, une virgule ou un point, deux chiffres après au plus, pas zéro).
  function champ(el, entree) {
    const v = el.value, avant = el.dataset.envoye ?? el.defaultValue, t = textesFiche();
    const suite = () => { if (entree) scene()?.querySelector(el.id === 'fPrix' ? '.choix-f .oui' : '.fermer-f')?.focus({ preventScroll: true }); };
    if (v === avant) { suite(); return; }
    if (el.id === 'fPrix') {
      const err = scene()?.querySelector('#fErr'), p = v.replace(/[\s€]/g, '');
      if (!/^\d{1,6}([.,]\d{1,2})?$/.test(p) || !/[1-9]/.test(p)) {
        if (err) err.textContent = t.erreur_prix || '';
        el.closest('.prix-champ')?.classList.add('faux');
        O.son?.retour?.();
        return;
      }
      if (err) err.textContent = '';
      el.dataset.envoye = v;
      O.son?.ok?.();
      geste('fiche-prix', { prix: v.trim().slice(0, 20) });
    } else if (el.id === 'fNom') {
      // (un nom vide n'est pas envoyé : l'ancien revient)
      if (!v.trim()) { el.value = avant; suite(); return; }
      el.dataset.envoye = v;
      geste('fiche-nom', { nom: v.slice(0, 200) });
    } else if (el.id === 'fNote') {
      el.dataset.envoye = v;
      geste('fiche-note', { note: v.slice(0, 2000) });
    }
    suite();
  }

  // Le calendrier s'ouvre sur le mois du jour choisi (ou d'aujourd'hui si sa date est passée)
  function ouvrirCalendrier() {
    const x = cartes.get(ouverte), k = textesFiche().calendrier || {}; if (!x) return;
    const min = ISO.test(k.min || '') ? k.min : '', iso = x.fiche?.date_iso;
    const d = ISO.test(iso || '') && iso >= min ? iso : min;
    if (!ISO.test(d)) return;
    cal = { id: ouverte, a: +d.slice(0, 4), m: +d.slice(5, 7) - 1 };
    confirme = null;
    majScene(`f-jour:${d}`);
    scene()?.querySelector('.cal')?.scrollIntoView({ block: 'nearest' });
  }

  // Les flèches dans la fiche : d'un élément à son voisin (comme le kit), sans jamais sortir de la fiche
  function voisinScene(dir) {
    const sc = scene(), els = [...(sc?.querySelectorAll('.f') || [])].filter((e) => e.offsetParent !== null);
    const cur = document.activeElement?.closest?.('.f');
    if (!cur || !els.includes(cur)) { els[0]?.focus({ preventScroll: true }); return; }
    const a = cur.getBoundingClientRect(), ax = a.left + a.width / 2, ay = a.top + a.height / 2;
    let best = null, bd = Infinity;
    for (const e of els) {
      if (e === cur) continue;
      const b = e.getBoundingClientRect(), dx = b.left + b.width / 2 - ax, dy = b.top + b.height / 2 - ay;
      if ((dir === 'd' && dx <= 6) || (dir === 'g' && dx >= -6) || (dir === 'b' && dy <= 6) || (dir === 'h' && dy >= -6)) continue;
      const horiz = dir === 'd' || dir === 'g';
      const aligne = horiz ? b.top < a.bottom - 2 && b.bottom > a.top + 2 : b.left < a.right - 2 && b.right > a.left + 2;
      const d = (horiz ? Math.abs(dx) : Math.abs(dy)) + (horiz ? Math.abs(dy) : Math.abs(dx)) * 2.5 + (aligne ? 0 : 100000);
      if (d < bd) { bd = d; best = e; }
    }
    if (best) { best.focus({ preventScroll: true }); best.scrollIntoView({ block: 'nearest' }); }
  }

  // Un élément de la fiche (clic, A ou Entrée) ; vrai s'il en était un
  function validerFiche(el, d) {
    const x = cartes.get(ouverte);
    if (!x) return false;
    const pastille = () => {
      el.parentElement.querySelectorAll('button').forEach((b) => { const oui = b === el; b.classList.toggle('oui', oui); b.setAttribute('aria-checked', String(oui)); });
    };
    switch (d.perso) {
      case 'f-prix': case 'f-nom': case 'f-note': champ(el, true); return true;
      case 'f-rythme': case 'f-rappel':
        if (!idSur(d.v) || el.classList.contains('oui')) return true;
        pastille(); O.son?.bascule?.();
        geste(d.perso === 'f-rythme' ? 'fiche-rythme' : 'fiche-rappel', d.perso === 'f-rythme' ? { rythme: d.v } : { rappel: d.v });
        return true;
      case 'f-auto': {
        const oui = !el.classList.contains('oui');
        el.classList.toggle('oui', oui); el.setAttribute('aria-checked', String(oui)); O.son?.bascule?.();
        geste('fiche-auto', { oui });
        return true;
      }
      case 'f-pause': O.son?.ok?.(); geste('fiche-pause', { oui: !x.fiche?.pause }); return true;
      case 'f-renouvele': O.son?.ok?.(); geste('fiche-renouvele', {}); return true;
      case 'f-suppr':
        O.son?.ok?.(); confirme = ouverte; cal = null; majScene('f-suppr-non');
        scene()?.querySelector('.f-suppr')?.scrollIntoView({ block: 'nearest' });
        return true;
      case 'f-suppr-non': O.son?.retour?.(); confirme = null; majScene('f-suppr'); return true;
      case 'f-suppr-oui': {
        if (confirme !== ouverte) return true;
        confirme = null;
        // (la carte s'en va, puis Rust la retire : la fiche se referme avec le nouvel état)
        const g = scene()?.querySelector('.carte.grande'), id = ouverte;
        g?.classList.add('part-suppr');
        el.disabled = true;
        setTimeout(() => { if (ouverte === id) geste('fiche-supprimer', {}); }, reduit() ? 0 : 380);
        return true;
      }
      case 'f-date': O.son?.ok?.(); ouvrirCalendrier(); return true;
      case 'f-cal-annuler': O.son?.retour?.(); cal = null; majScene('f-date'); return true;
      case 'f-mois': {
        if (!cal) return true;
        const t = cal.a * 12 + cal.m + (Number(d.v) < 0 ? -1 : 1);
        cal = { ...cal, a: Math.floor(t / 12), m: t % 12 };
        O.son?.tic?.();
        majScene(d.k);
        if (!scene()?.querySelector(`[data-k="${CSS.escape(d.k)}"]`)) scene()?.querySelector('.cal-d.f')?.focus({ preventScroll: true });
        return true;
      }
      case 'f-jour':
        if (!ISO.test(d.v || '')) return true;
        O.son?.ok?.(); cal = null;
        geste('fiche-date', { date: d.v });
        majScene('f-date');
        return true;
      default: return false;
    }
  }

  // ---------------------------------------------------------------- avec le kit
  window.kitAccueil = Object.freeze({
    // L'accueil dessiné (à chaque nouvel état et à chaque retour sur l'accueil), dans la vue choisie
    dessiner(h, perso, outils) {
      const rouvrir = estOuverte() ? ouverte : null, memeHote = hote === h;
      // (la fiche ouverte redessinée : la même place dans la fiche)
      const haut = rouvrir ? (scene()?.scrollTop || 0) : 0;
      hote = h; P = perso || {}; O = { ...outils };
      const garde = memeHote ? (h.querySelector('#pf')?.scrollTop || 0) : 0;
      const pf = dessinerVue(true);
      pf.scrollTop = garde;
      if (vue === 'portefeuille' && aMontrer && montrerNouvelle(pf)) return;
      if (rouvrir && cartes.has(rouvrir)) { ouvrir(rouvrir, true, { haut }, outils.focus); return; }
      // (la carte de la fiche ouverte a disparu, supprimée : on oublie la fiche)
      if (rouvrir) { cal = null; confirme = null; }
      const k = outils.focus || (vue === 'portefeuille' ? dernierK : null);
      const cible = k && pf.querySelector(`[data-k="${CSS.escape(k)}"]`);
      if (cible) cible.focus({ preventScroll: true });
      else if (rouvrir) pf.querySelector('.carte.f, .f')?.focus({ preventScroll: true });
    },
    // Un élément de l'accueil ou de la fiche (clic, A ou Entrée)
    valider,
    // Retour (Échap, B, le bouton du bas) : le calendrier ou la question « Supprimer ? » se referment, puis la
    // fiche, « Un abonnement à moi », puis on quitte « Ajouter » ; sinon le kit revient au menu de Mindset
    retour() {
      if (estOuverte() && cal) { cal = null; majScene('f-date'); return true; }
      if (estOuverte() && confirme) { confirme = null; majScene('f-suppr'); return true; }
      if (fermer()) return true;
      if (vue === 'catalogue' && moiOuvert) { moiOuvert = false; redessiner('a:moi'); return true; }
      if (vue === 'catalogue') { quitterCatalogue(); return true; }
      return false;
    },
    ordre,
  });

  // Un élément de l'accueil ou de la fiche (clic, A ou Entrée)
  function valider(el) {
    const d = el.dataset;
    if (estOuverte() && el.closest('#pfScene') && validerFiche(el, d)) return;
    { // (l'accueil et « Ajouter »)
      switch (d.perso) {
        case 'carte': if (cartes.has(d.id)) { O.son?.ok?.(); cal = null; confirme = null; ouvrir(d.id); } return;
        case 'fermer': O.son?.retour?.(); fermer(); return;
        case 'ajouter': O.son?.ok?.(); ouvrirCatalogue(); return;
        case 'retour': O.son?.retour?.(); quitterCatalogue(); return;
        case 'modele': {
          if (!idSur(d.id) || Date.now() < envoiJusqua) return;
          el.classList.remove('pop'); void el.offsetWidth; el.classList.add('pop');
          envoyer(P.catalogue?.action, d.id); return;
        }
        // (Entrée dans la recherche : le premier abonnement trouvé)
        case 'chercher': hote.querySelector('#pf .mini:not([hidden])')?.focus(); return;
        case 'moi': O.son?.ok?.(); moiOuvert = true; redessiner('a:moi-nom'); return;
        case 'moi-annuler': O.son?.retour?.(); moiOuvert = false; redessiner('a:moi'); return;
        case 'moi-rythme': case 'moi-cat': {
          if (!idSur(d.v)) return;
          if (d.perso === 'moi-rythme') moi.rythme = d.v; else moi.categorie = d.v;
          el.parentElement.querySelectorAll('button').forEach((b) => { const oui = b === el; b.classList.toggle('oui', oui); b.setAttribute('aria-pressed', String(oui)); });
          O.son?.bascule?.(); return;
        }
        // (Entrée dans le nom : le prix ; dans le prix : l'ajout)
        case 'moi-champ': if (el.id === 'moiNom') hote.querySelector('#moiPrix')?.focus(); else envoyerMoi(); return;
        case 'moi-ok': envoyerMoi(); return;
        default:
      }
    }
  }

  // Les ordres de l'appli : { kit: 'accueil', fiche: id } ouvre une fiche, { fermer: true } referme ce qui est
  // ouvert (la fiche, « Ajouter »), { voir: data-k } choisit un élément et le montre, { vue: 'ajouter' |
  // 'portefeuille' } change d'écran, { chercher: texte } montre « Ajouter » avec cette recherche, { ajoute: id } :
  // un abonnement vient d'être ajouté (retour sur sa carte), { valider: data-k } choisit un élément comme un clic
  // (dans la fiche ouverte d'abord ; un champ avec { texte } : rempli puis validé comme avec Entrée),
  // { moi: { nom, prix, rythme, categorie } } ouvre « Un abonnement à moi » déjà rempli
  function ordre(o) {
    { // (un seul ordre à la fois)
      if (idSur(o.ajoute)) {
        vue = 'portefeuille'; moiOuvert = false; cherche = ''; envoiJusqua = 0;
        moi = { nom: '', prix: '', rythme: 'mois', categorie: 'moi' };
        aMontrer = o.ajoute;
        // (un champ choisi retient l'état chez le kit jusqu'à ce qu'on le quitte : il redessine alors lui-même)
        const champ = document.activeElement?.tagName === 'INPUT' ? document.activeElement : null;
        champ?.blur();
        setTimeout(() => { if (aMontrer) redessiner(); }, champ ? 60 : 0);
      } else if (idSur(o.fiche)) {
        if (vue !== 'portefeuille') { vue = 'portefeuille'; redessiner(); }
        cal = null; confirme = null;
        ouvrir(o.fiche);
      } else if (o.fermer) {
        fermer();
        if (vue !== 'portefeuille') quitterCatalogue();
      } else if (o.vue === 'ajouter' || typeof o.chercher === 'string') {
        ouvrirCatalogue(typeof o.chercher === 'string' ? o.chercher : undefined);
      } else if (o.moi && typeof o.moi === 'object') {
        // (« Un abonnement à moi » ouvert et déjà rempli : la personne n'a plus qu'à vérifier et choisir « Ajouter »)
        const t = (v, n) => (typeof v === 'string' ? v.slice(0, n) : '');
        moi = { nom: t(o.moi.nom, 200), prix: t(o.moi.prix, 20), rythme: idSur(o.moi.rythme) ? o.moi.rythme : 'mois', categorie: idSur(o.moi.categorie) ? o.moi.categorie : 'moi' };
        fermer(); vue = 'catalogue'; moiOuvert = true;
        redessiner('a:moi-ok');
        hote?.querySelector('[data-k="a:moi-ok"]')?.scrollIntoView({ block: 'nearest' });
      } else if (o.vue === 'portefeuille') {
        if (vue !== 'portefeuille') quitterCatalogue();
      } else if (idSur(o.valider)) {
        const k = `[data-k="${CSS.escape(o.valider)}"]`;
        const el = (estOuverte() && scene()?.querySelector(k)) || hote?.querySelector(`#pf ${k}`);
        el?.focus({ preventScroll: true }); el?.scrollIntoView({ block: 'nearest' });
        if (el && el.tagName === 'INPUT') {
          if (typeof o.texte === 'string') { el.value = o.texte.slice(0, 200); valider(el); }
        } else if (el) el.click();
      } else if (idSur(o.voir) && !estOuverte()) {
        const el = hote?.querySelector(`#pf [data-k="${CSS.escape(o.voir)}"]`);
        el?.focus({ preventScroll: true }); el?.scrollIntoView({ block: 'nearest' });
      }
    }
  }

  // Fiche ouverte : les flèches restent sur elle (rien ne bouge derrière) ; dans un champ, gauche et droite
  // déplacent le curseur
  addEventListener('keydown', (e) => {
    if (!estOuverte() || !e.key.startsWith('Arrow')) return;
    if (e.target.tagName === 'INPUT' && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) { e.stopPropagation(); return; }
    e.preventDefault(); e.stopPropagation();
    voisinScene({ ArrowRight: 'd', ArrowLeft: 'g', ArrowDown: 'b', ArrowUp: 'h' }[e.key]);
  }, true);
  // Le dernier élément choisi sur le portefeuille (on y revient depuis « Bientôt »)
  document.addEventListener('focusin', (e) => { const k = e.target.closest?.('#pf[data-vue="portefeuille"] [data-k]')?.dataset.k; if (k) dernierK = k; });
  // Le reflet de la carte suit la souris
  document.addEventListener('pointermove', (e) => {
    const c = e.target.closest?.('.carte'); if (!c) return;
    const r = c.getBoundingClientRect();
    c.style.setProperty('--mx', `${(((e.clientX - r.left) / r.width) * 100).toFixed(1)}%`);
    c.style.setProperty('--my', `${(((e.clientY - r.top) / r.height) * 100).toFixed(1)}%`);
  });
})();
