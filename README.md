# Lumo-Hub

**把日常工作，放回顺手的位置。**

Lumo-Hub 是一款本地优先的 Windows 桌面效率工作台。它把常用应用启动、快速记录和每日安排放进一个可自定义的空间，帮助你减少寻找工具和切换窗口的时间。

> 当前版本：**0.0.1-beta.1**。这是测试版，安装或升级前请备份重要内容。

[产品介绍与使用指南](https://yeyongzhi.github.io/Lumo-Hub/) · [更新日志](https://yeyongzhi.github.io/Lumo-Hub/changelog) · [GitHub Releases](https://github.com/yeyongzhi/Lumo-Hub/releases/latest) · [问题反馈](https://github.com/yeyongzhi/Lumo-Hub/issues)

## 产品功能

- **快捷启动**：按 `Alt + Space` 唤出面板，搜索应用、网页或文件；支持拼音和别名，按回车即可打开。
- **应用库**：管理启动项、分组、收藏与固定项；支持扫描本机常见应用并选择导入。
- **个性化首页**：组合日期时间、最近打开、便签、笔记、倒数日、天气和每日内容等卡片。
- **日常记录**：使用便签快速记录，用 Markdown 笔记整理内容，用倒数日跟踪重要日期。
- **桌面常驻**：系统托盘、全局快捷键、亮色与暗色主题，以及可调整的快捷启动面板。
- **数据保护**：支持手动与定期备份、恢复、回收站和数据目录迁移。

## 下载与安装

前往 [GitHub Releases](https://github.com/yeyongzhi/Lumo-Hub/releases/latest) 下载 Windows x64 安装包并运行。Beta 阶段以 Release 页面中的安装包手动安装和更新为准；应用内自动更新通道可能尚未启用。

普通卸载会保留用户数据。若需要连同数据一起清除，请使用开始菜单中的“卸载并清理数据”。建议把导出的备份另存到其他磁盘或安全位置。

## 数据与隐私

应用内容默认保存在本机，不需要注册账号。可在应用的“设置 → 数据与备份”查看当前数据目录、迁移数据、导出备份或恢复备份。迁移和恢复前会创建备份；删除的便签、笔记和倒数日可从回收站恢复。

桌面首页中的天气和新闻等在线卡片需要网络连接；你的笔记、便签、应用库和设置仍保存在本机。请勿在公开问题反馈中上传个人数据库或备份文件。

## 使用指南与文档

- [产品功能与使用指南](https://yeyongzhi.github.io/Lumo-Hub/guide)
- [版本更新日志](https://yeyongzhi.github.io/Lumo-Hub/changelog)
- [开发说明书](https://yeyongzhi.github.io/Lumo-Hub/开发说明书)
- [技术方案](https://yeyongzhi.github.io/Lumo-Hub/技术方案)

## 开发环境

- Windows 10/11
- Node.js 22 或兼容版本、pnpm 10
- Rust stable、Windows C++ Build Tools 和 WebView2

安装依赖并启动桌面开发预览：

```powershell
pnpm install --frozen-lockfile
pnpm run tauri:dev
```

常用命令：

```powershell
pnpm run build        # TypeScript 检查与前端构建
pnpm run tauri:build  # 构建 Windows NSIS 安装包
pnpm run docs:dev     # 本地预览产品和开发文档
pnpm run docs:build   # 构建 VitePress 站点
pnpm run docs:preview # 预览构建后的文档站
```

## 发布说明

版本遵循 SemVer 预发布编号：Beta 版本依次使用 `0.0.1-beta.0`、`0.0.1-beta.1`、`0.0.1-beta.2`，稳定版使用 `0.0.1`。前端、Tauri 和 Rust crate 的版本号应保持一致。

Windows updater 构建需要使用与应用内公钥匹配的 Tauri updater 私钥。私钥应从安全备份或发布机密中恢复，不能随意重新生成，否则旧版本将无法验证后续更新。当前 Beta 以 GitHub Release 安装包分发；启用应用内更新前，需确认 Release 清单、签名附件和更新通道配置。updater `.sig` 是更新验签文件，不是 Windows Authenticode 代码签名。

安装包输出目录：`src-tauri/target/release/bundle/nsis/`。发布前应验证安装、升级、数据保留、托盘退出及备份恢复流程。

## 故障排查与反馈

启动异常日志位于 `%LOCALAPPDATA%\Lumo-Hub\startup-error.log`；也可查看 Windows 事件查看器中的应用程序错误。提交问题时请提供应用版本、Windows 版本、复现步骤和预期结果，可在 [GitHub Issues](https://github.com/yeyongzhi/Lumo-Hub/issues) 反馈。

## 技术栈

Tauri 2、Rust、Vue 3、TypeScript、Vite、Tailwind CSS、shadcn-vue、SQLite 和 VitePress。

文档站托管在 GitHub Pages；只有将 dev 合并到 main 后，GitHub Actions 才会从 main 自动构建并部署产品介绍、使用指南、更新日志与开发文档。dev 分支用于开发和预览。
