# Génère l'icône de Paymind : une carte bancaire au trait, dans le style « glitch » des icônes de Nevermind et
# Evermind (repris de D:\Projets\Evermind\outils\icone.ps1) : carré arrondi sombre, lignes d'écran, la carte en blanc
# avec deux échos décalés cyan et magenta, et deux tranches décalées. (Pas de lettre : le M est déjà celui de Mindset.)
# La même carte est dessinée dans ui\logo.svg (l'écran de l'appli) et dans le champ « logo » de
# outils\entree-apps-rust.json (la cartouche de Mindset), sur une grille de 100 :
#   M 18,30 L 82,30 L 82,72 L 18,72 Z  (la carte)   M 18,43 L 82,43  (la bande)   M 28,59 L 44,59  (le numéro),
#   d'un trait rond d'épaisseur 8
#   rust\paymind-appli\icons\Paymind.ico   icône des programmes (Mindset.exe et Paymind.exe), livrée aussi dans le zip
#                                          (Paymind\fichiers\Paymind.ico : le logo de la console 3D de Mindset)
#   _test\apercu-icone.png                 rendu à plusieurs tailles, sur fond clair et sombre
#
#   & '.\outils\icone.ps1'
param(
    [string]$OutDir = (Join-Path (Split-Path $PSScriptRoot -Parent) 'rust\paymind-appli\icons'),
    [string]$Apercu = (Join-Path (Split-Path $PSScriptRoot -Parent) '_test\apercu-icone.png')
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Force -Path $OutDir, (Split-Path $Apercu -Parent) | Out-Null

function New-RoundedRect([float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
    $p = New-Object Drawing.Drawing2D.GraphicsPath
    $d = 2 * $r
    $p.AddArc($x, $y, $d, $d, 180, 90); $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $p.CloseFigure()
    $p
}
function Col([string]$Hex, [int]$Alpha = 255) { $c = [Drawing.ColorTranslator]::FromHtml($Hex); [Drawing.Color]::FromArgb($Alpha, $c.R, $c.G, $c.B) }

# La carte : un trait épais aux bouts et aux angles arrondis (le contour, la bande, le numéro), transformé en forme
# pleine, comme le E d'Evermind. En dessous de 32 px, l'espace entre le haut de la carte et la bande ferait une bouillie
# grise : il est rempli (une bande pleine, comme sur les petites icônes de carte).
function New-CartePath([float]$s) {
    $k = $s / 100
    $pt = { param($x, $y) [Drawing.PointF]::new([float]($x * $k), [float]($y * $k)) }
    $p = New-Object Drawing.Drawing2D.GraphicsPath
    $p.AddPolygon([Drawing.PointF[]]@((& $pt 18 30), (& $pt 82 30), (& $pt 82 72), (& $pt 18 72)))
    $p.StartFigure(); $p.AddLine((& $pt 18 43), (& $pt 82 43))
    $p.StartFigure(); $p.AddLine((& $pt 28 59), (& $pt 44 59))
    $pen = New-Object Drawing.Pen ([Drawing.Color]::Black), ([float]($s * 0.08))
    $pen.StartCap = 'Round'; $pen.EndCap = 'Round'; $pen.LineJoin = 'Round'
    $p.Widen($pen)
    if ($s -lt 32) {
        # (la bande pleine, ajoutée à la forme : en remplissage « Winding », les morceaux qui se recouvrent restent pleins)
        $p.FillMode = 'Winding'
        $bande = New-Object Drawing.Drawing2D.GraphicsPath
        $bande.AddRectangle([Drawing.RectangleF]::new([float]($s * 0.18), [float]($s * 0.30), [float]($s * 0.64), [float]($s * 0.13)))
        $r = New-Object Drawing.Region $p; $r.Union($bande)
        $p = New-Object Drawing.Drawing2D.GraphicsPath
        foreach ($sc in $r.GetRegionScans((New-Object Drawing.Drawing2D.Matrix))) { $p.AddRectangle($sc) }
    }
    $p
}
function Fill-Shifted($g, $path, $brush, [float]$dx) {
    $m = New-Object Drawing.Drawing2D.Matrix
    $m.Translate($dx, 0)
    $c = $path.Clone(); $c.Transform($m)
    $g.FillPath($brush, $c)
}

function New-IconBitmap([int]$s) {
    $bmp = New-Object Drawing.Bitmap $s, $s, ([Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'
    $g.Clear([Drawing.Color]::Transparent)

    # Fond : carré arrondi sombre, liseré discret
    $m = [math]::Max(0.5, $s * 0.03)
    $bg = New-RoundedRect $m $m ($s - 2 * $m) ($s - 2 * $m) ($s * 0.22)
    $bgBrush = New-Object Drawing.Drawing2D.LinearGradientBrush ([Drawing.PointF]::new(0, 0)), ([Drawing.PointF]::new(0, $s)), (Col '#1D1C2B'), (Col '#07070A')
    $g.FillPath($bgBrush, $bg)
    if ($s -ge 32) { $g.DrawPath((New-Object Drawing.Pen (Col '#FFFFFF' 50), ([float]($s / 128))), $bg) }
    $g.SetClip($bg)
    # Lignes d'écran (seulement en grand, invisibles en petit)
    if ($s -ge 64) { for ($y = 0.08; $y -lt 0.95; $y += 0.03) { $g.FillRectangle((New-Object Drawing.SolidBrush (Col '#FFFFFF' 12)), 0, [float]($s * $y), $s, [float][math]::Max(1, $s * 0.006)) } }

    # La carte et ses échos de couleur (au moins un pixel de décalage, même en 16 px)
    $e = New-CartePath $s
    $dx = [float][math]::Max(1, $s * 0.03)
    Fill-Shifted $g $e (New-Object Drawing.SolidBrush (Col '#00E5FF' 225)) (-$dx)
    Fill-Shifted $g $e (New-Object Drawing.SolidBrush (Col '#FF2EB5' 225)) $dx
    $g.FillPath([Drawing.Brushes]::White, $e)

    # Tranches décalées : l'effet « glitch » (à partir de 32 px, sinon c'est du bruit), sur la bande et sur le bas de la carte
    if ($s -ge 32) {
        foreach ($band in @(@(0.40, 0.05, 0.05), @(0.635, 0.035, -0.04))) {
            $r = [Drawing.RectangleF]::new(0, [float]($s * $band[0]), $s, [float][math]::Max(1, $s * $band[1]))
            $g.SetClip($r, 'Intersect')
            $g.FillRectangle($bgBrush, $r)
            Fill-Shifted $g $e (New-Object Drawing.SolidBrush (Col '#00E5FF')) ([float]($s * $band[2]))
            Fill-Shifted $g $e ([Drawing.Brushes]::White) ([float]($s * $band[2] * 0.6))
            $g.ResetClip(); $g.SetClip($bg)
        }
    }
    $g.ResetClip(); $g.Dispose()
    $bmp
}

function Save-Ico([string]$Path) {
    $sizes = 16, 20, 24, 32, 40, 48, 64, 128, 256
    $pngs = foreach ($s in $sizes) { $b = New-IconBitmap $s; $ms = New-Object IO.MemoryStream; $b.Save($ms, [Drawing.Imaging.ImageFormat]::Png); $b.Dispose(); , $ms.ToArray() }
    $out = New-Object IO.MemoryStream
    $bw = New-Object IO.BinaryWriter $out
    $bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$sizes.Count)
    $offset = 6 + 16 * $sizes.Count
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $s = $sizes[$i]
        $bw.Write([byte]$(if ($s -ge 256) { 0 } else { $s })); $bw.Write([byte]$(if ($s -ge 256) { 0 } else { $s }))
        $bw.Write([byte]0); $bw.Write([byte]0); $bw.Write([uint16]1); $bw.Write([uint16]32)
        $bw.Write([uint32]$pngs[$i].Length); $bw.Write([uint32]$offset)
        $offset += $pngs[$i].Length
    }
    foreach ($p in $pngs) { $bw.Write($p) }
    $bw.Flush()
    [IO.File]::WriteAllBytes($Path, $out.ToArray())
}

Save-Ico (Join-Path $OutDir 'Paymind.ico')
$prev = New-Object Drawing.Bitmap 600, 160
$g = [Drawing.Graphics]::FromImage($prev)
$g.Clear((Col '#F3F3F3')); $g.FillRectangle((New-Object Drawing.SolidBrush (Col '#202020')), 300, 0, 300, 160)
foreach ($x0 in 10, 310) { $x = $x0; foreach ($s in 128, 64, 32, 16) { $g.DrawImage((New-IconBitmap $s), $x, 10, $s, $s); $x += $s + 10 } }
$g.Dispose()
$prev.Save($Apercu, [Drawing.Imaging.ImageFormat]::Png)
"Icône créée : $(Join-Path $OutDir 'Paymind.ico') (aperçu : $Apercu)"
