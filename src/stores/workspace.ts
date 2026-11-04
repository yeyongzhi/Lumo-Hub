import { migrateLegacyStorage, writeLocal } from '@/lib/local-files';
import { normalizeNoteTags } from '@/lib/note-tags';
import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { api } from '@/lib/api';
import { applySettings } from '@/lib/theme';
import { defaultHomeLayout } from '@/lib/home-layout';
import type {
  AppSettings,
  BootstrapInfo,
  Countdown,
  Note,
  Resource,
  ResourceGroup,
  Sticky,
} from '@/lib/types';

const defaults: AppSettings = {
  product: {
    close_to_tray: true,
    autostart: false,
    reminder_hour: 9,
    reminders: {},
    pinned_stickies: [],
    note_tags: {},
    tag_catalog: [],
  },
  theme_mode: 'system',
  accent: 'blue',
  font_size: 14,
  border_radius: 16,
  sidebar_width: 240,
  common_launchers: [
    'computer',
    'recycle_bin',
    'explorer',
    'cmd',
    'powershell',
    'control_panel',
    'notepad',
    'calculator',
    'task_manager',
    'windows_settings',
  ],
  welcome_completed: false,
  home_layout: defaultHomeLayout(),
  launcher: { enabled: true, shortcut: 'Alt+Space', minimal_mode: false },
  auto_backup: { enabled: true, interval_hours: 24, retain: 7 },
};

export const useWorkspaceStore = defineStore('workspace', () => {
  const initialized = ref(false);
  const cleanUninstall = ref(false);
  const startupError = ref('');
  const recoveryError = ref('');
  const message = ref('');
  const undoAction = ref<null | (() => Promise<void>)>(null);
  const undoBusy = ref(false);
  const settings = ref<AppSettings>(structuredClone(defaults));
  const bootstrap = ref<BootstrapInfo>({ data_dir: '', default_data_dir: '' });
  const notes = ref<Note[]>([]);
  const noteFileIssues = ref<string[]>([]);
  const stickies = ref<Sticky[]>([]);
  const countdowns = ref<Countdown[]>([]);
  const resources = ref<Resource[]>([]);
  const groups = ref<ResourceGroup[]>([]);
  const searchTarget = ref<{
    kind: 'note' | 'sticky' | 'countdown' | 'resource';
    id: string;
  } | null>(null);
  const pendingSaves = new Set<() => Promise<void>>();
  watch(
    () => [
      settings.value.theme_mode,
      settings.value.accent,
      settings.value.font_size,
      settings.value.border_radius,
    ],
    () => applySettings(settings.value),
    { immediate: true },
  );

  let messageTimer: ReturnType<typeof setTimeout> | undefined;
  function notify(text: string) {
    undoAction.value = null;
    message.value = text;
    if (messageTimer) clearTimeout(messageTimer);
    messageTimer = setTimeout(() => {
      message.value = '';
      undoAction.value = null;
    }, 8000);
  }

  async function notifyDeleted(kind: 'note' | 'sticky' | 'countdown', ids: string[]) {
    notify('内容已移入回收站');
    try {
      const items = (await api.listTrash()).filter(
        (item) => item.kind === kind && ids.includes(item.original_id),
      );
      if (!items.length) return;
      undoAction.value = async () => {
        if (undoBusy.value) return;
        undoBusy.value = true;
        try {
          await flushPendingSaves();
          for (const item of items) await api.restoreTrash(item.id);
          if (kind === 'note') await reloadNotes();
          else if (kind === 'sticky') await reloadStickies();
          else await reloadCountdowns();
          notify('已撤销删除');
        } catch (error) {
          notify(`撤销未完成，请到回收站检查：${String(error)}`);
        } finally {
          undoBusy.value = false;
        }
      };
    } catch {
      /* Deletion succeeded; recovery remains available in the recycle bin. */
    }
  }

  async function initialize() {
    try {
      const result = await api.bootstrap();
      bootstrap.value = result.bootstrap;
      settings.value = { ...result.settings, product: normalizeNoteTags(result.settings.product) };
      cleanUninstall.value = cleanUninstall.value || result.clean_uninstall;
      if (result.recovery_error) {
        recoveryError.value = result.recovery_error;
        initialized.value = true;
        return;
      }
      await migrateLegacyStorage();
      await reload();
      initialized.value = true;
    } catch (error) {
      startupError.value = String(error);
      initialized.value = true;
    }
  }

  async function reload() {
    await settingChain;
    const [n, noteIssues, s, c, r, g] = await Promise.all([
      api.listNotes(),
      api.noteFileIssues(),
      api.listStickies(),
      api.listCountdowns(),
      api.listResources(),
      api.listGroups(),
    ]);
    const currentSettings = await api.bootstrap();
    settings.value = {
      ...currentSettings.settings,
      product: normalizeNoteTags(currentSettings.settings.product),
      ...Object.fromEntries(pendingSettings),
    };
    for (const key of confirmedSettings.keys()) {
      if (!pendingSettings.has(key))
        confirmedSettings.set(key, currentSettings.settings[key as keyof AppSettings]);
    }
    notes.value = n;
    noteFileIssues.value = noteIssues;
    stickies.value = s;
    countdowns.value = c;
    resources.value = r;
    groups.value = g;
  }

  async function reloadNotes() {
    await settingChain;
    const [items, issues, current] = await Promise.all([
      api.listNotes(),
      api.noteFileIssues(),
      api.bootstrap(),
    ]);
    notes.value = items;
    noteFileIssues.value = issues;
    if (!pendingSettings.has('product'))
      settings.value = { ...settings.value, product: normalizeNoteTags(current.settings.product) };
  }
  async function reloadStickies() {
    stickies.value = await api.listStickies();
  }
  async function reloadCountdowns() {
    countdowns.value = await api.listCountdowns();
  }
  async function reloadResources() {
    const [items, folders] = await Promise.all([api.listResources(), api.listGroups()]);
    resources.value = items;
    groups.value = folders;
  }
  async function reloadSettings() {
    await settingChain;
    const current = await api.bootstrap();
    settings.value = {
      ...current.settings,
      product: normalizeNoteTags(current.settings.product),
      ...Object.fromEntries(pendingSettings),
    };
  }

  async function recoverDataDirectory(path: string) {
    bootstrap.value = await api.recoverDataDirectory(path);
    const result = await api.bootstrap();
    settings.value = { ...result.settings, product: normalizeNoteTags(result.settings.product) };
    recoveryError.value = result.recovery_error ?? '';
    if (!recoveryError.value) await reload();
  }

  let settingChain: Promise<unknown> = Promise.resolve();
  const settingVersions = new Map<string, number>();
  const confirmedSettings = new Map<string, unknown>();
  const pendingSettings = new Map<string, unknown>();
  function setSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    if (!confirmedSettings.has(key)) confirmedSettings.set(key, settings.value[key]);
    const version = (settingVersions.get(key) ?? 0) + 1;
    settingVersions.set(key, version);
    pendingSettings.set(key, value);
    settings.value = { ...settings.value, [key]: value };
    const job = settingChain.then(async () => {
      try {
        const saved = await api.setSetting(key, value);
        confirmedSettings.set(key, saved[key]);
        // Other fields may have optimistic changes queued after this request.
        if (settingVersions.get(key) === version) {
          settings.value = { ...settings.value, [key]: saved[key] };
          pendingSettings.delete(key);
        }
      } catch (error) {
        if (settingVersions.get(key) === version) {
          settings.value = { ...settings.value, [key]: confirmedSettings.get(key) };
          pendingSettings.delete(key);
        }
        notify(`设置保存失败：${String(error)}`);
        throw error;
      }
    });
    settingChain = job.catch(() => {});
    return job;
  }

  async function completeWelcome() {
    try {
      settings.value = await api.completeWelcome();
    } catch (error) {
      notify(`无法保存欢迎流程：${String(error)}`);
    }
  }

  const unsavedClient = new Map<string, unknown>();
  function saveClient(key: string, value: unknown) {
    const snapshot = JSON.parse(JSON.stringify(value)) as unknown;
    unsavedClient.set(key, snapshot);
    const job = settingChain.then(async () => {
      await writeLocal(key, snapshot);
      settings.value = { ...settings.value, client: { ...settings.value.client, [key]: snapshot } };
      if (unsavedClient.get(key) === snapshot) unsavedClient.delete(key);
    });
    settingChain = job.catch(() => {});
    return job;
  }

  async function launchResource(id: string) {
    await api.launchResource(id);
    resources.value = await api.listResources();
  }

  function registerPendingSave(flush: () => Promise<void>) {
    pendingSaves.add(flush);
    return () => pendingSaves.delete(flush);
  }

  async function flushPendingSaves() {
    await settingChain;
    await Promise.all([...pendingSaves].map((flush) => flush()));
    await settingChain;
    await Promise.all([...unsavedClient].map(([key, value]) => saveClient(key, value)));
  }

  return {
    undoAction,
    undoBusy,
    notifyDeleted,
    initialized,
    cleanUninstall,
    startupError,
    recoveryError,
    message,
    settings,
    bootstrap,
    notes,
    noteFileIssues,
    stickies,
    countdowns,
    resources,
    groups,
    searchTarget,
    reloadNotes,
    reloadStickies,
    reloadCountdowns,
    reloadResources,
    reloadSettings,
    initialize,
    recoverDataDirectory,
    reload,
    notify,
    setSetting,
    saveClient,
    completeWelcome,
    launchResource,
    registerPendingSave,
    flushPendingSaves,
  };
});
