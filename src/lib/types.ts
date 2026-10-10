export type PageId = 'home' | 'countdowns' | 'stickies' | 'notes' | 'library' | 'settings' | 'about'

export interface Note {
  id: string
  title: string
  content: string
  created_at: string
  updated_at: string
}

export interface Sticky {
  id: string
  title: string
  content: string
  created_at: string
  updated_at: string
}

export interface Countdown {
  id: string
  title: string
  description: string | null
  target_date: string
  repeat_type: 'once' | 'yearly'
  created_at: string
  updated_at: string
}

export type ResourceKind = 'app' | 'web' | 'file'

export interface Resource {
  id: string
  name: string
  kind: ResourceKind
  target: string
  group_id: string | null
  is_favorite: boolean
  icon_path: string | null
  icon_data: string | null
  aliases: string
  usage_count: number
  last_opened_at: string | null
  created_at: string
  updated_at: string
}

export interface ResourceGroup {
  id: string
  name: string
  sort_order: number
}

export interface ScanCandidate {
  name: string
  target: string
  kind: ResourceKind
  source: string
  icon_data: string | null
  installation_path: string | null
}

export type HomeWidgetKind = 'clock' | 'countdowns' | 'sticky' | 'note' | 'recent'
export type HomeWidgetSize = '1x1' | '2x1' | '1x2' | '2x2' | '4x1'
export interface HomeWidget { id: string; kind: HomeWidgetKind; size: HomeWidgetSize; selected_ids?: string[] }

export interface AppSettings {
  theme_mode: 'light' | 'dark' | 'system'
  accent: 'red' | 'orange' | 'yellow' | 'green' | 'blue' | 'indigo' | 'violet'
  font_size: number
  border_radius: number
  sidebar_width: number
  common_launchers: string[]
  welcome_completed: boolean
  home_layout: HomeWidget[]
  launcher: { enabled: boolean; shortcut: string; minimal_mode: boolean }
  auto_backup: { enabled: boolean; interval_hours: number; retain: number }
}

export interface BootstrapInfo {
  data_dir: string
  default_data_dir: string
}

export interface SearchResult {
  id: string
  title: string
  subtitle: string
  kind: 'note' | 'countdown' | 'resource'
}

export interface TrashItem { id: string; kind: 'note' | 'sticky' | 'countdown'; title: string; deleted_at: string }
export interface BackupStatus { last_backup: string | null; count: number; directory: string; error: string | null }
