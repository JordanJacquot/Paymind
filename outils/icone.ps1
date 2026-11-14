# Génère l'icône de Paymind : une carte bancaire verte (le logo de l'écran, ui\logo.svg), avec un écho cyan derrière,
# sur fond sombre, dans le style des icônes de Mindcare, Nevermind et Evermind. (Pas de lettre : le M est déjà celui de
# Mindset.) Repris de D:\Projets\Mindcare\outils\icone.ps1.
#   rust\paymind-appli\icons\Paymind.ico   icône des programmes (Mindset.exe et Paymind.exe)
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

function New-IconBitmap([int]$s) {
    $bmp = New-Object Drawing.Bitmap $s, $s, ([Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'
    $g.Clear([Drawing.Color]::Transparent)
    $m = [math]::Max(0.5, $s * 0.03)
    $bg = New-RoundedRect $m $m ($s - 2 * $m) ($s - 2 * $m) ($s * 0.22)
    $bgBrush = New-Object Drawing.Drawing2D.LinearGradientBrush ([Drawing.PointF]::new(0, 0)), ([Drawing.PointF]::new(0, $s)), (Col '#1D1C2B'), (Col '#07070A')
    $g.FillPath($bgBrush, $bg)
    if ($s -ge 32) { $g.DrawPath((New-Object Drawing.Pen (Col '#FFFFFF' 50), ([float]($s / 128))), $bg) }
    $g.SetClip($bg)
    # L'écho de la carte (cyan, penché) derrière
    $echo = New-RoundedRect ($s * 0.12) ($s * 0.26) ($s * 0.64) ($s * 0.42) ($s * 0.08)
    $rot = New-Object Drawing.Drawing2D.Matrix; $rot.RotateAt(-10, [Drawing.PointF]::new($s * 0.44, $s * 0.47))
    $echo.Transform($rot)
    $g.DrawPath((New-Object Drawing.Pen (Col '#00E5FF' 220), ([float][math]::Max(1, $s * 0.055))), $echo)
    # La carte (verte, la couleur de Paymind), sa bande et sa puce
    $carte = New-RoundedRect ($s * 0.22) ($s * 0.34) ($s * 0.66) ($s * 0.44) ($s * 0.08)
    $vert = New-Object Drawing.Drawing2D.LinearGradientBrush ([Drawing.PointF]::new($s * 0.22, $s * 0.34)), ([Drawing.PointF]::new($s * 0.88, $s * 0.78)), (Col '#22E07A'), (Col '#00C8E0')
    $g.FillPath($vert, $carte)
    if ($s -ge 24) {
        $g.FillRectangle((New-Object Drawing.SolidBrush (Col '#06140C' 140)), [float]($s * 0.22), [float]($s * 0.44), [float]($s * 0.66), [float]($s * 0.08))
    }
    $g.FillPath((New-Object Drawing.SolidBrush (Col '#06140C')), (New-RoundedRect ($s * 0.32) ($s * 0.62) ($s * 0.22) ([math]::Max(1, $s * 0.07)) ([math]::Max(0.5, $s * 0.035))))
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
