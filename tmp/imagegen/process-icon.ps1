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

$sizes = @(16, 24, 32, 48, 64, 256)
$images = foreach ($size in $sizes) {
    $frame = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $frameGraphics = [System.Drawing.Graphics]::FromImage($frame)
    $frameGraphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $frameGraphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $frameGraphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $frameGraphics.DrawImage($final, 0, 0, $size, $size)
    $stream = New-Object System.IO.MemoryStream
    $frame.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $stream.ToArray()
    $stream.Dispose()
    $frameGraphics.Dispose()
    $frame.Dispose()
    ,$bytes
}

$iconPath = Join-Path (Resolve-Path '.\assets') 'tray-icon.ico'
$iconStream = [System.IO.File]::Create($iconPath)
$writer = New-Object System.IO.BinaryWriter $iconStream
$writer.Write([uint16]0)
$writer.Write([uint16]1)
$writer.Write([uint16]$images.Count)
$offset = 6 + (16 * $images.Count)
for ($index = 0; $index -lt $images.Count; $index++) {
    $size = $sizes[$index]
    $writer.Write([byte]$(if ($size -eq 256) { 0 } else { $size }))
    $writer.Write([byte]$(if ($size -eq 256) { 0 } else { $size }))
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]32)
    $writer.Write([uint32]$images[$index].Length)
    $writer.Write([uint32]$offset)
    $offset += $images[$index].Length
}
foreach ($image in $images) {
    $writer.Write($image)
}
$writer.Dispose()
$iconStream.Dispose()

$graphics.Dispose()
$final.Dispose()
$transparent.Dispose()
$source.Dispose()
