import { Palette, ClipboardList, QrCode, TextCursorInput, Pin, Clock } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'

export const builtinTools = [
  { id: 'color', title: '取色器 + 色板', aliases: '取色 配色 颜色 色板 color picker colorPicker palette quse yanse', description: '屏幕取色、颜色转换与收藏色板', icon: Palette },
  { id: 'clipboard', title: '剪贴板', aliases: '剪切板 复制 历史 clipboard copy paste history jiantieban', description: '本机文本与图片复制历史', icon: ClipboardList },
  { id: 'qrcode', title: '二维码工具', aliases: '二维码 二维码生成 识别 qr qrcode qrCode erweima', description: '生成二维码、识别图片二维码', icon: QrCode },
  { id: 'text', title: '文本工具箱', aliases: '文本 格式化 去重 JSON URL base64 text toolbox format wenben', description: '文本处理、JSON 格式化与编解码', icon: TextCursorInput },
  { id: 'floating', title: '悬浮文本', aliases: '悬浮 置顶 桌面 钉住 便签 floating text pin sticky note xuanfu', description: '自动保存、可钉在桌面的文本窗口', icon: Pin },
  { id: 'timestamp', title: '时间戳工具', aliases: '时间 日期 时间戳 timestamp unix epoch time shijianchuo', description: '秒 / 毫秒时间戳与日期互转', icon: Clock },
] as const
export function findTools(query: string) {
  const terms = query.trim().toLowerCase().split(/\s+/)
  return builtinTools.filter(t => terms.every(term => `${t.title} ${t.aliases}`.toLowerCase().includes(term)))
}
export const openTool = (id: string) => invoke<void>('open_builtin_tool', { id })
