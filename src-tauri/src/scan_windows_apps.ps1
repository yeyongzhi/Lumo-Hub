$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$shell = New-Object -ComObject Shell.Application
$folder = $shell.Namespace('shell:AppsFolder')
if ($null -eq $folder) { throw '无法读取 Windows 应用列表' }
$items = @(
    foreach ($item in $folder.Items()) {
        $id = [string]$item.ExtendedProperty('System.AppUserModel.ID')
        $target = [string]$item.ExtendedProperty('System.Link.TargetParsingPath')
        if (-not $target -and $item.Path -match '\.(exe|lnk)$') { $target = [string]$item.Path }
        if ($id -or $target) { [pscustomobject]@{name=[string]$item.Name; app_id=$id; target=$target} }
    }
    foreach ($item in Get-StartApps) {
        if ($item.AppID) { [pscustomobject]@{name=[string]$item.Name; app_id=[string]$item.AppID; target=''} }
    }
)
ConvertTo-Json -InputObject $items -Compress -Depth 3
