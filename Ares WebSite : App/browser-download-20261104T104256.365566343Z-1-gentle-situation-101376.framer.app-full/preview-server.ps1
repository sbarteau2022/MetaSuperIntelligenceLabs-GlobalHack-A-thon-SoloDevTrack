$port = 8123
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$mime = @{ ".html"="text/html; charset=utf-8"; ".js"="text/javascript"; ".css"="text/css"; ".png"="image/png"; ".jpg"="image/jpeg"; ".jpeg"="image/jpeg"; ".webp"="image/webp"; ".gif"="image/gif"; ".svg"="image/svg+xml"; ".woff2"="font/woff2"; ".woff"="font/woff"; ".ttf"="font/ttf"; ".mp4"="video/mp4"; ".webm"="video/webm"; ".json"="application/json"; ".wasm"="application/wasm"; ".ico"="image/x-icon" }
$l = New-Object System.Net.HttpListener
$l.Prefixes.Add("http://127.0.0.1:$port/")
$l.Start()
Start-Process "http://127.0.0.1:$port/"
Write-Host "Site is running at: http://127.0.0.1:$port/   (to stop: close this window)"
while ($l.IsListening) {
  $c = $l.GetContext()
  try {
    $p = [Uri]::UnescapeDataString($c.Request.Url.AbsolutePath)
    if ($p.EndsWith("/")) { $p += "index.html" }
    $f = Join-Path $root ($p -replace "/", "\")
    if (Test-Path $f -PathType Container) { $f = Join-Path $f "index.html" }
    $ff = [IO.Path]::GetFullPath($f)
    if (-not $ff.StartsWith($root) -or -not (Test-Path $ff -PathType Leaf)) {
      $c.Response.StatusCode = 404
      $b = [Text.Encoding]::UTF8.GetBytes("404 - not found")
    } else {
      $ext = [IO.Path]::GetExtension($ff).ToLower()
      $c.Response.ContentType = if ($mime[$ext]) { $mime[$ext] } else { "application/octet-stream" }
      $b = [IO.File]::ReadAllBytes($ff)
    }
    $c.Response.OutputStream.Write($b, 0, $b.Length)
  } catch {} finally { $c.Response.Close() }
}