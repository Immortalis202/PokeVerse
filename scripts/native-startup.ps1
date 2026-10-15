param([string]$Executable = "$PSScriptRoot\..\src-tauri\target\release\pokeverse.exe")
$ErrorActionPreference = 'Stop'
$taskWorkspace = (Resolve-Path -LiteralPath "$PSScriptRoot\..").Path
$taskExecutable = (Resolve-Path -LiteralPath $Executable).Path
$taskVersion = (Get-Content -LiteralPath (Join-Path $taskWorkspace 'package.json') -Raw | ConvertFrom-Json).version
$taskRoot = Join-Path $taskWorkspace "tmp\native-$taskVersion-$([Guid]::NewGuid())"
$taskProcesses = @()
try {
  foreach ($name in @('host', 'gm')) {
    $installation = Join-Path $taskRoot $name
    New-Item -ItemType Directory -Path (Join-Path $installation 'portable-data'), (Join-Path $installation 'resources') -Force | Out-Null
    Copy-Item -LiteralPath $taskExecutable -Destination (Join-Path $installation 'pokeverse.exe')
    Copy-Item -LiteralPath (Join-Path $taskWorkspace 'src-tauri\resources\catalog.db') -Destination (Join-Path $installation 'resources\catalog.db')
    $taskProcesses += Start-Process -FilePath (Join-Path $installation 'pokeverse.exe') -WorkingDirectory $installation -WindowStyle Hidden -PassThru
  }
  Start-Sleep -Seconds 8
  foreach ($process in $taskProcesses) {
    $process.Refresh()
    if ($process.HasExited) { throw "Isolated application exited with code $($process.ExitCode)" }
  }
  foreach ($name in @('host', 'gm')) {
    $profile = Join-Path $taskRoot "$name\portable-data"
    foreach ($file in @('library.db', 'campaigns.db', 'logs\app.log')) {
      if (!(Test-Path -LiteralPath (Join-Path $profile $file))) { throw "Missing $name startup file: $file" }
    }
  }
  Write-Output $taskRoot
} finally {
  foreach ($process in $taskProcesses) {
    $process.Refresh()
    if (!$process.HasExited) {
      $null = $process.CloseMainWindow()
      if (!$process.WaitForExit(3000)) { Stop-Process -Id $process.Id -Force }
    }
  }
}
