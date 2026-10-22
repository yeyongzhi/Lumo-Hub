import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'Lumo-Hub',
  description: 'Lumo-Hub 是一款本地优先的 Windows 桌面效率工作台，集中管理常用应用、快速记录与每日安排。',
  base: '/Lumo-Hub/',
  head: [
    ['meta', { name: 'theme-color', content: '#f6f8fc' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Lumo-Hub · 桌面效率工作台' }],
    ['meta', { property: 'og:description', content: '把日常工作，放回顺手的位置。' }]
  ],
  themeConfig: {
    nav: [
      { text: '产品功能', link: '/guide' },
      { text: '安装说明', link: '/install' },
      { text: '更新日志', link: '/changelog' },
      { text: '开发文档', items: [
        { text: '开发说明书', link: '/开发说明书' },
        { text: '技术方案', link: '/技术方案' }
      ] },
      { text: 'GitHub', link: 'https://github.com/yeyongzhi/Lumo-Hub' }
    ],
    sidebar: [
      { text: '产品', items: [
        { text: '产品功能', link: '/guide' },
      { text: '安装说明', link: '/install' },
        { text: '更新日志', link: '/changelog' }
      ] },
      { text: '开发文档', items: [
        { text: '开发说明书', link: '/开发说明书' },
        { text: '技术方案', link: '/技术方案' }
      ] }
    ],
    outline: { label: '本页目录', level: [2, 3] },
    socialLinks: [{ icon: 'github', link: 'https://github.com/yeyongzhi/Lumo-Hub' }],
    search: { provider: 'local' }
  },
  appearance: true,
  cleanUrls: true
})
