import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'Lumo-Hub',
  description: '本地优先的 Windows 桌面效率工作台',
  base: '/Lumo-Hub/',
  themeConfig: {
    nav: [{ text: '使用指南', link: '/guide' }],
    sidebar: [
      { text: '使用指南', link: '/guide' },
      { text: '开发说明书', link: '/开发说明书' },
      { text: '技术方案', link: '/技术方案' }
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/yeyongzhi/Lumo-Hub' }],
    search: { provider: 'local' }
  }
})
