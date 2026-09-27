# Offline assets (SPEC §6, §13): streaming zipformer small-ru (Vosk team, sherpa-onnx format), Silero VAD, Piper RU voice,
# Priler rustpotter wake models + voice packs, FitoDomik/Jarvis-Sound.
# Binaries never go to git: they live in GitHub Release `assets-v1`, sha256 in tools/assets.sha256.
#
#   pwsh tools/fetch-assets.ps1                    # download from Release (needs `gh auth`), verify, extract to assets/
#   pwsh tools/fetch-assets.ps1 -Publish           # rebuild from upstream, rewrite assets.sha256, upload to Release
#   pwsh tools/fetch-assets.ps1 -Publish -NoUpload # rebuild only
param([switch]$Publish, [switch]$NoUpload, [string]$Tag = 'assets-v1')
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Root = Split-Path $PSScriptRoot -Parent
$Out = Join-Path $Root 'assets'
$Cache = Join-Path $Out '.download'
$SumsFile = Join-Path $PSScriptRoot 'assets.sha256'

# file in release -> upstream source (URL, or git:<owner/repo>:<subdir>, zipped with a top folder = file base name)
$Upstream = [ordered]@{
    'sherpa-onnx-streaming-zipformer-small-ru-vosk-int8-2025-08-16.tar.bz2' = 'https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-small-ru-vosk-int8-2025-08-16.tar.bz2'
    'silero_vad.onnx'                       = 'https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx'
    'vits-piper-ru_RU-denis-medium.tar.bz2' = 'https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-ru_RU-denis-medium.tar.bz2'
    'rustpotter-jarvis.zip'                 = 'git:Priler/jarvis:resources/rustpotter'
    'voices-priler.zip'                     = 'git:Priler/jarvis:resources/sound/voices'
    'jarvis-sound.zip'                      = 'git:FitoDomik/Jarvis-Sound:.'
}

function Get-Sha([string]$Path) { (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLower() }

function Read-Sums {
    $sums = @{}
    foreach ($line in Get-Content $SumsFile) {
        $sha, $name = $line -split '\s+', 2
        if ($name) { $sums[$name] = $sha }
    }
    $sums
}

function Build-FromGit([string]$Name, [string]$Spec) {
    $null, $repo, $sub = $Spec -split ':', 3
    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("jarvis-src-" + [guid]::NewGuid())
    git clone --depth 1 --quiet "https://github.com/$repo" $tmp
    if ($LASTEXITCODE) { throw "git clone $repo failed" }
    $stage = Join-Path $tmp ('_stage/' + [IO.Path]::GetFileNameWithoutExtension($Name))
    New-Item -ItemType Directory -Force $stage | Out-Null
    Get-ChildItem (Join-Path $tmp $sub) -Force | Where-Object { $_.Name -notin '.git', '_stage' } |
        Copy-Item -Destination $stage -Recurse
    Compress-Archive -Path $stage -DestinationPath (Join-Path $Cache $Name) -Force
    Remove-Item $tmp -Recurse -Force
}

function Expand-Asset([string]$Name) {
    $file = Join-Path $Cache $Name
    if ($Name -like '*.zip') { Expand-Archive $file -DestinationPath $Out -Force }
    elseif ($Name -like '*.tar.bz2') { tar -xjf $file -C $Out; if ($LASTEXITCODE) { throw "tar $Name failed" } }
    else { Copy-Item $file $Out -Force }
}

New-Item -ItemType Directory -Force $Cache | Out-Null
Push-Location $Root
try {
    if ($Publish) {
        $lines = foreach ($name in $Upstream.Keys) {
            $src = $Upstream[$name]
            Write-Host "build $name"
            if ($src -like 'git:*') { Build-FromGit $name $src }
            else { Invoke-WebRequest $src -OutFile (Join-Path $Cache $name) -TimeoutSec 600 }
            "$(Get-Sha (Join-Path $Cache $name))  $name"
        }
        Set-Content $SumsFile $lines
        if (-not $NoUpload) {
            gh release view $Tag *> $null
            if ($LASTEXITCODE) { gh release create $Tag --prerelease --title $Tag --notes 'Offline assets. See tools/fetch-assets.ps1' }
            gh release upload $Tag ($Upstream.Keys | ForEach-Object { Join-Path $Cache $_ }) --clobber
            if ($LASTEXITCODE) { throw 'gh release upload failed' }
        }
    }
    else {
        $sums = Read-Sums
        foreach ($name in $sums.Keys) {
            $file = Join-Path $Cache $name
            if ((Test-Path $file) -and (Get-Sha $file) -eq $sums[$name]) { Write-Host "cached $name"; continue }
            Write-Host "download $name"
            gh release download $Tag -p $name -D $Cache --clobber
            if ($LASTEXITCODE) { throw "gh release download $name failed" }
            if ((Get-Sha $file) -ne $sums[$name]) { throw "sha256 mismatch: $name" }
        }
    }
    foreach ($name in (Read-Sums).Keys) { Expand-Asset $name }
    Write-Host "assets ready in $Out"
}
finally { Pop-Location }
