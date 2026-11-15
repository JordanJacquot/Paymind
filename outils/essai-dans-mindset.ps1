# Paymind dans un Mindset de test : la CONSTRUCTION D'ESSAI de Paymind installée dans un faux disque de la suite sous
# %TEMP%, lancée cachée par son relais comme Mindset le fait, trouvée et accrochée dans un panneau par le VRAI code
# d'accrochage de Mindset (MSIntegration de D:\Projets\Mindset\Mindset\fichiers\modules\natif.cs, compilé tel quel,
# en lecture seule), capturée, puis fermée par « Fermer » de Mindset (WM_CLOSE) et par « Tout quitter »
# (quitter.demande). À lancer SEULEMENT sur un bureau Windows à part :
#
#   & '.\outils\sur-bureau.ps1' -Script '.\outils\essai-dans-mindset.ps1' -Sortie '.\_test\essai-dans-mindset.txt' `
#       -Arguments "-Exe '<Mindset.exe d''essai>' -Relais '<Paymind.exe>' -Resultats '<dossier>'"
#
# Le contrat complet avec le vrai code de Mindset (installation signée, version.json, verrou...) est vérifié à part,
# sur l'exe livré, par D:\Projets\Mindset\outils\tester-contrat.ps1 -AppliConstruite (voir README.md).
#
# Rien sur le vrai PC : la construction d'essai ne lit pas HKLM, ne lance qu'elle-même, n'écrit que dans le dossier
# d'essai, avec un verrou et un signal d'essai (Local\PaymindEssai-...) ; seuls les processus lancés depuis le dossier
# d'essai sont arrêtés à la fin. Le script refuse un exe qui n'est pas la construction d'essai.
#
# Vérifié, dans l'ordre :
#  1. le relais Paymind.exe lance Mindset.exe de son dossier avec -Demarrage, puis se ferme ;
#  2. Mindset trouve sa fenêtre (MSIntegration.Trouver : titre exact « Paymind », classe « Tauri Window ») ;
#  3. accrochée dans le panneau (MSIntegration.Accrocher) : elle le voit (journal) et se montre ;
#  4. etat.json : la carte de Mindset dit « Aucun abonnement pour l'instant » ; le thème de la suite est suivi ; premier
#     lancement des rappels : aucune bulle pour le passé ;
#  5. une capture de Paymind affichée dans le panneau ;
#  6. « Fermer » de Mindset (WM_CLOSE) : elle quitte en moins de 6 s, verrou rendu ;
#  7. les bulles (étape 6) : un rappel manqué pendant que le « PC était éteint » (Netflix payé aujourd'hui, rappel la
#     veille, dernière vérification il y a 2 jours). Relancée et accrochée de nouveau, elle demande une bulle
#     (notification.demande) avec le bon texte, lue et supprimée par ce faux Mindset comme le vrai le fait (AUCUNE bulle
#     n'est montrée : ce script ne fait que lire le fichier) ; le clic (« notif:<id> » dans afficher.demande, ce que
#     Mindset écrit) ouvre sa fiche (journal, capture) ; une bulle inconnue est ignorée ; la carte de Mindset suit ;
#     aucune seconde bulle ; puis « Tout quitter » (quitter.demande) : elle quitte en moins de 6 s ;
#  8. trois rappels en même temps : une seule bulle « 3 rappels de paiement », le clic ouvre le portefeuille (journal,
#     capture) ; relancée encore une fois sans rien changer : aucune bulle (jamais deux fois) ;
#  9. plus aucun processus de l'essai ; rien de créé hors de l'essai.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Relais,
    [Parameter(Mandatory = $true)][string]$Resultats,
    [string]$Mindset = 'D:\Projets\Mindset',
    [string]$Theme = 'crepuscule'
)
$ErrorActionPreference = 'Stop'
$echecs = New-Object System.Collections.Generic.List[string]
# (Write-Host : une ligne écrite dans une fonction qui rend une valeur ne doit pas se mêler à cette valeur)
function Ok([string]$t) { Write-Host "OK : $t" }
function Echec([string]$t) { Write-Host "ÉCHEC : $t"; $echecs.Add($t) }
function Verifier([bool]$c, [string]$t) { if ($c) { Ok $t } else { Echec $t } }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
# Le vrai code d'accrochage de Mindset, lu en lecture seule et compilé tel quel
Add-Type -TypeDefinition ([IO.File]::ReadAllText((Join-Path $Mindset 'Mindset\fichiers\modules\natif.cs'), [Text.Encoding]::UTF8))
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class EssaiMindset
{
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
}
'@

# Attend qu'une condition soit vraie, en laissant passer les messages de Windows de ce fil (le panneau)
function Attendre([scriptblock]$Condition, [int]$Ms) {
    $t = [Diagnostics.Stopwatch]::StartNew()
    do {
        [System.Windows.Forms.Application]::DoEvents()
        try { if (& $Condition) { return $true } } catch {}
        Start-Sleep -Milliseconds 100
    } while ($t.ElapsedMilliseconds -lt $Ms)
    $false
}

# --- Le faux disque de la suite --------------------------------------------------------------------------------------
$Exe = (Resolve-Path -LiteralPath $Exe).Path
$Relais = (Resolve-Path -LiteralPath $Relais).Path
if (-not [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($Exe)).Contains('PAYMIND_ESSAI_NOM')) {
    Write-Output "ÉCHEC : $Exe n'est pas la construction d'essai (option cargo « essai ») : rien n'est lancé."
    exit 1
}
$racine = Join-Path $env:TEMP "paymind-essai-mindset-$PID"
if (Test-Path -LiteralPath $racine) { Remove-Item -LiteralPath $racine -Recurse -Force }
$programmes = Join-Path $racine 'Mindset\Programmes\Paymind'
$donnees = Join-Path $racine 'Mindset\Donnees\Paymind'
$local = Join-Path $racine 'Local'
New-Item -ItemType Directory -Force -Path $programmes, $donnees, (Join-Path $local 'Mindset') | Out-Null
$exeEssai = Join-Path $programmes 'Mindset.exe'
Copy-Item -LiteralPath $Exe -Destination $exeEssai
Copy-Item -LiteralPath $Relais -Destination (Join-Path $programmes 'Paymind.exe')
[IO.File]::WriteAllText((Join-Path $programmes 'version.json'), '{"appli":"paymind","version":"0.1.0","mindset_min":"2.15.0"}', (New-Object Text.UTF8Encoding($false)))
New-Item -ItemType Directory -Force -Path $Resultats | Out-Null
$nomEssai = "dans-mindset-$PID"
# suite.json : Paymind gérée par Mindset (son dossier), avec le thème choisi dans Mindset
$suite = [ordered]@{ theme = $Theme; micro = ''; casque = ''; applis = [ordered]@{ paymind = $programmes }; modifie = (Get-Date -Format s) }
[IO.File]::WriteAllText((Join-Path $local 'Mindset\suite.json'), ($suite | ConvertTo-Json -Depth 4), (New-Object Text.UTF8Encoding($false)))
$journal = Join-Path $donnees 'journal.txt'
function Get-Journal {
    if (-not (Test-Path -LiteralPath $journal)) { return @() }
    $f = New-Object IO.FileStream($journal, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try { $l = (New-Object IO.StreamReader($f, [Text.Encoding]::UTF8)).ReadToEnd() } finally { $f.Dispose() }
    @($l -split "`r`n" | Where-Object { $_ })
}
function Compter([string]$Motif) { @(Get-Journal | Where-Object { $_ -like "*$Motif*" }).Count }
function Get-Hotes { @(Get-CimInstance Win32_Process -Filter "Name = 'Mindset.exe'" | Where-Object { $_.ExecutablePath -eq $exeEssai }) }

# --- Les rappels (étape 6) : les fichiers du dossier d'essai, et le faux Mindset qui lit les bulles --------------------
$notif = Join-Path $donnees 'notification.demande'
$afficher = Join-Path $donnees 'afficher.demande'
$nbsp = [char]0xA0
function Write-Donnee([string]$Nom, [string]$Texte) {
    [IO.File]::WriteAllText((Join-Path $donnees $Nom), $Texte, (New-Object Text.UTF8Encoding($false)))
}
# Un abonnement mensuel, payé dans $Jours jours (0 : aujourd'hui), avec ce rappel
function New-Abonnement([string]$Id, [string]$Nom, [int]$Prix, [int]$Jours, [string]$Rappel) {
    $d = (Get-Date).Date.AddDays($Jours)
    [ordered]@{ id = $Id; nom = $Nom; catalogue = $null; categorie = 'moi'; prix = $Prix; rythme = 'mois'; date = $d.ToString('yyyy-MM-dd')
        ancrage = $d.Day; auto = $true; rappel = $Rappel; pause = $false; note = '' }
}
function Set-Abonnements($Liste) {
    Write-Donnee 'abonnements.json' (ConvertTo-Json -InputObject ([ordered]@{ format = 1; abonnements = @($Liste) }) -Depth 4)
}
# La dernière vérification des rappels, il y a $Jours jours à 10 h (comme un PC resté éteint depuis)
function Set-DerniereVerification([int]$Jours) {
    $v = (Get-Date).Date.AddDays(-$Jours).AddHours(10).ToString("yyyy-MM-dd'T'HH:mm")
    Write-Donnee 'rappels.json' ('{ "format": 1, "verifie": "' + $v + '", "attente": [], "envoyees": [] }')
}
# Le faux Mindset (Watch-NotificationsApplis) : lit notification.demande et le supprime ; rien n'est montré
function Read-Bulle([int]$Ms) {
    $script:bulle = $null
    [void](Attendre {
        if (Test-Path -LiteralPath $notif) {
            $o = [IO.File]::ReadAllBytes($notif)
            Remove-Item -LiteralPath $notif -Force
            $script:bulle = [pscustomobject]@{ Bom = ($o.Length -ge 3 -and $o[0] -eq 0xEF); Json = ([Text.Encoding]::UTF8.GetString($o) | ConvertFrom-Json) }
        }
        $null -ne $script:bulle
    } $Ms)
    $script:bulle
}
function Save-Capture([string]$Nom) {
    $capture = Join-Path $donnees 'capture.png'
    if (Test-Path -LiteralPath $capture) { Remove-Item -LiteralPath $capture -Force }
    [IO.File]::WriteAllText((Join-Path $donnees 'capture.demande'), '1')
    if (-not (Attendre { Test-Path -LiteralPath $capture } 6000)) { return $false }
    [void](Attendre { $false } 300)
    $dest = Join-Path $Resultats $Nom
    Copy-Item -LiteralPath $capture -Destination $dest -Force
    Write-Host "Capture : $dest ($((Get-Item -LiteralPath $dest).Length) octets)"
    $true
}
function Get-Carte { [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Join-Path $donnees 'etat.json'))) | ConvertFrom-Json }
# « Tout quitter » (quitter.demande) : elle quitte en moins de 6 s
function Stop-ParMindset($Copie, [string]$Quand) {
    $t = [Diagnostics.Stopwatch]::StartNew()
    [IO.File]::WriteAllText((Join-Path $donnees 'quitter.demande'), "$PID")
    $p = Get-Process -Id $Copie.Id -ErrorAction SilentlyContinue
    Verifier ((-not $p) -or $p.WaitForExit(6000)) "$Quand : « Tout quitter » (quitter.demande), elle quitte en moins de 6 s ($($t.ElapsedMilliseconds) ms)"
}

# Ce qui existe déjà hors de l'essai (comparé à la fin : rien ne doit y apparaître)
$horsEssai = @(
    (Join-Path $env:LOCALAPPDATA 'Mindset\WebView2-Tauri\Paymind'),
    (Join-Path $env:LOCALAPPDATA 'Mindset\Paymind'),
    (Join-Path $env:LOCALAPPDATA 'JordanJacquot.Paymind')
)
$avant = @{}
foreach ($d in $horsEssai) { $avant[$d] = if (Test-Path -LiteralPath $d) { (Get-Item -LiteralPath $d).LastWriteTimeUtc } else { $null } }

# Lance Paymind comme Mindset : son relais (le champ « exe »), caché (-Demarrage), avec le dossier d'essai
function Start-ParRelais {
    $psi = New-Object Diagnostics.ProcessStartInfo (Join-Path $programmes 'Paymind.exe')
    $psi.Arguments = '-Demarrage'
    $psi.UseShellExecute = $false
    $psi.WorkingDirectory = $programmes
    $psi.EnvironmentVariables['PAYMIND_DONNEES'] = $donnees
    $psi.EnvironmentVariables['PAYMIND_ESSAI_LOCAL'] = $local
    $psi.EnvironmentVariables['PAYMIND_ESSAI_NOM'] = $nomEssai
    [Diagnostics.Process]::Start($psi)
}

# Le panneau de Mindset : une fenêtre à la taille de l'onglet (sur le bureau à part, jamais sur celui de Jordan)
$cadre = New-Object System.Windows.Forms.Form
$cadre.Text = 'Mindset (essai)'
$cadre.ShowInTaskbar = $false
$cadre.StartPosition = 'Manual'
$cadre.Location = New-Object Drawing.Point(0, 0)
$cadre.ClientSize = New-Object Drawing.Size(1180, 760)
$cadre.BackColor = [Drawing.Color]::FromArgb(6, 5, 15)
$panneau = New-Object System.Windows.Forms.Panel
$panneau.Dock = 'Fill'
$cadre.Controls.Add($panneau)
$cadre.Show()
[System.Windows.Forms.Application]::DoEvents()

# Trouve, accroche et vérifie une copie lancée par son relais ; rend { Id, Fenetre } ou $null
function Open-DansMindset([string]$Quand) {
    $avantPid = @(Get-Hotes | ForEach-Object { [int]$_.ProcessId })
    $r = Start-ParRelais
    Verifier ($r.WaitForExit(10000) -and $r.ExitCode -eq 0) "$Quand : le relais Paymind.exe lance le programme puis se ferme"
    # (les valeurs trouvées pendant l'attente sont gardées au niveau du script : le bloc tourne dans Attendre)
    $script:hote = $null
    [void](Attendre { $script:hote = Get-Hotes | Where-Object { $avantPid -notcontains [int]$_.ProcessId } | Select-Object -First 1; $null -ne $script:hote } 10000)
    $hote = $script:hote
    Verifier ($null -ne $hote) "$Quand : Mindset.exe de son dossier tourne (trouvé par son chemin)"
    if (-not $hote) { return $null }
    Verifier ($hote.CommandLine -match '-Demarrage') "$Quand : lancé caché (-Demarrage) par le relais"
    $id = [int]$hote.ProcessId
    $script:h = [IntPtr]::Zero
    $vu = Attendre { $script:h = [MSIntegration]::Trouver($id, 'Paymind'); $script:h -ne [IntPtr]::Zero } 20000
    $h = $script:h
    Verifier $vu "$Quand : Mindset trouve sa fenêtre (titre exact « Paymind », classe Tauri Window)"
    if (-not $vu) { return $null }
    Verifier (-not [EssaiMindset]::IsWindowVisible($h)) "$Quand : cachée tant qu'elle n'est pas accrochée"
    $accrochees = Compter 'Affichée dans Mindset'
    $styles = [MSIntegration]::Accrocher($h, $panneau.Handle, $panneau.ClientSize.Width, $panneau.ClientSize.Height)
    Verifier ($null -ne $styles -and [MSIntegration]::EstEnfantDe($h, $panneau.Handle)) "$Quand : accrochée dans le panneau de Mindset"
    Verifier (Attendre { (Compter 'Affichée dans Mindset') -gt $accrochees } 6000) "$Quand : elle se sait affichée dans Mindset (journal)"
    Verifier ([MSIntegration]::EstMontree($h)) "$Quand : elle se montre"
    [pscustomobject]@{ Id = $id; Fenetre = $h }
}

try {
    # --- 1 à 4. Lancée par son relais, trouvée et accrochée ------------------------------------------------------------
    $a = Open-DansMindset 'Premier lancement'
    $etat = Join-Path $donnees 'etat.json'
    Verifier (Attendre { Test-Path -LiteralPath $etat } 5000) 'etat.json écrit pour la carte de Mindset'
    if (Test-Path -LiteralPath $etat) {
        $o = [IO.File]::ReadAllBytes($etat)
        Verifier ($o[0] -eq [byte][char]'{') 'etat.json en UTF-8 sans BOM'
        $j = [Text.Encoding]::UTF8.GetString($o) | ConvertFrom-Json
        Verifier ($j.Titre -ceq 'Mes abonnements' -and @($j.Lignes).Count -eq 1 -and $j.Lignes[0].Texte -ceq "Aucun abonnement pour l'instant" -and $j.Lignes[0].Etat -ceq 'info') "la carte dit « $($j.Lignes[0].Texte) »"
    }
    Verifier ((Compter 'Écran : menu du clic droit, raccourcis du navigateur, saisie automatique, mots de passe, outils de développement et zoom coupés') -ge 1 -or (Attendre { (Compter 'outils de développement et zoom coupés') -ge 1 } 10000)) 'réglages de WebView2 coupés et relus (journal)'
    Verifier (Attendre { (Compter 'Écran : page chargée http://tauri.localhost/index.html') -ge 1 } 15000) 'la page de Paymind est chargée (journal)'
    Verifier ((Compter 'Thème : ') -eq 0) "le thème de la suite (« $Theme ») est lu sans erreur"
    Verifier (Attendre { (Compter 'Rappels : premier lancement') -ge 1 } 5000) 'rappels : premier lancement, rappels.json écrit (journal)'
    Verifier ((Test-Path -LiteralPath (Join-Path $donnees 'rappels.json')) -and -not (Test-Path -LiteralPath $notif)) 'premier lancement : aucune bulle pour le passé'

    # --- 5. La capture ------------------------------------------------------------------------------------------------
    if ($a) {
        [void](Attendre { $false } 3500)   # (la page dessinée, la 3D lancée)
        $capture = Join-Path $donnees 'capture.png'
        [IO.File]::WriteAllText((Join-Path $donnees 'capture.demande'), '1')
        Verifier (Attendre { Test-Path -LiteralPath $capture } 6000) 'capture de Paymind affichée dans Mindset'
        if (Test-Path -LiteralPath $capture) {
            [void](Attendre { $false } 300)
            $dest = Join-Path $Resultats 'capture-dans-mindset.png'
            Copy-Item -LiteralPath $capture -Destination $dest -Force
            Write-Output "Capture : $dest ($((Get-Item -LiteralPath $dest).Length) octets)"
        }

        # --- 6. « Fermer » de Mindset (WM_CLOSE) ----------------------------------------------------------------------
        $t = [Diagnostics.Stopwatch]::StartNew()
        [MSIntegration]::Fermer($a.Fenetre)
        $p = Get-Process -Id $a.Id -ErrorAction SilentlyContinue
        $sorti = (-not $p) -or $p.WaitForExit(6000)
        Verifier $sorti "« Fermer » de Mindset (WM_CLOSE) : elle quitte ($($t.ElapsedMilliseconds) ms)"
        Verifier (-not [EssaiMindset]::IsWindow($a.Fenetre)) 'sa fenêtre a disparu'
        $m = $null
        Verifier (-not [Threading.Mutex]::TryOpenExisting("Local\PaymindEssai-$nomEssai-Instance", [ref]$m)) 'son verrou est rendu'
        if ($m) { $m.Dispose() }
    }

    # --- 7. Un rappel manqué « PC éteint » : relancée, la bulle, le clic, puis « Tout quitter » ----------------------
    # Netflix payé aujourd'hui, rappel la veille (hier à 9 h) ; dernière vérification il y a 2 jours
    Set-Abonnements @((New-Abonnement 'a1' 'Netflix' 1349 0 'veille'))
    Set-DerniereVerification 2
    $prets = Compter 'Écran prêt'
    $b = Open-DansMindset 'Relancée'
    if ($b) {
        $bulle = Read-Bulle 8000
        Verifier ($null -ne $bulle) 'rappel manqué : une bulle demandée à Mindset (notification.demande), lue et supprimée par le faux Mindset'
        if ($bulle) {
            $j = $bulle.Json
            Verifier (-not $bulle.Bom) 'notification.demande en UTF-8 sans BOM'
            Verifier ($j.titre -ceq "Netflix aujourd'hui : 13,49$nbsp€") "titre de la bulle : « $($j.titre) »"
            Verifier ($j.texte -clike 'Paiement prévu le *.' -and $j.texte -notmatch '[\u2013\u2014]') "texte de la bulle : « $($j.texte) »"
            Verifier ($j.id -cmatch '^[0-9a-f]{12}$') "numéro de la bulle : $($j.id) (12 chiffres hexadécimaux)"
            Verifier ((Compter "Bulle envoyée ($($j.id))") -eq 1) 'bulle envoyée une fois (journal)'
            # Le clic sur la bulle : Mindset écrit « notif:<id> » dans afficher.demande
            [void](Attendre { (Compter 'Écran prêt') -gt $prets } 15000)
            [void](Attendre { $false } 2500)   # (la page dessinée)
            [IO.File]::WriteAllText($afficher, "notif:$($j.id)")
            Verifier (Attendre { -not (Test-Path -LiteralPath $afficher) -and (Compter "Bulle ouverte : $($j.id)") -ge 1 } 4000) 'clic sur la bulle : afficher.demande lu et supprimé, la bulle reconnue (journal)'
            Verifier (Attendre { (Compter 'Bulle : fiche a1 ouverte') -ge 1 } 3000) 'clic sur la bulle : la fiche de Netflix est ouverte (journal)'
            Verifier ([MSIntegration]::EstEnfantDe($b.Fenetre, $panneau.Handle)) 'clic sur la bulle : elle reste accrochée dans Mindset'
            [void](Attendre { $false } 1500)   # (la fiche ouverte, son animation finie)
            Verifier (Save-Capture 'capture-bulle-fiche.png') 'capture : la fiche ouverte par le clic'
            [IO.File]::WriteAllText($afficher, 'notif:000000000000')
            Verifier (Attendre { -not (Test-Path -LiteralPath $afficher) -and (Compter 'Bulle inconnue ignorée (000000000000)') -ge 1 } 4000) 'une bulle qu''elle n''a pas donnée : ignorée'
        }
        $c = Get-Carte
        $lignes = @($c.Lignes | ForEach-Object { $_.Texte })
        Verifier ($lignes[0] -ceq "13,49$nbsp€ par mois" -and $lignes[1] -ceq "Netflix aujourd'hui" -and $lignes[2] -ceq '1 abonnement') "carte de Mindset : $($lignes -join ' | ')"
        Verifier (-not (Read-Bulle 8000)) 'aucune seconde bulle (8 s)'
        Stop-ParMindset $b 'Relancée'
        Verifier (-not (Test-Path -LiteralPath (Join-Path $donnees 'quitter.demande'))) 'quitter.demande effacé'
        Verifier ((Compter 'Fermeture demandée par Mindset') -ge 1) 'fermeture demandée par Mindset (journal)'
    }

    # --- 8. Trois rappels en même temps : une seule bulle, le portefeuille ; puis plus rien --------------------------
    Set-Abonnements @(
        (New-Abonnement 'a1' 'Netflix' 1349 0 'veille'),
        (New-Abonnement 'a2' 'Spotify' 1112 1 '3-jours'),
        (New-Abonnement 'a3' 'Box' 2999 5 '7-jours'))
    Set-DerniereVerification 3
    $prets = Compter 'Écran prêt'
    $g = Open-DansMindset 'Trois rappels'
    if ($g) {
        $bulle = Read-Bulle 8000
        Verifier ($null -ne $bulle -and $bulle.Json.titre -ceq '3 rappels de paiement') "trois rappels : une seule bulle « $($bulle.Json.titre) »"
        if ($bulle) {
            Verifier ($bulle.Json.texte -clike 'Netflix aujourd''hui (13,49*), Spotify demain (11,12*), Box dans 5 jours (29,99*).') "texte de la bulle : « $($bulle.Json.texte) »"
            [void](Attendre { (Compter 'Écran prêt') -gt $prets } 15000)
            [void](Attendre { $false } 2500)
            [IO.File]::WriteAllText($afficher, "notif:$($bulle.Json.id)")
            Verifier (Attendre { (Compter 'Bulle : portefeuille ouvert') -ge 1 } 4000) 'clic sur la bulle : le portefeuille est ouvert (journal)'
            [void](Attendre { $false } 1000)
            Verifier (Save-Capture 'capture-bulle-portefeuille.png') 'capture : le portefeuille ouvert par le clic'
        }
        Verifier (-not (Read-Bulle 7000)) 'une seule bulle pour les trois (7 s)'
        Stop-ParMindset $g 'Trois rappels'
    }
    $envoyees = Compter 'Bulle envoyée ('
    $r = Open-DansMindset 'Relancée sans rien changer'
    if ($r) {
        Verifier (-not (Read-Bulle 8000)) 'relancée : aucune bulle (jamais deux fois la même)'
        Verifier ((Compter 'Bulle envoyée (') -eq $envoyees -and $envoyees -eq 2) "deux bulles envoyées en tout (journal : $envoyees)"
        Stop-ParMindset $r 'Relancée sans rien changer'
    }
} catch {
    Echec "l'essai s'est arrêté : $($_.Exception.Message) (ligne $($_.InvocationInfo.ScriptLineNumber))"
} finally {
    $cadre.Close(); $cadre.Dispose()
    # --- 8. Plus aucun processus de l'essai ---------------------------------------------------------------------------
    [void](Attendre { $false } 2000)
    $restants = @(Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -and $_.ExecutablePath.StartsWith($racine, [StringComparison]::OrdinalIgnoreCase) })
    $restants += @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" | Where-Object { $_.CommandLine -like "*$racine*" })
    Verifier ($restants.Count -eq 0) "plus aucun processus de l'essai ($(@($restants | ForEach-Object { "$($_.Name) $($_.ProcessId)" }) -join ', '))"
    foreach ($r in $restants) { try { Stop-Process -Id $r.ProcessId -Force -ErrorAction Stop } catch {} }
}

foreach ($d in $horsEssai) {
    $maintenant = if (Test-Path -LiteralPath $d) { (Get-Item -LiteralPath $d).LastWriteTimeUtc } else { $null }
    Verifier ($maintenant -eq $avant[$d]) "rien de créé ni changé hors de l'essai : $d"
}

Get-Journal | Set-Content -LiteralPath (Join-Path $Resultats 'journal-dans-mindset.txt') -Encoding UTF8
Write-Output '--- Journal de l''essai ---'
Get-Journal | ForEach-Object { Write-Output "  $_" }
[void](Attendre { $false } 1000)
try { Remove-Item -LiteralPath $racine -Recurse -Force -ErrorAction Stop } catch { Write-Output "Dossier d'essai pas encore effacé : $racine" }

if ($echecs.Count) { Write-Output "$($echecs.Count) échec(s)."; exit 1 }
Write-Output 'Tout est bon.'
exit 0
