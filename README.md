# Lumo-Hub

Lumo-Hub 是一款本地优先的 Windows 桌面效率工作台。首版使用 Tauri 2、Rust、Vue 3、TypeScript、Tailwind CSS 和 shadcn-vue 组件方案。

## 项目文档

- [开发说明书](./开发说明书.md)：首版产品范围和验收要求。
- [技术方案](./技术方案.md)：前后端架构、SQLite、数据目录、备份、更新与 Windows 分发方案。

## 本地运行

需要 Node.js、npm、Rust stable、Windows C++ Build Tools 和 WebView2。依赖安装后：

```powershell
npm install
npm run tauri:dev
```

生成 Windows NSIS 安装程序：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Join-Path $env:USERPROFILE '.tauri\lumo-hub.key'
npm run tauri:build
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY
```

本机 updater 私钥保存在用户目录的 `.tauri\lumo-hub.key`，请单独安全备份，切勿提交到仓库。公钥已配置在 `src-tauri/tauri.conf.json`。构建产物位于 `src-tauri/target/release/bundle/nsis/`，包含安装包与对应 `.sig` 文件。正式发布前，用 `scripts/prepare-release.ps1 -Repository "你的账号/你的仓库"` 生成 `release/latest.json`，并将这三个文件上传到同一个 GitHub Release；还需把配置中的 `OWNER/REPO` 换成真实仓库地址。

`src-tauri/target/release/lumo-hub.exe` 是编译产物；日常安装请使用 `bundle/nsis/` 下的 `*-setup.exe`。旁边的 `.exe.sig` 用于 Tauri updater 校验更新包，并不是 Windows 的 Authenticode 代码签名。若程序启动即退出，可查看 `%LOCALAPPDATA%\Lumo-Hub\startup-error.log` 和 Windows 事件查看器中的应用程序错误；启动日志只在 Rust panic 时写入。

安装包默认以当前 Windows 用户安装。用户数据与程序安装位置分开：默认数据库在 `%APPDATA%\Lumo-Hub\app.db`，当前数据目录索引在 `%LOCALAPPDATA%\Lumo-Hub\bootstrap.json`。

## 首版功能

- 首页、倒数日、便签和 Markdown 笔记。
- 应用库：管理应用、网页、文件，支持自定义分组、收藏和扫描后勾选导入。
- 本地全局搜索、浅色/深色/跟随系统、七套强调色和全局字体大小。
- 数据目录迁移、备份和恢复。
- 关于页手动检查更新；更新签名已配置，GitHub Release 地址需在发布前配置。


## 快捷操作

- **Alt+Space** 唤出快捷启动面板，↑/↓ 选择、Enter 打开、Esc 收起。程序需保持运行，可最小化；在设置中更改快捷键。
- 搜索支持名称、路径、别名、拼音和首字母，收藏与常用项目优先。应用库编辑页可填写别名。
- 应用卡片支持右键菜单，工具栏“批量管理”可批量移组、收藏和移除。
- 设置页提供自动备份（默认每天一次、保留 7 份）及回收站。删除的便签、笔记和倒数日可恢复。
- 首页“编辑首页”后，通过模块右上角内容图标选择指定便签、笔记或倒数日，完成后保存。

## 文档站与自动部署

文档站使用 VitePress，直接引用本 README、开发说明书和技术方案。

```powershell
npm run docs:dev
npm run docs:build
npm run docs:preview
```

GitHub Actions 在每次推送到 `main` 时构建 `docs/.vitepress/dist` 并部署到 GitHub Pages，也支持手动触发。`dev` 用于开发，合并到 `main` 后发布文档。

首次启用：在仓库 Settings → Pages → Build and deployment → Source 中选择 **GitHub Actions**。站点地址为 https://yeyongzhi.github.io/Lumo-Hub/ 。文档部署不生成 Windows 安装包；安装包仍通过 `npm run tauri:build` 构建。
