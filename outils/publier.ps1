# Publie une nouvelle version de Paymind sur GitHub, avec sa signature. Repris de la partie « Rust » de
# outils\publier.ps1 de Mindcare. Mindset la propose ensuite (il installe et met à jour Paymind à partir du zip signé
# de la Release).
#
#   & '.\outils\publier.ps1' -Version 0.1.0 -Notes "Ce qui change, en mots simples"
#   & '.\outils\publier.ps1' -Version 0.2.0 -Notes "..." -Preversion     version d'essai (préversion GitHub) : seuls
#                                                                       ceux qui ont coché « Recevoir les versions
#                                                                       d'essai » de Paymind dans Mindset la reçoivent
#   & '.\outils\publier.ps1' -Essai [-ZipLocal <Paymind.zip>]           essai à blanc : rien n'est modifié dans le
#                                                                       dépôt, envoyé ni publié (voir plus bas)
#
# Le zip n'est JAMAIS construit sur ce PC. Le numéro de version est écrit dans rust\paymind-appli\Cargo.toml (et
# Cargo.lock) par outils\version-rust.ps1, commité (« Paymind <version> (Rust) ») et envoyé ; GitHub construit alors
# Paymind à partir de ce commit (release.yml : cargo build --release --locked, cargo deny, zip vérifié, attestation),
# après ses tests (tests.yml, contrat de la suite compris). outils\zip-rust-github.ps1 récupère ce zip et vérifie son
# attestation (ce dépôt, release.yml, ce commit) ; outils\verifier-zip-rust.ps1 revérifie son contenu (fichiers
# exacts, version.json, fiches de version, pas de construction d'essai, aucun runtime C à part). Ensuite seulement :
# signature RSA avec la clé de la suite (Mindset\outils\signer.ps1 : sans signature, rien n'est publié), tag, Release.
#
# Avant tout : les fichiers de la suite (les .ps1 en UTF-8 avec BOM, aucun tiret long ni demi-cadratin : le même
# contrôle que tests.yml). Les notes de version ne citent jamais un thème perso (Mario, Pokémon, Fire Emblem, Dofus...).
# -SansTest : n'attend pas les tests de GitHub (urgence seulement).
#
# -Essai : les mêmes vérifications, sans rien écrire dans le dépôt (sauf release\, jamais suivi par git) ni rien
# envoyer sur GitHub : ni commit, ni envoi, ni tag, ni Release, ni construction lancée sur GitHub, ni la vraie clé.
# Le zip est celui que GitHub a déjà construit pour le commit en cours (seulement lu), ou -ZipLocal <Paymind.zip> (un
# zip fait sur ce PC par outils\zip-rust.ps1, sans attestation : seulement pour un essai). Il est signé avec une clé
# jetable.

param(
    [string]$Version = '',
    [string]$Notes = '',
    [switch]$Preversion,
    [switch]$SansTest,
    [switch]$Essai,
    [string]$ZipLocal = '',
    # le dépôt de Mindset (signer.ps1, verifier-imports.ps1) : par défaut à côté de celui-ci
    [string]$Mindset = ''
)
$ErrorActionPreference = 'Stop'
$Depot      = 'JordanJacquot/Paymind'
$racine     = Split-Path $PSScriptRoot -Parent
$zip        = Join-Path $racine 'release\Paymind.zip'
$mindset    = if ($Mindset) { $Mindset } else { Join-Path (Split-Path $racine -Parent) 'Mindset' }
$cargoAppli = Join-Path $racine 'rust\paymind-appli\Cargo.toml'
$cargoLock  = Join-Path $racine 'rust\Cargo.lock'

if ($ZipLocal -and -not $Essai) { throw '-ZipLocal ne sert qu''à un essai (-Essai) : une vraie publication prend toujours le zip construit par GitHub.' }
if (-not (Test-Path -LiteralPath $cargoAppli)) { throw "Paymind introuvable : $cargoAppli" }
if (-not (Test-Path -LiteralPath (Join-Path $mindset 'outils\verifier-imports.ps1'))) { throw "Dépôt de Mindset introuvable : $mindset (passe -Mindset <dossier>)." }

# La version du Cargo.toml de paymind-appli (ligne « version = "x.y.z" » de sa section [package])
function Get-VersionCargo {
    $section = ''
    foreach ($ligne in [IO.File]::ReadAllLines($cargoAppli, [Text.Encoding]::UTF8)) {
        $l = $ligne.Trim()
        if ($l.StartsWith('[')) { $section = $l; continue }
        if ($section -eq '[package]' -and $l -match '^version\s*=\s*"([^"]*)"') { return $Matches[1] }
    }
    $null
}

if ($Essai -and -not $Version) { $Version = Get-VersionCargo }
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Numéro de version invalide : $Version (exemple : 0.1.0)" }

# Les notes de version : jamais un thème perso (ils ne vont jamais sur GitHub), jamais un tiret long ni demi-cadratin
$themesPerso = @('Mario', 'Pokémon', 'Pokemon', 'Fire Emblem', 'Dofus')
$cites = @($themesPerso | Where-Object { $Notes.IndexOf($_, [StringComparison]::OrdinalIgnoreCase) -ge 0 })
if ($cites.Count) { throw "Publication annulée : les notes de version citent un thème perso ($($cites -join ', ')). Les thèmes perso ne vont jamais sur GitHub : enlève-les des notes." }
if ($Notes.IndexOfAny(([string][char]0x2013 + [char]0x2014).ToCharArray()) -ge 0) { throw 'Publication annulée : les notes de version ont un tiret long ou demi-cadratin. Mets un tiret simple ou une virgule.' }

# Un numéro déjà publié ne doit jamais être réutilisé (GitHub garde l'ancien zip en cache)
if (-not $Essai) {
    $ErrorActionPreference = 'Continue'
    & git -C $racine rev-parse -q --verify "refs/tags/v$Version" 2>$null | Out-Null; $tagExiste = ($LASTEXITCODE -eq 0)
    & git -C $racine ls-remote --exit-code --tags origin "refs/tags/v$Version" 2>$null | Out-Null; $tagDistant = ($LASTEXITCODE -eq 0)
    & gh release view "v$Version" --repo $Depot 2>$null | Out-Null; $releaseExiste = ($LASTEXITCODE -eq 0)
    $ErrorActionPreference = 'Stop'
    if ($tagExiste -or $tagDistant -or $releaseExiste) { throw "La version $Version existe déjà. Choisis un numéro plus grand." }
}

# Les fichiers de la suite : le même contrôle que l'étape « Fichiers de la suite » de tests.yml (.ps1 en UTF-8 avec
# BOM, aucun tiret long ni demi-cadratin), sur les fichiers que GitHub verra : ceux de git, plus les nouveaux pas
# encore ajoutés (jamais ceux que .gitignore écarte).
function Test-FichiersSuite {
    $avant = [Console]::OutputEncoding
    try {
        [Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
        $ErrorActionPreference = 'Continue'
        $liste = (& git -C $racine -c core.quotepath=false ls-files -z --cached --others --exclude-standard 2>$null) -join ''
        $code = $LASTEXITCODE
    } finally { [Console]::OutputEncoding = $avant; $ErrorActionPreference = 'Stop' }
    if ($code) { return @('git ls-files a échoué : impossible de lister les fichiers du dépôt') }
    $problemes = @()
    $tirets = ([string][char]0x2013 + [char]0x2014).ToCharArray()
    $textes = '.rs', '.ps1', '.json', '.toml', '.md', '.html', '.js', '.css', '.svg', '.yml'
    foreach ($rel in @($liste.Split([char]0) | Where-Object { $_ } | Sort-Object -Unique)) {
        $f = Join-Path $racine ($rel.Replace('/', '\'))
        if (-not (Test-Path -LiteralPath $f -PathType Leaf)) { continue }   # (effacé mais pas encore commité)
        $ext = [IO.Path]::GetExtension($f).ToLowerInvariant()
        if ($ext -eq '.ps1' -and $f -notmatch '\\(target|_test)\\') {
            $o = [IO.File]::ReadAllBytes($f)
            if (-not ($o.Length -ge 3 -and $o[0] -eq 0xEF -and $o[1] -eq 0xBB -and $o[2] -eq 0xBF)) { $problemes += "$rel : pas de BOM" }
        }
        if ($textes -contains $ext -and $f -notmatch '\\(target|_test|maquettes|three)\\') {
            if ([IO.File]::ReadAllText($f, [Text.Encoding]::UTF8).IndexOfAny($tirets) -ge 0) { $problemes += "$rel : tiret long ou demi-cadratin" }
        }
    }
    $problemes
}
$problemes = @(Test-FichiersSuite)
if ($problemes.Count) { $problemes | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }; throw 'Publication annulée : corrige les fichiers ci-dessus.' }
Write-Host 'Fichiers de la suite : .ps1 avec BOM, aucun tiret long ni demi-cadratin.'

# Un programme (git, gh) : seul son code de sortie compte. Ses avertissements (« LF will be replaced by CRLF »...) ne
# sont pas des échecs : sans « Continue », PowerShell les prendrait pour des erreurs et arrêterait la publication.
function Invoke-Native([string]$Exe, [string[]]$Arguments) {
    $ErrorActionPreference = 'Continue'
    & $Exe @Arguments 2>&1 | ForEach-Object { "$_" }
    if ($LASTEXITCODE) { throw "Échec : $Exe $($Arguments -join ' ')" }
}

# La préversion porte un titre qui le dit (Mindset la montre comme une version d'essai)
$titre = "Paymind $Version$(if ($Preversion) { ' (version d''essai)' })"

# 1. Tout est commité : la version publiée est celle de git (GitHub construit le commit, pas ce dossier). Une
# publication arrêtée en route laisse seulement le numéro dans Cargo.toml et Cargo.lock : remis comme dans git.
if (-not $Essai) {
    $avant = [Console]::OutputEncoding
    try { [Console]::OutputEncoding = New-Object Text.UTF8Encoding($false); $sale = @(& git -C $racine -c core.quotepath=false status --porcelain 2>$null | Where-Object { $_ }) }
    finally { [Console]::OutputEncoding = $avant }
    $siens = @('rust/paymind-appli/Cargo.toml', 'rust/Cargo.lock')
    if ($sale.Count -and -not @($sale | Where-Object { $_ -notmatch '^(M |MM| M) ' -or $siens -notcontains $_.Substring(3).Trim('"') }).Count) {
        Invoke-Native 'git' @('-C', $racine, 'reset', '-q', '--', $siens[0], $siens[1]) | Out-Null
        Invoke-Native 'git' @('-C', $racine, 'checkout', '--', $siens[0], $siens[1]) | Out-Null
        Write-Host 'Le numéro de version laissé par une publication arrêtée en route est remis comme dans git.'
        $sale = @()
    }
    if ($sale.Count) { throw "Publication annulée : des fichiers ne sont pas commités ($(($sale | Select-Object -First 5) -join ' ; ')). Commite-les d'abord : la version publiée est celle de git." }

    # 2. Le numéro de version : rust\paymind-appli\Cargo.toml (la seule source : version.json, Mindset.exe et
    # Paymind.exe le prennent là) et sa ligne dans Cargo.lock (sinon --locked refuse de construire). Commité tout de
    # suite et envoyé : GitHub construit ce commit. Une tentative arrêtée plus loin laisse ce commit ; la suivante avec
    # le même numéro n'a rien de nouveau à commiter et reprend la même construction.
    & (Join-Path $PSScriptRoot 'version-rust.ps1') -Version $Version
    if ((Get-VersionCargo) -cne $Version) { throw 'Publication annulée : le numéro de version n''a pas pu être écrit dans Cargo.toml.' }
    & git -C $racine diff --quiet -- $cargoAppli $cargoLock
    if ($LASTEXITCODE) {
        Invoke-Native git @('-C', $racine, 'add', '--', $cargoAppli, $cargoLock)
        Invoke-Native git @('-C', $racine, 'commit', '-m', "Paymind $Version (Rust$(if ($Preversion) { ', version d''essai' }))")
    }
    Invoke-Native git @('-C', $racine, 'push', 'origin', 'HEAD')
} elseif ((Get-VersionCargo) -cne $Version) {
    throw "Essai : -Version $Version, mais le Cargo.toml de paymind-appli dit $(Get-VersionCargo) (l'essai ne modifie rien : laisse -Version vide)."
}

# 3. Le zip construit et attesté par GitHub pour ce commit (ou, en essai seulement, un zip fait sur ce PC)
if ($ZipLocal) {
    Write-Host "Essai : zip fait sur ce PC ($ZipLocal), sans attestation. Une vraie publication prend celui de GitHub." -ForegroundColor Yellow
    $source = (Resolve-Path -LiteralPath $ZipLocal).Path
} else {
    $source = & (Join-Path $PSScriptRoot 'zip-rust-github.ps1') -Essai:$Essai -SansTest:$SansTest
    $source = @($source | Where-Object { $_ }) | Select-Object -Last 1
    if (-not $source) {
        if ($Essai) { Write-Host 'Essai arrêté : aucun zip construit par GitHub pour ce commit (ou passe -ZipLocal).' -ForegroundColor Yellow; return }
        throw 'Publication annulée : pas de zip construit par GitHub.'
    }
}
New-Item -ItemType Directory -Force -Path (Split-Path $zip -Parent) | Out-Null
Remove-Item -LiteralPath $zip, "$zip.sig" -ErrorAction SilentlyContinue   # (jamais l'ancien zip ni l'ancienne signature)
Copy-Item -LiteralPath $source -Destination $zip

# 4. Son contenu revérifié ici (fichiers exacts, version.json, fiches de version, pas de construction d'essai, aucun
# runtime C à part)
& (Join-Path $PSScriptRoot 'verifier-zip-rust.ps1') -Zip $zip -Version $Version -VerifierImports (Join-Path $mindset 'outils\verifier-imports.ps1')
if ($LASTEXITCODE) { throw 'Publication annulée : le zip de Paymind n''est pas conforme.' }

# 5. La signature (en essai : une clé jetable, jamais la vraie)
if ($Essai) {
    $cleEssai = New-Object Security.Cryptography.RSACryptoServiceProvider(3072); $cleEssai.PersistKeyInCsp = $false
    try { [IO.File]::WriteAllBytes("$zip.sig", $cleEssai.SignData([IO.File]::ReadAllBytes($zip), [Security.Cryptography.CryptoConfig]::MapNameToOID('SHA256'))) }
    finally { $cleEssai.Dispose() }
} else { & (Join-Path $mindset 'outils\signer.ps1') -Fichier $zip }
if (-not (Test-Path -LiteralPath "$zip.sig")) { throw 'Publication annulée : le zip n''a pas pu être signé.' }

$notesFinales = if ($Notes) { $Notes } else { "Paymind $Version" }
$fin = if ($Preversion) { '--prerelease' } else { '--latest' }
if ($Essai) {
    Write-Host "Essai fini : $zip (Paymind $Version, signé avec une clé jetable). Rien n'est publié." -ForegroundColor Green
    Write-Host "Une vraie publication ferait : tag v$Version, envoi, puis gh release create v$Version Paymind.zip Paymind.zip.sig --title `"$titre`" $fin."
    return
}

# 6. Tag et Release (le numéro est déjà commité et envoyé, à l'étape 2)
Invoke-Native git @('-C', $racine, 'tag', "v$Version")
Invoke-Native git @('-C', $racine, 'push', 'origin', "v$Version")
Invoke-Native gh @('release', 'create', "v$Version", $zip, "$zip.sig", '--repo', $Depot, '--title', $titre, '--notes', $notesFinales, $fin)
if ($Preversion) { "Paymind $Version (version d'essai) publiée." } else { "Paymind $Version publiée pour tout le monde." }
