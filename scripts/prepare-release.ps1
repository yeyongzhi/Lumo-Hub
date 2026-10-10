param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$')]
    [string]$Repository,
    [string]$Notes = 'Lumo-Hub 新版本',
    [string]$Tag = ''
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$configPath = Join-Path $projectRoot 'src-tauri\tauri.conf.json'
$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
$version = [string]$config.version
if ([string]::IsNullOrWhiteSpace($Tag)) { $Tag = "v$version" }

$bundleDir = Join-Path $projectRoot 'src-tauri\target\release\bundle\nsis'
$installerName = "Lumo-Hub_${version}_x64-setup.exe"
$installerPath = Join-Path $bundleDir $installerName
$signaturePath = "$installerPath.sig"
if (!(Test-Path -LiteralPath $installerPath -PathType Leaf)) {
    throw "找不到安装包：$installerPath。请先执行 npm run tauri:build。"
}
if (!(Test-Path -LiteralPath $signaturePath -PathType Leaf)) {
    throw "找不到签名文件：$signaturePath。请检查 Tauri updater 签名配置。"
}
$signature = [System.IO.File]::ReadAllText($signaturePath).Trim()
if ([string]::IsNullOrWhiteSpace($signature)) { throw '签名文件为空。' }

$release = [ordered]@{
    version = $version
    notes = $Notes
    pub_date = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    platforms = @{
        'windows-x86_64' = @{
            url = "https://github.com/$Repository/releases/download/$([uri]::EscapeDataString($Tag))/$([uri]::EscapeDataString($installerName))"
            signature = $signature
        }
    }
}
$releaseDir = Join-Path $projectRoot 'release'
New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
$manifestPath = Join-Path $releaseDir 'latest.json'
$json = $release | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText($manifestPath, $json, [System.Text.UTF8Encoding]::new($false))
Write-Output "更新清单：$manifestPath"
Write-Output "安装包：$installerPath"
Write-Output "签名文件：$signaturePath"
