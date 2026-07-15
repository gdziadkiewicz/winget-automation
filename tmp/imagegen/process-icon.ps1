Add-Type -AssemblyName System.Drawing

$source = [System.Drawing.Bitmap]::FromFile((Resolve-Path '.\tmp\imagegen\tray-icon-source.png'))
$transparent = New-Object System.Drawing.Bitmap $source.Width, $source.Height, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)

for ($y = 0; $y -lt $source.Height; $y++) {
    for ($x = 0; $x -lt $source.Width; $x++) {
        $pixel = $source.GetPixel($x, $y)
        $dominance = $pixel.G - [Math]::Max($pixel.R, $pixel.B)
        $alpha = if ($dominance -le 70) { 255 } elseif ($dominance -ge 190) { 0 } else { [int](255 * (190 - $dominance) / 120) }
        $green = [Math]::Min($pixel.G, [Math]::Max($pixel.R, $pixel.B))
        $transparent.SetPixel($x, $y, [System.Drawing.Color]::FromArgb($alpha, $pixel.R, $green, $pixel.B))
    }
}

$final = New-Object System.Drawing.Bitmap 256, 256, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [System.Drawing.Graphics]::FromImage($final)
$graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
$graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
$graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$graphics.DrawImage($transparent, 0, 0, 256, 256)
$final.Save((Join-Path (Resolve-Path '.\assets') 'tray-icon.png'), [System.Drawing.Imaging.ImageFormat]::Png)

$graphics.Dispose()
$final.Dispose()
$transparent.Dispose()
$source.Dispose()
