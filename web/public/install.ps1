# Installs CP Lang on Windows: irm https://cp-lang.org/install.ps1 | iex
#
#   & ([scriptblock]::Create((irm https://cp-lang.org/install.ps1))) -Channel edge
#   .\install.ps1 [-Channel stable|edge] [-Version 0.3.0|edge-YYYY-MM-DD] [-NoModifyPath]
#
# stable (the default) is the newest v* release, beta while 0.x; until the
# first one exists it installs edge, the build of the newest green main.
#
# Downloads the release for this machine from GitHub, checks it against the
# sha256 in the release's latest.json, unpacks it into
# $env:CPL_HOME\toolchains\<toolchain>\ (CPL_HOME defaults to ~\.cpl), puts
# the launcher at $env:CPL_HOME\bin\cpl.exe and adds that folder to the user
# PATH. CPL_RELEASES_URL points it at a mirror.
param(
    [ValidateSet("stable", "edge")]
    [string]$Channel = "stable",
    [switch]$Edge,
    [string]$Version = "",
    [switch]$NoModifyPath
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Say($message) { Write-Host "cpl-install: $message" }
function Die($message) {
    [Console]::Error.WriteLine("cpl-install: error: $message")
    throw "cpl-install failed"
}

$releases = if ($env:CPL_RELEASES_URL) { $env:CPL_RELEASES_URL.TrimEnd("/") } else { "https://github.com/ryanbai1412/cp-lang/releases" }
$cplHome = if ($env:CPL_HOME) { $env:CPL_HOME } else { Join-Path $HOME ".cpl" }
$channel = if ($Edge) { "edge" } else { $Channel }

$target = $env:CPL_INSTALL_TARGET
if (-not $target) {
    if ($PSVersionTable.PSVersion.Major -ge 6 -and -not $IsWindows) {
        Die "install.ps1 is for Windows; elsewhere run: curl -fsSL https://cp-lang.org/install.sh | sh"
    }
    $target = "x86_64-pc-windows-msvc"
}

$tag = ""
if ($Version) {
    if ($Version -match '^(edge|edge-\d{4}-\d{2}-\d{2}|v\d+\.\d+\.\d+)$') { $tag = $Version }
    elseif ($Version -match '^\d+\.\d+\.\d+$') { $tag = "v$Version" }
    elseif ($Version -ne "stable") { Die "-Version takes 0.3.0, edge or edge-YYYY-MM-DD, not $Version" }
} elseif ($channel -eq "edge") {
    $tag = "edge"
}
$base = if ($tag) { "$releases/download/$tag" } else { "$releases/latest/download" }

[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
function Fetch($url, $dest) {
    if ($url.StartsWith("file://")) {
        Copy-Item -LiteralPath ([Uri]$url).LocalPath -Destination $dest
    } else {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $dest
    }
}

$tmp = Join-Path (Join-Path $cplHome "tmp") ("install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
    Say "fetching $base/latest.json"
    $manifestPath = Join-Path $tmp "latest.json"
    try { Fetch "$base/latest.json" $manifestPath } catch {
        # Until the first stable release, the stable channel installs edge.
        if ($tag) { Die "could not download $base/latest.json: $_" }
        $base = "$releases/download/edge"
        Say "no stable release yet; fetching $base/latest.json"
        try { Fetch "$base/latest.json" $manifestPath } catch { Die "could not download $base/latest.json: $_" }
    }
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    $toolchain = [string]$manifest.toolchain
    if ($toolchain -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') { Die "the release manifest names no valid toolchain" }
    $asset = $manifest.assets.$target
    if (-not $asset) { Die "$toolchain has no build for $target" }
    $file = [string]$asset.file
    if ($file -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') { Die "the release manifest names a bad file: $file" }

    $dest = Join-Path (Join-Path $cplHome "toolchains") $toolchain
    $exe = Join-Path (Join-Path $dest "bin") "cpl.exe"
    if (Test-Path -LiteralPath $exe) {
        Say "$toolchain is already installed"
    } else {
        Say "downloading $toolchain for $target"
        $archive = Join-Path $tmp $file
        try { Fetch "$base/$file" $archive } catch { Die "could not download $base/${file}: $_" }
        $want = ([string]$asset.sha256).ToLowerInvariant()
        $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
        if ($got -ne $want) { Die "$file does not match its checksum (expected $want, got $got); nothing was installed" }
        $unpack = Join-Path $tmp "unpack"
        Expand-Archive -LiteralPath $archive -DestinationPath $unpack
        if (-not (Test-Path -LiteralPath (Join-Path (Join-Path $unpack "bin") "cpl.exe"))) { Die "$file has no bin\cpl.exe" }
        Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $unpack "release.json")
        New-Item -ItemType Directory -Force -Path (Join-Path $cplHome "toolchains") | Out-Null
        if (Test-Path -LiteralPath $dest) { Remove-Item -Recurse -Force -LiteralPath $dest }
        Move-Item -LiteralPath $unpack -Destination $dest
    }

    $bin = Join-Path $cplHome "bin"
    New-Item -ItemType Directory -Force -Path $bin | Out-Null
    $launcher = Join-Path $bin "cpl.exe"
    if (Test-Path -LiteralPath $launcher) {
        $old = Join-Path $bin "cpl.old.exe"
        Remove-Item -Force -LiteralPath $old -ErrorAction SilentlyContinue
        Move-Item -Force -LiteralPath $launcher -Destination $old
    }
    Copy-Item -LiteralPath $exe -Destination $launcher

    $settingsPath = Join-Path $cplHome "settings.toml"
    $default = if ($Version) { $Version } else { $channel }
    $key = if ($default -eq "stable" -or $default -eq "edge") { $channel } else { "" }
    $lines = @()
    if (Test-Path -LiteralPath $settingsPath) {
        $lines = @(Get-Content -LiteralPath $settingsPath | Where-Object { $_ -notmatch '^default *=' -and (-not $key -or $_ -notmatch "^$key *=") })
    }
    if ($default -ne "stable") { $lines += "default = `"$default`"" }
    if ($key) { $lines += "$key = `"$toolchain`"" }
    [IO.File]::WriteAllText($settingsPath, (($lines | ForEach-Object { "$_`n" }) -join ""))

    Say "installed $toolchain into $dest"
    Say "launcher: $launcher"
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @($userPath -split ";" | Where-Object { $_ })
    if ($parts -contains $bin) {
        Say "run: cpl --version"
    } elseif ($NoModifyPath -or -not ($PSVersionTable.PSVersion.Major -lt 6 -or $IsWindows)) {
        Say "add $bin to your PATH, then run: cpl --version"
    } else {
        [Environment]::SetEnvironmentVariable("Path", (($parts + $bin) -join ";"), "User")
        $env:Path = "$bin;$env:Path"
        Say "added $bin to your user PATH; open a new terminal, then run: cpl --version"
    }
} finally {
    Remove-Item -Recurse -Force -LiteralPath $tmp -ErrorAction SilentlyContinue
}
