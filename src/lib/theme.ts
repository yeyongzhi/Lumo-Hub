import type { AppSettings } from './types'

const accents: Record<AppSettings['accent'], string> = {
  red: '0.62 0.22 25',
  orange: '0.60 0.18 48',
  yellow: '0.58 0.15 85',
  green: '0.56 0.16 151',
  blue: '0.56 0.18 250',
  indigo: '0.51 0.2 276',
  violet: '0.58 0.2 305',
}

export function applySettings(settings: AppSettings) {
  const root = document.documentElement
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  root.dataset.theme = settings.theme_mode === 'system' ? (prefersDark ? 'dark' : 'light') : settings.theme_mode
  root.dataset.themeMode = settings.theme_mode
  root.dataset.accent = settings.accent
  root.style.setProperty('--accent-color', accents[settings.accent])
  root.style.setProperty('--primary-foreground', 'oklch(1 0 0)')
  root.style.setProperty('--radius', `${settings.border_radius ?? 16}px`)
  root.style.fontSize = `${settings.font_size}px`
}

export function watchSystemTheme(getSettings: () => AppSettings) {
  const media = window.matchMedia('(prefers-color-scheme: dark)')
  const onChange = () => {
    const settings = getSettings()
    if (settings.theme_mode === 'system') applySettings(settings)
  }
  media.addEventListener('change', onChange)
  return () => media.removeEventListener('change', onChange)
}
