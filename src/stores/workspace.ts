import { migrateLegacyStorage } from '@/lib/local-files'
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { api } from '@/lib/api'
import { applySettings } from '@/lib/theme'
import { defaultHomeLayout } from '@/lib/home-layout'
import type { AppSettings, BootstrapInfo, Countdown, Note, Resource, ResourceGroup, Sticky } from '@/lib/types'

const defaults: AppSettings = { product: {close_to_tray:true,autostart:false,reminder_hour:9,reminders:{},pinned_stickies:[],note_tags:{}}, theme_mode: 'system', accent: 'blue', font_size: 14, border_radius: 16, sidebar_width: 240, common_launchers: ["computer","recycle_bin","explorer","cmd","powershell","control_panel","notepad","calculator"], welcome_completed: false, home_layout: defaultHomeLayout(), launcher: { enabled: true, shortcut: "Alt+Space", minimal_mode: false }, auto_backup: { enabled: true, interval_hours: 24, retain: 7 } }

export const useWorkspaceStore = defineStore('workspace', () => {
  const initialized = ref(false)
  const cleanUninstall = ref(false)
  const startupError = ref('')
  const recoveryError = ref('')
  const message = ref('')
  const settings = ref<AppSettings>({ ...defaults })
  const bootstrap = ref<BootstrapInfo>({ data_dir: '', default_data_dir: '' })
  const notes = ref<Note[]>([])
  const stickies = ref<Sticky[]>([])
  const countdowns = ref<Countdown[]>([])
  const resources = ref<Resource[]>([])
  const groups = ref<ResourceGroup[]>([])
  const searchTarget = ref<{ kind: 'note' | 'sticky' | 'countdown' | 'resource'; id: string } | null>(null)
  const pendingSaves = new Set<() => Promise<void>>()
  watch(settings, value => applySettings(value), { deep: true, immediate: true })

  let messageTimer: ReturnType<typeof setTimeout> | undefined
  function notify(text: string) {
    message.value = text
    if (messageTimer) clearTimeout(messageTimer)
    messageTimer = setTimeout(() => { message.value = '' }, 3600)
  }

  async function initialize() {
    try {
      const result = await api.bootstrap()
      bootstrap.value = result.bootstrap
      settings.value = result.settings
      cleanUninstall.value = cleanUninstall.value || result.clean_uninstall
      if (result.recovery_error) {
        recoveryError.value = result.recovery_error
        initialized.value = true
        return
      }
      await migrateLegacyStorage();await reload()
      initialized.value = true
    } catch (error) {
      startupError.value = String(error)
      initialized.value = true
    }
  }

  async function reload() {
    const [n, s, c, r, g] = await Promise.all([
      api.listNotes(), api.listStickies(), api.listCountdowns(), api.listResources(), api.listGroups(),
    ])
    const currentSettings=await api.bootstrap();settings.value=currentSettings.settings;
    notes.value = n
    stickies.value = s
    countdowns.value = c
    resources.value = r
    groups.value = g
  }

  async function recoverDataDirectory(path: string) {
    bootstrap.value = await api.recoverDataDirectory(path)
    const result = await api.bootstrap()
    settings.value = result.settings
    recoveryError.value = result.recovery_error ?? ''
    if (!recoveryError.value) await reload()
  }

  async function setSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    const previous = settings.value[key]
    settings.value = { ...settings.value, [key]: value }
    try {
      settings.value = await api.setSetting(key, value)
    } catch (error) {
      settings.value = { ...settings.value, [key]: previous }
      notify(`设置保存失败：${String(error)}`)
      throw error
    }
  }

  async function completeWelcome() {
    try { settings.value = await api.completeWelcome() }
    catch (error) { notify(`无法保存欢迎流程：${String(error)}`) }
  }

  async function launchResource(id: string) {
    await api.launchResource(id)
    resources.value = await api.listResources()
  }

  function registerPendingSave(flush: () => Promise<void>) {
    pendingSaves.add(flush)
    return () => pendingSaves.delete(flush)
  }

  async function flushPendingSaves() {
    await Promise.all([...pendingSaves].map(flush => flush()))
  }

  return { initialized, cleanUninstall, startupError, recoveryError, message, settings, bootstrap, notes, stickies, countdowns, resources, groups, searchTarget, initialize, recoverDataDirectory, reload, notify, setSetting, completeWelcome, launchResource, registerPendingSave, flushPendingSaves }
})
