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
// - une carte (ou une pastille de « Bientôt ») ouvre sa fiche en lecture seule : la carte s'agrandit, la fiche
//   apparaît dessous (la fiche modifiable vient à l'étape 5) ; Retour, Échap ou B la referment ;
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

  // La fiche en lecture seule, sous la carte agrandie (les mêmes places que la fiche modifiable de la maquette)
  function sceneHTML(x) {
    const f = x.fiche || {}, communs = P.communs || {};
    const choix = (liste, choisi) => (Array.isArray(liste) ? liste : []).map((c) => `<span class="${Array.isArray(c) && c[0] === choisi ? 'oui' : ''}">${esc(Array.isArray(c) ? c[1] : '')}</span>`).join('');
    return `<div class="scene-in">
      <button type="button" class="fermer-f f" data-perso="fermer" data-k="fermer">${RETOUR}Retour</button>
      ${carteHTML(x, true)}
      <div class="fiche">
        <div class="f-col">
          <div class="f-titre">${esc(x.nom)}<em>${esc(f.categorie)}</em></div>
          <span class="lbl">Je paie</span>
          <div class="prix-f">${esc(f.prix)}</div>
          <span class="lbl">Tous les</span>
          <div class="choix-l">${choix(communs.rythmes, f.rythme)}</div>
          <div class="calc"><div>Par mois<b>${esc(f.mois)}</b></div><div>Par an<b>${esc(f.an)}</b></div></div>
          ${f.etat ? `<p class="f-etat${x.statut === 'pause' ? ' pause' : ''}">${esc(f.etat)}</p>` : ''}
          ${f.note ? `<p class="f-note">${esc(f.note)}</p>` : ''}
        </div>
        <div class="f-col">
          <div class="rangf"><div class="txt">${esc(f.titre_date)}</div><div class="date${x.statut === 'renouveler' ? ' passee' : ''}">${esc(f.date)}${f.quand ? `<small>${esc(f.quand)}</small>` : ''}</div></div>
          <div class="rangf"><div class="txt">Renouvellement automatique<small>${f.auto ? 'Il repart tout seul à chaque fois' : 'Non : il faut le renouveler soi-même'}</small></div><span class="bascule-l${f.auto ? ' oui' : ''}" role="img" aria-label="${f.auto ? 'Oui' : 'Non'}"></span></div>
          <div class="rangf rappel"><div class="txt">Me prévenir<small>${esc(f.rappel_texte)}</small></div></div>
          <div class="choix-l petit">${choix(communs.rappels, f.rappel)}</div>
          <p class="f-bientot">Bientôt, tu pourras changer cet abonnement ici.</p>
        </div>
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

  function ouvrir(id, sansAnim) {
    const x = cartes.get(id), sc = scene(), pf = hote?.querySelector('#pf');
    if (!x || !sc || !pf) return;
    if (!estOuverte()) retourK = document.activeElement?.closest?.('#ecran [data-k]')?.dataset.k || `c:${id}`;
    hote.querySelectorAll('.carte.cache').forEach((c) => c.classList.remove('cache'));
    ouverte = id;
    sc.innerHTML = sceneHTML(x);
    secoursLogos(sc);
    sc.setAttribute('aria-hidden', 'false'); sc.setAttribute('aria-label', x.nom || '');
    pf.classList.add('estompe'); pf.inert = true;
    sc.classList.remove('part'); sc.classList.add('ouverte'); sc.scrollTop = 0;
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
    ouverte = null;
    pf.classList.remove('estompe'); pf.inert = false;
    sc.classList.remove('ouverte'); sc.classList.add('part'); sc.setAttribute('aria-hidden', 'true');
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

  // ---------------------------------------------------------------- avec le kit
  window.kitAccueil = Object.freeze({
    // L'accueil dessiné (à chaque nouvel état et à chaque retour sur l'accueil), dans la vue choisie
    dessiner(h, perso, outils) {
      const rouvrir = estOuverte() ? ouverte : null, memeHote = hote === h;
      hote = h; P = perso || {}; O = { ...outils };
      const garde = memeHote ? (h.querySelector('#pf')?.scrollTop || 0) : 0;
      const pf = dessinerVue(true);
      pf.scrollTop = garde;
      if (vue === 'portefeuille' && aMontrer && montrerNouvelle(pf)) return;
      if (rouvrir && cartes.has(rouvrir)) { ouvrir(rouvrir, true); return; }
      const k = outils.focus || (vue === 'portefeuille' ? dernierK : null);
      const cible = k && pf.querySelector(`[data-k="${CSS.escape(k)}"]`);
      cible?.focus({ preventScroll: true });
    },
    // Un élément de l'accueil (clic, A ou Entrée)
    valider(el) {
      const d = el.dataset;
      switch (d.perso) {
        case 'carte': if (cartes.has(d.id)) { O.son?.ok?.(); ouvrir(d.id); } return;
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
    },
    // Retour (Échap, B, le bouton du bas) : la fiche se referme, « Un abonnement à moi » aussi, puis on quitte
    // « Ajouter » ; sinon le kit revient au menu de Mindset
    retour() {
      if (fermer()) return true;
      if (vue === 'catalogue' && moiOuvert) { moiOuvert = false; redessiner('a:moi'); return true; }
      if (vue === 'catalogue') { quitterCatalogue(); return true; }
      return false;
    },
    // Les ordres de l'appli : { kit: 'accueil', fiche: id } ouvre une fiche, { fermer: true } referme ce qui est
    // ouvert (la fiche, « Ajouter »), { voir: data-k } choisit un élément et le montre, { vue: 'ajouter' |
    // 'portefeuille' } change d'écran, { chercher: texte } montre « Ajouter » avec cette recherche, { ajoute: id } :
    // un abonnement vient d'être ajouté (retour sur sa carte), { valider: data-k } choisit un élément comme un clic,
    // { moi: { nom, prix, rythme, categorie } } ouvre « Un abonnement à moi » déjà rempli
    ordre(o) {
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
        const el = hote?.querySelector(`#pf [data-k="${CSS.escape(o.valider)}"]`);
        el?.focus({ preventScroll: true }); el?.scrollIntoView({ block: 'nearest' });
        if (el && el.tagName !== 'INPUT') el.click();
      } else if (idSur(o.voir) && !estOuverte()) {
        const el = hote?.querySelector(`#pf [data-k="${CSS.escape(o.voir)}"]`);
        el?.focus({ preventScroll: true }); el?.scrollIntoView({ block: 'nearest' });
      }
    },
  });

  // Fiche ouverte : les flèches restent sur elle (rien ne bouge derrière)
  addEventListener('keydown', (e) => {
    if (!estOuverte() || !e.key.startsWith('Arrow')) return;
    e.preventDefault(); e.stopPropagation();
    scene()?.querySelector('.fermer-f')?.focus({ preventScroll: true });
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
