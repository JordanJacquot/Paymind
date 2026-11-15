# Paymind : le Paymind.zip d'une publication (outils\publier.ps1). Repris de Mindcare. Il n'est jamais construit sur ce
# PC : il vient de la construction GitHub de CE commit (.github\workflows\release.yml, job « Construire Paymind »), et
# n'est accepté que si son attestation GitHub dit qu'il a été construit par ce dépôt, par release.yml, à partir de ce
# commit (vérifier seulement qu'une attestation existe ne suffit pas). Sinon : erreur, et la publication est annulée
# avant toute signature. Sans ça, la signature de Jordan couvrirait un fichier qu'on aurait pu remplacer en route.
# Avant, les tests de GitHub de ce commit (.github\workflows\tests.yml : tests Rust, contrat de la suite avec le vrai
# Mindset, cargo deny...) doivent avoir réussi.
#
#   & '.\outils\zip-rust-github.ps1'             le chemin du Paymind.zip vérifié ; si GitHub ne l'a pas encore construit
#                                                pour ce commit, la construction est lancée (le commit doit déjà être
#                                                envoyé), puis attendue (40 minutes au plus), comme les tests
#   & '.\outils\zip-rust-github.ps1' -Essai      rien n'est lancé : seulement une construction déjà faite pour ce
#                                                commit ; s'il n'y en a pas, rien ($null)
#   -SansTest                                    n'attend pas les tests de GitHub (urgence seulement)
param([switch]$Essai, [switch]$SansTest)
$ErrorActionPreference = 'Stop'
$Depot = 'JordanJacquot/Paymind'
$Workflow = '.github/workflows/release.yml'
$Artefact = 'Paymind-rust'
$racine = Split-Path $PSScriptRoot -Parent

# gh et git : seul leur code de sortie compte (leurs messages normaux passent par la sortie d'erreur)
function Invoke-Outil([string]$Exe, [string[]]$Arguments) {
    $ErrorActionPreference = 'Continue'
    $s = @(& $Exe @Arguments 2>&1 | ForEach-Object { "$_" })
    if ($LASTEXITCODE) { throw "Publication annulée : $Exe $($Arguments -join ' ') a échoué ($(($s | Select-Object -Last 3) -join ' '))" }
    $s
}

$commit = ((Invoke-Outil git @('-C', $racine, 'rev-parse', 'HEAD')) -join '').Trim()
if ($commit -notmatch '^[0-9a-f]{40}$') { throw "Publication annulée : commit illisible ($commit)." }

# Les exécutions d'un workflow pour ce commit (envoi ou lancement à la main), les plus récentes d'abord
function Get-Executions([string]$Fichier) {
    $l = (Invoke-Outil gh @('run', 'list', '--repo', $Depot, '--workflow', $Fichier, '--commit', $commit, '--limit', '20', '--json', 'databaseId,headSha,event,status,conclusion,createdAt,url')) -join "`n" | ConvertFrom-Json
    @($l | Where-Object { $_.headSha -eq $commit -and $_.event -in 'push', 'workflow_dispatch' } | Sort-Object createdAt -Descending)
}
# Une exécution réussie de release.yml a-t-elle déposé le zip ? (artefact pas encore expiré)
function Test-Artefact([string]$Id) {
    $noms = @(Invoke-Outil gh @('api', "repos/$Depot/actions/runs/$Id/artifacts", '--jq', '.artifacts[] | select(.expired == false) | .name'))
    $noms -ccontains $Artefact
}
# Le verdict des tests d'un commit : seule sa DERNIÈRE exécution compte. Un vert ancien ne couvre pas un rouge plus
# récent (audit du 08/11 : Nevermind ec176a4, vert à 17:08 puis rouge à 17:47). Une exécution relancée garde son
# numéro et prend la conclusion de sa dernière tentative. Rend : absent, en cours, rouge ou vert.
function Get-VerdictTests($Executions) {
    $t = @(@($Executions) | Where-Object { $_ } | Sort-Object createdAt -Descending)
    if (-not $t.Count) { return 'absent' }
    $d = $t[0]
    if ($d.status -ne 'completed') { return 'en cours' }
    if ($d.conclusion -ne 'success') { return 'rouge' }
    'vert'
}

# 1. Les tests de GitHub de ce commit (40 minutes au plus)
if (-not $SansTest) {
    $limite = (Get-Date).AddMinutes(40)
    while ($true) {
        $t = @(Get-Executions 'tests.yml')
        $v = Get-VerdictTests $t
        if ($v -eq 'vert') { Write-Host "Tests de GitHub du commit $commit : réussis à la dernière exécution ($($t[0].url))."; break }
        if ($Essai) {
            $etat = switch ($v) {
                'en cours' { 'en cours' }
                'rouge' { "ratés à la dernière exécution ($($t[0].conclusion))" }
                default { 'pas lancés (commit pas envoyé ?)' }
            }
            Write-Host "Essai : tests de GitHub du commit $commit $etat. Une vraie publication attendrait qu'ils réussissent." -ForegroundColor Yellow
            break
        }
        if ($v -eq 'rouge') { throw "Publication annulée : la dernière exécution des tests de GitHub du commit $commit a échoué ($($t[0].conclusion)) : $($t[0].url)" }
        if ((Get-Date) -gt $limite) { throw "Publication annulée : les tests de GitHub du commit $commit n'ont pas fini en 40 minutes." }
        Start-Sleep -Seconds 15
    }
}

# 2. La construction de Paymind (release.yml) de ce commit
function Find-Bonne {
    foreach ($r in @(Get-Executions 'release.yml' | Where-Object { $_.conclusion -eq 'success' })) {
        if (Test-Artefact ([string]$r.databaseId)) { return $r }
    }
    $null
}
$bonne = Find-Bonne
if (-not $bonne) {
    if ($Essai) {
        Write-Host "Essai : GitHub n'a pas encore construit Paymind pour le commit $commit." -ForegroundColor Yellow
        return $null
    }
    $branche = ((Invoke-Outil git @('-C', $racine, 'rev-parse', '--abbrev-ref', 'HEAD')) -join '').Trim()
    if ($branche -eq 'HEAD') { throw 'Publication annulée : aucune branche en cours (HEAD détaché).' }
    if (-not @(Get-Executions 'release.yml' | Where-Object { $_.status -ne 'completed' }).Count) {
        Write-Host "Construction de Paymind par GitHub (release.yml) pour le commit $commit..."
        $null = Invoke-Outil gh @('workflow', 'run', 'release.yml', '--repo', $Depot, '--ref', $branche)
    }
    $depuis = Get-Date
    $limite = $depuis.AddMinutes(40)
    $toutFini = $null
    while (-not $bonne) {
        if ((Get-Date) -gt $limite) { throw "Publication annulée : GitHub n'a pas fini de construire Paymind pour le commit $commit en 40 minutes." }
        Start-Sleep -Seconds 15
        $bonne = Find-Bonne
        $toutes = @(Get-Executions 'release.yml')
        # (une minute de grâce : la construction lancée à la main met quelques secondes à apparaître ; puis deux minutes
        # après la fin de toutes les constructions : GitHub montre leurs fichiers un peu après les avoir dites finies.
        # Leçon de Mindcare 0.91.0 : annulée alors que la construction avait réussi et déposé le zip.)
        $finies = -not @($toutes | Where-Object { $_.status -ne 'completed' }).Count
        if (-not $finies) { $toutFini = $null } elseif (-not $toutFini) { $toutFini = Get-Date }
        if (-not $bonne -and ((Get-Date) - $depuis).TotalSeconds -gt 60 -and $finies -and ((Get-Date) - $toutFini).TotalSeconds -gt 120) {
            $derniere = if ($toutes.Count) { "$($toutes[0].conclusion) : $($toutes[0].url)" } else { 'aucune construction trouvée' }
            throw "Publication annulée : la construction GitHub de Paymind n'a rien donné ($derniere)."
        }
    }
}

# 3. Le zip de cette construction : exactement Paymind.zip, attesté (ce dépôt, release.yml, ce commit)
$dossier = Join-Path ([IO.Path]::GetTempPath()) "paymind-rust-$PID"
if (Test-Path -LiteralPath $dossier) { Remove-Item -LiteralPath $dossier -Recurse -Force }
$null = Invoke-Outil gh @('run', 'download', [string]$bonne.databaseId, '--repo', $Depot, '--name', $Artefact, '--dir', $dossier)
$recus = @(Get-ChildItem -LiteralPath $dossier -Recurse -File | ForEach-Object { $_.FullName.Substring($dossier.Length + 1) })
if (($recus -join '|') -cne 'Paymind.zip') { throw "Publication annulée : l'artefact $Artefact de la construction $($bonne.url) contient « $($recus -join ', ') » au lieu de « Paymind.zip »." }
$zip = Join-Path $dossier 'Paymind.zip'
$ErrorActionPreference = 'Continue'
$verif = @(& gh attestation verify $zip --repo $Depot --signer-workflow "$Depot/$Workflow" --source-digest $commit --signer-digest $commit --deny-self-hosted-runners 2>&1 | ForEach-Object { "$_" })
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($code) { throw "Publication annulée : Paymind.zip téléchargé n'est pas attesté comme construit par $Workflow de $Depot à partir du commit $commit ($(($verif | Select-Object -Last 3) -join ' '))." }
Write-Host "Paymind.zip : construction $($bonne.url), attestation vérifiée (dépôt, release.yml, commit $commit), SHA-256 $((Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash)" -ForegroundColor Green
$zip
