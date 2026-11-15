// Le portefeuille de Paymind (étape 3) : l'accueil de la maquette F validée par Jordan (maquettes\proposition-f.html).
// Le kit (kit.js) l'appelle à la place de ses tuiles (bloc accueil.perso de l'état, voir « perso » en tête de kit.js) ;
// il garde l'en-tête, le bas, les flèches, A et B (Entrée et Échap), les sons et les pages (« Ajouter », « Bientôt »).
//
// Rien n'est calculé ici : Rust (src\portefeuille.rs) envoie les totaux, les dates et les montants déjà écrits en
// français. Tout texte reçu est échappé (esc du kit), chaque couleur et chaque id vérifiés, chaque image passée par
// urlSure du kit. Ce fichier ne parle jamais à Rust lui-même : les gestes passent par le kit.
//
// - une carte (ou une pastille de « Bientôt ») ouvre sa fiche en lecture seule : la carte s'agrandit, la fiche
//   apparaît dessous (la fiche modifiable vient à l'étape 5) ; Retour, Échap ou B la referment ;
// - une carte sans logo (étape 4 : les logos de la liste toute prête) montre une plaque avec ses initiales, aux
//   couleurs de sa catégorie ;
// - la liste vide : un seul grand bouton, « Ajoute ton premier abonnement ».
(() => {
  'use strict';
  const reduit = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  const couleurSure = (c, defaut) => (typeof c === 'string' && /^#[0-9a-f]{6}$/i.test(c) ? c : defaut);
  const idSur = (t) => typeof t === 'string' && /^[\w:.-]{1,60}$/.test(t);
  const PLUS = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>';
  const RETOUR = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 5l-7 7 7 7"/></svg>';

  let P = null;          // le dernier portefeuille reçu
  let O = null;          // les outils du kit (esc, urlSure, son...)
  let hote = null;       // #ecran
  let ouverte = null;    // l'id de la carte dont la fiche est ouverte
  let retourK = null;    // l'élément à reprendre en refermant la fiche
  let dernierK = null;   // le dernier élément choisi sur l'accueil (pour y revenir depuis une page)
  const cartes = new Map();

  const esc = (t) => O.esc(t);
  const styleCouleurs = (x) => `--c1:${couleurSure(x.c1, '#5A5A6E')};--c2:${couleurSure(x.c2, '#17171F')}`;

  // Le logo d'une carte, ou la plaque de ses initiales (rond : la pastille de « Bientôt »)
  function logo(x, rond) {
    const u = O.urlSure(x.logo || '');
    const ini = esc(x.initiales || '?');
    if (u) return rond ? `<img src="${u}" alt="" data-logo="${ini}">` : `<span class="c-logo"><img src="${u}" alt="" data-logo="${ini}"></span>`;
    return rond ? `<i>${ini}</i>` : `<span class="c-logo c-plaque"><b>${ini}</b></span>`;
  }

  function carteHTML(x, grande) {
    const st = x.statut === 'pause' || x.statut === 'renouveler' ? ` ${x.statut}` : '';
    const etq = x.badge ? `<span class="c-badge${st}">${esc(x.badge)}</span>` : x.etiquette ? `<span>${esc(x.etiquette)}</span>` : '';
    const dedans = `<span class="c-haut">${logo(x)}<span class="c-etq">${etq}</span></span>
      <span class="c-bas"><span class="c-gauche"><b class="c-nom">${esc(x.nom)}</b><span class="c-proch">${esc(x.ligne)}</span></span><span class="c-prix"><small>${esc(x.rythme)}</small>${esc(x.prix)}</span></span>
      <i class="c-reflet"></i>`;
    if (grande) return `<div class="carte grande${st}" style="${styleCouleurs(x)}" aria-hidden="true">${dedans}</div>`;
    const aide = [x.nom, `${x.prix} ${x.rythme}`, x.badge, x.ligne].filter(Boolean).join(', ');
    return `<button type="button" class="carte f${st}" data-perso="carte" data-id="${esc(x.id)}" data-k="c:${esc(x.id)}" style="${styleCouleurs(x)}" aria-label="${esc(aide)}">${dedans}</button>`;
  }

  const ajouterHTML = (p, grand) => (idSur(p.ajouter)
    ? `<button type="button" class="ajouter${grand ? ' grand' : ''} f" data-action="${esc(p.ajouter)}" data-k="ajouter">${PLUS}${grand ? `<b>${esc(p.titre)}</b>` : 'Ajouter'}</button>`
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
    // (une pastille par abonnement : son rond, sa date, son nom tant qu'il n'a pas de logo)
    const pastilles = (b.elements || []).filter((x) => idSur(x.id) && cartes.has(x.id)).map((x) =>
      `<button type="button" class="pas f" data-perso="carte" data-id="${esc(x.id)}" data-k="b:${esc(x.id)}" title="${esc(x.aide)}" aria-label="${esc(x.aide)}"><span class="rond-c" style="${styleCouleurs(x)}">${logo(x, true)}</span><b>${esc(x.date)}</b>${O.urlSure(x.logo || '') ? '' : `<span class="pas-nom">${esc(x.nom)}</span>`}</button>`).join('');
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

  // Un logo qui ne se charge pas : la plaque des initiales à sa place
  function secoursLogos(racine) {
    racine.querySelectorAll('img[data-logo]').forEach((img) => img.addEventListener('error', () => {
      const parent = img.parentElement, plaque = document.createElement(parent.classList.contains('c-logo') ? 'b' : 'i');
      plaque.textContent = img.dataset.logo || '?';
      parent.classList.toggle('c-plaque', parent.classList.contains('c-logo'));
      parent.replaceChildren(plaque);
    }, { once: true }));
  }

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

  window.kitAccueil = Object.freeze({
    // L'accueil dessiné (à chaque nouvel état et à chaque retour sur l'accueil)
    dessiner(h, perso, outils) {
      const garde = h.querySelector('#pf')?.scrollTop || 0, rouvrir = estOuverte() ? ouverte : null;
      hote = h; P = perso || {}; O = outils; cartes.clear(); ouverte = null;
      h.innerHTML = `<div class="pf${outils.anime ? ' entree' : ''}" id="pf">${P.vide ? videHTML(P) : pleinHTML(P)}</div>
        <div class="scene-pf" id="pfScene" role="dialog" aria-modal="true" aria-hidden="true"></div>`;
      const pf = h.querySelector('#pf');
      pf.scrollTop = garde;
      secoursLogos(pf);
      h.querySelector('#pfScene').addEventListener('click', (e) => { if (e.target.id === 'pfScene' || e.target.classList.contains('scene-in')) { O.son?.retour?.(); fermer(); } });
      if (rouvrir && cartes.has(rouvrir)) { ouvrir(rouvrir, true); return; }
      const k = outils.focus || dernierK;
      const cible = k && pf.querySelector(`[data-k="${CSS.escape(k)}"]`);
      cible?.focus({ preventScroll: true });
    },
    // Une carte ou une pastille (clic, A ou Entrée) ouvre sa fiche ; « Retour » de la fiche la referme
    valider(el) {
      const d = el.dataset;
      if (d.perso === 'carte' && cartes.has(d.id)) { O.son?.ok?.(); ouvrir(d.id); } else if (d.perso === 'fermer') { O.son?.retour?.(); fermer(); }
    },
    // Retour (Échap, B, le bouton du bas) : la fiche se referme ; sinon le kit revient au menu de Mindset
    retour: () => fermer(),
    // Les ordres de l'appli : { kit: 'accueil', fiche: id } ouvre une fiche, { kit: 'accueil', fermer: true } la
    // referme, { kit: 'accueil', voir: 'ajouter' } choisit un élément de l'accueil et le montre (data-k)
    ordre(o) {
      if (idSur(o.fiche)) ouvrir(o.fiche);
      else if (o.fermer) fermer();
      else if (idSur(o.voir) && !estOuverte()) {
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
  // Le dernier élément choisi sur l'accueil (on y revient depuis « Ajouter » ou « Bientôt »)
  document.addEventListener('focusin', (e) => { const k = e.target.closest?.('#pf [data-k]')?.dataset.k; if (k) dernierK = k; });
  // Le reflet de la carte suit la souris
  document.addEventListener('pointermove', (e) => {
    const c = e.target.closest?.('.carte'); if (!c) return;
    const r = c.getBoundingClientRect();
    c.style.setProperty('--mx', `${(((e.clientX - r.left) / r.width) * 100).toFixed(1)}%`);
    c.style.setProperty('--my', `${(((e.clientY - r.top) / r.height) * 100).toFixed(1)}%`);
  });
})();
