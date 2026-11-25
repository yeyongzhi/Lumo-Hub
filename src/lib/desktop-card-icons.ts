import type { Component } from 'vue';
import {
  AppWindow,
  BookOpen,
  BriefcaseBusiness,
  FileText,
  Folder,
  Globe,
  Heart,
  Home,
  LayoutGrid,
  Star,
} from '@lucide/vue';

export const desktopCardIconOptions: { value: string; label: string; component: Component }[] = [
  { value: 'layout-grid', label: '网格', component: LayoutGrid },
  { value: 'folder', label: '文件夹', component: Folder },
  { value: 'file-text', label: '文件', component: FileText },
  { value: 'globe', label: '网络', component: Globe },
  { value: 'app-window', label: '应用', component: AppWindow },
  { value: 'heart', label: '收藏', component: Heart },
  { value: 'star', label: '星标', component: Star },
  { value: 'home', label: '主页', component: Home },
  { value: 'briefcase-business', label: '工作', component: BriefcaseBusiness },
  { value: 'book-open', label: '阅读', component: BookOpen },
];

export const desktopCardEmojiOptions = [
  '😀',
  '😎',
  '🤩',
  '🚀',
  '⭐',
  '🔥',
  '❤️',
  '💡',
  '🎯',
  '🎨',
  '📁',
  '📚',
  '📝',
  '💼',
  '🏠',
  '🎮',
  '🎵',
  '📷',
  '🌱',
  '☕',
  '🔧',
  '🧰',
  '💻',
  '📌',
  '🗂️',
  '📦',
  '🌈',
  '🍀',
  '🐱',
  '✨',
];

export function desktopCardIcon(value: string | null | undefined): Component | undefined {
  return desktopCardIconOptions.find((option) => option.value === value)?.component;
}
