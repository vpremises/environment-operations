# Preserve notices for the exact registry dependencies linked into the helper.
$ErrorActionPreference = 'Stop'
$metadata = cargo +1.97 metadata --locked --format-version 1 | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
foreach ($package in $metadata.packages) {
    if ($null -eq $package.source) { continue }
    $directory = Split-Path $package.manifest_path
    $destination = "dist/licenses/$($package.name)-$($package.version)"
    New-Item -ItemType Directory -Force $destination | Out-Null
    "$($package.name) $($package.version): $($package.license)" | Out-File "$destination/metadata.txt"
    Get-ChildItem -Path $directory -File -Recurse | Where-Object {
        $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)([._-].*)?$'
    } | ForEach-Object {
        $relative = [IO.Path]::GetRelativePath($directory, $_.FullName)
        $target = Join-Path $destination $relative
        New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
        Copy-Item $_.FullName $target
    }
}
