# Lumo-Hub

Lumo-Hub 是一款本地优先的 Windows 桌面效率工作台，使用 Tauri 2、Rust、Vue 3、TypeScript、Tailwind CSS 和 shadcn-vue。当前准备发布版本为 **0.0.1-beta.1**（Beta 1）。

## 功能

- 可编辑的首页、倒数日、便签和 Markdown 笔记。
- 应用库：管理应用、网页和文件，支持分组、收藏、扫描并勾选导入、批量整理和失效路径定位。
- 快捷启动：全局快捷键唤出，支持拼音/别名搜索、方向键选择和快速打开。
- 本地全局搜索、主题和强调色设置。
- 数据目录迁移、手动与自动备份、恢复和回收站。
- 系统托盘：关闭主窗口后可留在托盘，支持从托盘重新打开、唤出快捷启动或退出程序。
- 关于页手动检查更新；更新包使用 Tauri updater 签名校验。

应用数据默认随正式安装目录保存，首次启动会导入可发现的旧数据；也可以在设置中迁移到其他目录。普通卸载保留用户数据；开始菜单提供单独的“卸载并清理数据”入口。开发预览使用独立的数据目录。

## 开发环境

- Windows 10/11
- Node.js 22
- pnpm
- Rust stable、Windows C++ Build Tools 和 WebView2

安装前端依赖并启动桌面开发预览：

```powershell
pnpm install --frozen-lockfile
pnpm run tauri:dev
```

常用命令：

```powershell
pnpm run build       # TypeScript 检查和前端生产构建
pnpm run tauri:build # Windows NSIS 安装包和 updater 签名产物
pnpm run docs:dev
pnpm run docs:build
```

## Beta 发布

版本遵循 SemVer 预发布编号：首个 Beta 使用 `0.0.1-beta.0`，后续 Beta 依次递增为 `0.0.1-beta.1`、`0.0.1-beta.2`，稳定发布时使用 `0.0.1`。前端、Tauri 和 Rust crate 的版本号需要保持一致。

发布前需要：

1. 确认工作区干净，提交待发布改动，并通过 `pnpm run build`、Rust 检查及 Windows 安装验证。
2. 使用与配置公钥匹配的 Tauri updater 私钥构建。私钥必须从安全备份或发布机密中恢复，不能新生成一把替代密钥，否则旧版本无法验证后续更新。
3. 在 PowerShell 设置私钥文件路径（有密码时再设置密码变量），运行构建：

   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY = Join-Path $env:USERPROFILE '.tauri\lumo-hub.key'
   pnpm run tauri:build
   Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY
   ```

4. 使用 `scripts/prepare-release.ps1 -Repository "yeyongzhi/Lumo-Hub"` 生成 `release/latest.json`，并将 NSIS 安装包、`.sig` 和 `latest.json` 一并附加到对应 GitHub Release。

当前 updater 地址使用 GitHub `releases/latest` 清单。GitHub 的 latest 通道面向非预发布 Release；Beta 版本可先通过 Release 安装包分发，应用内 Beta 更新通道需在后续配置专用清单地址后启用。updater `.sig` 是 Tauri 更新验签文件，不是 Windows Authenticode 代码签名。

安装包输出目录：`src-tauri/target/release/bundle/nsis/`。程序安装和升级不会主动清理用户数据；发布前仍应验证安装、升级、托盘退出和数据保留流程。

## 数据与故障排查

数据目录与数据格式见[开发说明书](./开发说明书.md)和[技术方案](./技术方案.md)。启动异常日志位于 `%LOCALAPPDATA%\Lumo-Hub\startup-error.log`；也可查看 Windows 事件查看器中的应用程序错误。

## 文档站

文档使用 VitePress，GitHub Actions 会在 `dev` 分支部署到 [GitHub Pages](https://yeyongzhi.github.io/Lumo-Hub/)。文档部署与 Windows 应用安装包发布相互独立。
