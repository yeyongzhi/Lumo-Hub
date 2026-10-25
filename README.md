# Lumo-Hub

### 把日常工作，放回顺手的位置。

Lumo-Hub 是一款面向 Windows 的桌面效率工作台。它把常用应用、文件和网页入口，与便签、笔记和倒数日放在一起，帮你少花时间寻找工具，把更多注意力留给正在做的事。

**本地优先 · 快捷启动 · 自定义工作台 · 便签与笔记**

[产品介绍](https://yeyongzhi.github.io/Lumo-Hub/) · [功能与使用指南](https://yeyongzhi.github.io/Lumo-Hub/guide) · [更新日志](https://yeyongzhi.github.io/Lumo-Hub/changelog) · [下载 Windows 版](https://github.com/yeyongzhi/Lumo-Hub/releases/tag/v0.0.1-beta.5)

> 当前代码版本：**0.0.1-beta.5（测试版）**。测试版仍在完善中，首次使用或升级前请备份重要内容。

## 一个工作台，容纳常用日常

### 快速找到并打开

按 **Alt + Space** 呼出快捷启动，搜索应用、网页或文件名称，也支持拼音和别名。用方向键选择，按回车打开。常用项可以固定，应用库支持分组、收藏、排序和扫描导入。

### 按自己的习惯整理首页

添加或排列首页卡片，将时间、最近打开、便签、笔记、倒数日、天气和每日内容放在需要的位置。可选择简洁、工作或生活布局，并使用亮色或暗色主题。

### 记录灵感与重要日期

- **便签**：快速写下临时内容，可置顶；删除后可从回收站恢复。
- **Markdown 笔记**：整理较长内容，支持标签、预览和文件异常检查。
- **倒数日**：记录纪念日、目标日期和其他重要时刻。

### 随时可用，也能安心退出

系统托盘和全局快捷键让 Lumo-Hub 在需要时快速出现。关闭主窗口后可留在托盘继续工作，也可以从托盘菜单重新打开或退出程序。

### 内置工具，点击即用

快捷启动面板内置取色器与色板、文本和图片剪贴板历史、二维码生成与识别、文本工具箱、悬浮文本和时间戳工具。支持中文名称与英文关键词搜索，例如 `color picker`、`clipboard`、`qrcode`。工具以独立窗口打开，可置顶；色板和悬浮文本保存在本机。

## 下载与开始使用

前往 [GitHub Releases](https://github.com/yeyongzhi/Lumo-Hub/releases/tag/v0.0.1-beta.5) 下载 Windows x64 安装包，运行安装程序后即可开始使用。快捷启动默认快捷键为 `Alt + Space`，可以在设置中修改。

Beta 版本可通过 GitHub Release 安装包手动安装和更新，也可在“设置 → 关于”检查更新；在线更新需要对应版本的签名安装包和更新清单可用。更新内容请查看[更新日志](https://yeyongzhi.github.io/Lumo-Hub/changelog)。

## 数据与隐私

笔记、便签、倒数日、应用库和设置默认保存在本机，不需要注册账号。你可以在“设置 → 数据与备份”查看或迁移数据目录、导出备份、恢复备份和管理回收站。迁移或恢复前会先创建备份；普通卸载会保留用户数据。

天气、新闻等在线卡片需要网络连接。Lumo-Hub 不会自动把本地笔记、便签和应用库同步到云端；建议将导出的备份另存到其他磁盘或安全位置。需要彻底清除本地数据时，可使用开始菜单中的“卸载并清理数据”。

## 反馈与更新日志

如果遇到问题或有功能建议，请到 [GitHub Issues](https://github.com/yeyongzhi/Lumo-Hub/issues) 反馈，并附上应用版本、Windows 版本、复现步骤和预期结果。请勿在公开问题中上传包含个人信息的数据库或备份文件。

版本更新记录请查看[站点更新日志](https://yeyongzhi.github.io/Lumo-Hub/changelog)，其 Markdown 内容维护在 [`docs/changelog.md`](docs/changelog.md)。

## 开发简介与技术栈

前端使用 Vue 3、TypeScript、Pinia、Tailwind CSS 和 shadcn-vue；桌面运行时采用 Tauri 2 与 Rust，本地业务数据使用 SQLite，文档站使用 VitePress。

开发需要 Node.js 24、pnpm 10.4.1、Rust 稳定工具链，以及 Windows MSVC C++ 构建工具与 WebView2。

```sh
pnpm install --frozen-lockfile
pnpm tauri:dev
pnpm build
pnpm tauri:build
pnpm docs:build
```

安装包构建需要项目的 Tauri 更新签名配置。详细说明参见[开发说明书](docs/开发说明书.md)、[技术方案](docs/技术方案.md)和[安装说明](docs/install.md)。

## 联系作者

[作者 GitHub](https://github.com/yeyongzhi) · [问题与建议](https://github.com/yeyongzhi/Lumo-Hub/issues)
