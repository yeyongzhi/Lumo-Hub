<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm';
import AboutPage from '@/views/AboutPage.vue';
import HomePage from '@/views/HomePage.vue';
import EnhancementSettings from '@/components/EnhancementSettings.vue';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Field, FieldGroup, FieldLabel, FieldDescription } from '@/components/ui/field';
import { nextTick, ref, watch } from 'vue';
import {
  ArrowLeft,
  CircleHelp,
  Database,
  Download,
  FolderCog,
  FolderOpen,
  Settings,
  Sun,
  Moon,
  Monitor,
  Upload,
  Keyboard,
  ShieldCheck,
  Trash2,
  AppWindow,
  LayoutGrid,
  LayoutDashboard,
  List,
} from '@lucide/vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Card, CardContent } from '@/components/ui/card';
import { useWorkspaceStore } from '@/stores/workspace';
import type { AppSettings } from '@/lib/types';
import { api } from '@/lib/api';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Checkbox } from '@/components/ui/checkbox';
import { Switch } from '@/components/ui/switch';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Slider } from '@/components/ui/slider';
import {
  autoDownloadEnabled,
  autoUpdateEnabled,
  updateInterval,
  updateIntervals,
} from '@/lib/update-preferences';
import {
  libraryColumnOptions,
  libraryColumns,
  libraryDefaultFilter,
  libraryIconSize,
} from '@/lib/library-preferences';

const emit = defineEmits<{ navigate: [page: 'home'] }>();
const store = useWorkspaceStore();
const libraryColumnDraft = ref(libraryColumns(store.settings.client));
let pendingLibraryColumnWrites = 0;
watch(
  () => store.settings.client?.['lumo-library-columns'],
  () => {
    if (!pendingLibraryColumnWrites)
      libraryColumnDraft.value = libraryColumns(store.settings.client);
  },
  { deep: true },
);
const busy = ref(false);
const section = ref('appearance');
async function selectSection(id: string) {
  if (id === section.value) return;
  try {
    await store.flushPendingSaves();
    section.value = id;
  } catch (error) {
    store.notify(`保存设置失败：${String(error)}`);
  }
}
const props = defineProps<{
  request?: { section: 'about' | 'home'; checkUpdate: boolean; id: number } | null;
}>();
const aboutPage = ref<InstanceType<typeof AboutPage> | null>(null);
watch(
  () => props.request,
  async (request) => {
    if (!request) return;
    section.value = request.section;
    await nextTick();
    if (request.section === 'about' && request.checkUpdate) void aboutPage.value?.checkUpdate();
  },
  { immediate: true },
);
const settingsGroups = [
  {
    label: '通用',
    items: [
      { id: 'appearance', label: '外观', icon: Settings },
      { id: 'app', label: '应用设置', icon: Settings },
    ],
  },
  {
    label: '功能',
    items: [
      { id: 'home', label: '首页', icon: LayoutDashboard },
      { id: 'library', label: '应用库', icon: AppWindow },
      { id: 'launcher', label: '快捷启动', icon: Keyboard },
      { id: 'desktop-cards', label: '桌面卡片', icon: LayoutGrid },
    ],
  },
  {
    label: '数据',
    items: [
      { id: 'data', label: '数据与备份', icon: Database },
      { id: 'backup', label: '自动备份', icon: ShieldCheck },
      { id: 'trash', label: '回收站', icon: Trash2 },
    ],
  },
  { label: '其他', items: [{ id: 'about', label: '关于', icon: CircleHelp }] },
];
const fontSizes = [12, 14, 16, 18, 20, 22, 24];
const radii = [
  { value: 0, label: '直角' },
  { value: 4, label: '微圆角' },
  { value: 8, label: '小圆角' },
  { value: 12, label: '中圆角' },
  { value: 16, label: '大圆角（默认）' },
  { value: 24, label: '超大圆角' },
];
const modes = [
  { id: 'light', label: '浅色', icon: Sun },
  { id: 'dark', label: '深色', icon: Moon },
  { id: 'system', label: '跟随系统', icon: Monitor },
] as const;
const appearanceModes = [
  { id: 'solid', label: '纯色背景' },
  { id: 'glass', label: '毛玻璃' },
] as const;
const accents = [
  { id: 'brand', label: '品牌色', className: 'bg-[#007afc]' },
  { id: 'red', label: '红', className: 'bg-red-500' },
  { id: 'orange', label: '橙', className: 'bg-orange-500' },
  { id: 'yellow', label: '黄', className: 'bg-yellow-400' },
  { id: 'green', label: '绿', className: 'bg-green-500' },
  { id: 'blue', label: '蓝', className: 'bg-blue-500' },
  { id: 'indigo', label: '靛', className: 'bg-indigo-500' },
  { id: 'violet', label: '紫', className: 'bg-violet-500' },
] as const;
async function setMode(mode: AppSettings['theme_mode']) {
  try {
    await store.setSetting('theme_mode', mode);
  } catch {
    /* Best-effort platform operation; no state change on failure. */
  }
}
function updateThemeMode(value: string | number) {
  if (value === 'light' || value === 'dark' || value === 'system') void setMode(value);
}
function updateAppearanceMode(value: string | number) {
  if (value === 'solid' || value === 'glass') {
    void store.setSetting('appearance_mode', value).catch(() => {});
  }
}
async function setAccent(accent: AppSettings['accent']) {
  try {
    await store.setSetting('accent', accent);
  } catch {
    /* Best-effort platform operation; no state change on failure. */
  }
}
async function setFont(value: unknown) {
  try {
    await store.setSetting('font_size', Number(value));
  } catch {
    /* Best-effort platform operation; no state change on failure. */
  }
}
async function setRadius(value: unknown) {
  try {
    await store.setSetting('border_radius', Number(value));
  } catch {
    /* Best-effort platform operation; no state change on failure. */
  }
}
async function setLibraryDisplay(value: unknown) {
  if (value !== 'grid' && value !== 'list') return;
  try {
    await store.saveClient('lumo-library-display', value);
  } catch (e) {
    store.notify(`保存应用库设置失败：${String(e)}`);
  }
}
async function setUpdatePreference(key: string, value: boolean | number) {
  try {
    await store.saveClient(key, value);
  } catch (error) {
    store.notify(`保存更新设置失败：${String(error)}`);
  }
}
function setUpdateInterval(value: unknown) {
  const minutes = Number(value);
  if (updateIntervals.some((interval) => interval === minutes))
    void setUpdatePreference('lumo-update-check-interval', minutes);
}
async function setCloseBehavior(value: unknown) {
  if (value !== 'tray' && value !== 'quit') return;
  try {
    await store.setSetting('product', {
      ...store.settings.product,
      close_to_tray: value === 'tray',
    });
  } catch {
    // The store reports save failures and restores the previous setting.
  }
}
async function updateDesktopCardsSettings(patch: Partial<AppSettings['desktop_cards']>) {
  try {
    await store.setSetting('desktop_cards', { ...store.settings.desktop_cards, ...patch });
    if (patch.enabled) await store.reloadCards();
  } catch (error) {
    store.notify(`保存桌面卡片设置失败：${String(error)}`);
  }
}
function setDesktopCards(value: boolean) {
  void updateDesktopCardsSettings({ enabled: value });
}
function setDesktopCardsAutoFit(value: boolean) {
  void updateDesktopCardsSettings({ auto_fit: value });
}
function setDesktopCardsAdaptiveHeight(value: boolean) {
  void updateDesktopCardsSettings({ adaptive_height: value });
}
function setDesktopCardsAllowOverlap(value: boolean) {
  void updateDesktopCardsSettings({ allow_overlap: value });
}
function setDesktopCardView(value: string | number) {
  if (value === 'grid' || value === 'list') void updateDesktopCardsSettings({ item_view: value });
}
function setDesktopCardIconSize(value: string | number) {
  if (value === 'small' || value === 'medium' || value === 'large')
    void updateDesktopCardsSettings({ item_icon_size: value });
}
async function openDesktopCardStorage() {
  try {
    const path = await api.openDesktopCardStorageFolder();
    store.notify(`桌面卡片文件夹：${path}`);
  } catch (error) {
    store.notify(`打开桌面卡片文件夹失败：${String(error)}`);
  }
}
async function setLibraryDensity(value: unknown) {
  if (value !== 'compact' && value !== 'standard' && value !== 'comfortable') return;
  try {
    await store.saveClient('lumo-library-density', value);
  } catch (e) {
    store.notify(`保存应用库设置失败：${String(e)}`);
  }
}
async function setLibraryDefaultFilter(value: unknown) {
  if (
    typeof value !== 'string' ||
    libraryDefaultFilter(
      { ...store.settings.client, 'lumo-library-default-filter': value },
      store.groups,
    ) !== value
  )
    return;
  try {
    await store.saveClient('lumo-library-default-filter', value);
  } catch (error) {
    store.notify(`保存应用库设置失败：${String(error)}`);
  }
}
async function setLibraryIconSize(value: unknown) {
  if (value !== 'small' && value !== 'standard' && value !== 'large') return;
  try {
    await store.saveClient('lumo-library-icon-size', value);
  } catch (error) {
    store.notify(`保存应用库设置失败：${String(error)}`);
  }
}
async function setLibraryGridNames(value: boolean) {
  try {
    await store.saveClient('lumo-library-grid-show-names', value);
  } catch (error) {
    store.notify(`保存应用库设置失败：${String(error)}`);
  }
}
async function setLibraryColumn(id: string, enabled: boolean) {
  const next = enabled
    ? [...new Set([...libraryColumnDraft.value, id])]
    : libraryColumnDraft.value.filter((column) => column !== id);
  libraryColumnDraft.value = next;
  pendingLibraryColumnWrites++;
  try {
    await store.saveClient('lumo-library-columns', next);
  } catch (error) {
    store.notify(`保存应用库设置失败：${String(error)}`);
  } finally {
    pendingLibraryColumnWrites--;
    if (!pendingLibraryColumnWrites)
      libraryColumnDraft.value = libraryColumns(store.settings.client);
  }
}
async function chooseDataPath() {
  const result = await open({ directory: true, multiple: false });
  if (typeof result !== 'string') return;
  if (
    !(await askConfirmation(
      `将全部设置、笔记、便签和其他数据迁移到：\n${result}\n\n目标目录需要为空；迁移前自动备份，原数据保留。继续吗？`,
    ))
  )
    return;
  busy.value = true;
  try {
    await store.flushPendingSaves();
    store.bootstrap = await api.moveDataDirectory(result);
    await store.reload();
    store.notify('数据迁移完成，后续使用新目录');
  } catch (e) {
    store.notify(`迁移失败：${String(e)}`);
  } finally {
    busy.value = false;
  }
}
async function createBackup() {
  const path = await save({
    defaultPath: `Lumo-Hub-备份-${new Date().toISOString().slice(0, 10)}.lumo`,
    filters: [{ name: 'Lumo-Hub 备份', extensions: ['lumo'] }],
  });
  if (!path) return;
  busy.value = true;
  try {
    await store.flushPendingSaves();
    await api.exportBackup(path);
    store.notify('备份已导出');
  } catch (e) {
    store.notify(`备份失败：${String(e)}`);
  } finally {
    busy.value = false;
  }
}
async function restoreBackup() {
  const path = await open({
    defaultPath: `${store.bootstrap.data_dir}/backups`,
    multiple: false,
    filters: [{ name: 'Lumo-Hub 备份', extensions: ['lumo'] }],
  });
  if (typeof path !== 'string') return;
  if (
    !(await askConfirmation(
      '恢复备份会替换当前笔记、便签、倒数日、应用库数据和外观设置。恢复前会自动创建当前数据备份。确认继续？',
    ))
  )
    return;
  busy.value = true;
  try {
    await store.flushPendingSaves();
    await api.restoreBackup(path);
    const current = await api.bootstrap();
    store.bootstrap = current.bootstrap;
    store.settings = current.settings;
    await store.reload();
    store.notify('备份已恢复');
  } catch (e) {
    store.notify(`恢复失败：${String(e)}`);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="flex min-h-0 min-w-0 flex-1 gap-4 overflow-hidden">
    <Card class="flex min-h-0 w-44 shrink-0 flex-col gap-0 overflow-hidden py-0">
      <ScrollArea class="min-h-0 flex-1">
        <nav class="space-y-4 p-2" aria-label="设置分类">
          <div v-for="group in settingsGroups" :key="group.label" class="space-y-1">
            <p class="px-3 pb-1 pt-2 text-xs font-medium text-muted-foreground">
              {{ group.label }}
            </p>
            <Button
              v-for="item in group.items"
              :key="item.id"
              variant="ghost"
              class="h-9 w-full justify-start gap-2 rounded-lg px-3 text-sm"
              :class="
                section === item.id
                  ? 'bg-primary/10 font-semibold text-primary'
                  : 'text-muted-foreground'
              "
              :aria-current="section === item.id ? 'page' : undefined"
              @click="selectSection(item.id)"
              ><component :is="item.icon" />{{ item.label }}</Button
            >
          </div>
        </nav>
      </ScrollArea>
      <div class="shrink-0 border-t border-border p-2">
        <Button
          variant="ghost"
          class="w-full justify-start gap-2"
          @click="emit('navigate', 'home')"
        >
          <ArrowLeft />返回首页
        </Button>
      </div>
    </Card>
    <Card class="min-h-0 min-w-0 flex-1 gap-0 overflow-hidden py-0" data-settings-panel>
      <CardContent
        v-if="section === 'home'"
        class="flex min-h-0 flex-1 flex-col overflow-hidden py-6"
      >
        <HomePage settings-mode />
      </CardContent>
      <CardContent
        v-else-if="section === 'trash'"
        class="flex min-h-0 flex-1 flex-col overflow-hidden py-6"
        ><EnhancementSettings section="trash" @restore-backup="restoreBackup"
      /></CardContent>
      <ScrollArea v-else :key="section" class="min-h-0 flex-1" data-settings-scroll>
        <CardContent class="py-6">
          <div v-if="section === 'appearance'" class="flex min-w-0 flex-col gap-6">
            <div class="flex flex-col gap-7">
              <section>
                <h3 class="mb-3 text-base font-semibold">界面背景</h3>
                <Tabs
                  :model-value="store.settings.appearance_mode"
                  @update:model-value="updateAppearanceMode"
                >
                  <TabsList aria-label="界面背景">
                    <TabsTrigger v-for="mode in appearanceModes" :key="mode.id" :value="mode.id">
                      {{ mode.label }}
                    </TabsTrigger>
                  </TabsList>
                </Tabs>
              </section>
              <section>
                <h3 class="mb-3 text-base font-semibold">外观模式</h3>
                <Tabs
                  class="max-w-full"
                  :model-value="store.settings.theme_mode"
                  @update:model-value="updateThemeMode"
                  ><TabsList
                    ><TabsTrigger v-for="mode in modes" :key="mode.id" :value="mode.id"
                      ><component :is="mode.icon" />{{ mode.label }}</TabsTrigger
                    ></TabsList
                  ></Tabs
                >
              </section>
              <section>
                <h3 class="mb-3 text-base font-semibold">主题色</h3>
                <div class="flex flex-wrap gap-3">
                  <Button
                    v-for="accent in accents"
                    :key="accent.id"
                    variant="outline"
                    size="sm"
                    class="rounded-full"
                    :class="
                      store.settings.accent === accent.id
                        ? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
                        : ''
                    "
                    :aria-label="accent.id === 'brand' ? '品牌色主题' : `${accent.label}色主题`"
                    :aria-pressed="store.settings.accent === accent.id"
                    @click="setAccent(accent.id)"
                    ><span class="size-4 rounded-full" :class="accent.className"></span
                    >{{ accent.label }}</Button
                  >
                </div>
              </section>
              <FieldGroup>
                <Field
                  ><FieldLabel class="text-base font-semibold" for="global-radius"
                    >全局圆角</FieldLabel
                  ><Tabs
                    :model-value="String(store.settings.border_radius ?? 16)"
                    @update:model-value="setRadius"
                    ><TabsList
                      id="global-radius"
                      aria-label="全局圆角"
                      class="h-auto max-w-full flex-wrap justify-start"
                      ><TabsTrigger
                        v-for="option in radii"
                        :key="option.value"
                        :value="String(option.value)"
                        >{{ option.label }}</TabsTrigger
                      ></TabsList
                    ></Tabs
                  ><FieldDescription
                    >统一调整卡片、按钮、输入框和弹窗的圆角。</FieldDescription
                  ></Field
                >
                <Field>
                  <FieldLabel class="text-base font-semibold" for="global-font-size"
                    >全局字体大小</FieldLabel
                  >
                  <div class="flex w-full max-w-xl items-center gap-4">
                    <Slider
                      id="global-font-size"
                      aria-label="全局字体大小"
                      :model-value="[store.settings.font_size]"
                      :min="12"
                      :max="24"
                      :step="2"
                      class="flex-1"
                      @update:model-value="(value) => value?.[0] && setFont(value[0])"
                    />
                    <output class="w-10 text-right text-sm tabular-nums"
                      >{{ store.settings.font_size }}px</output
                    >
                  </div>
                  <div
                    class="flex w-full max-w-xl justify-between px-2 text-xs text-muted-foreground"
                    aria-hidden="true"
                  >
                    <span v-for="size in fontSizes" :key="size" class="text-center">
                      {{ size }}<span v-if="size === 14"> 默认</span>
                    </span>
                  </div>
                  <FieldDescription>拖动滑块调整字号，整个应用立即更新。</FieldDescription>
                </Field>
              </FieldGroup>
            </div>
          </div>
          <div v-else-if="section === 'library'" class="flex min-w-0 flex-col gap-6">
            <FieldGroup>
              <Field>
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold" for="library-default-filter"
                    >默认打开范围</FieldLabel
                  >
                  <FieldDescription
                    >进入应用库时显示指定分类或分组，也可以记住上次选择。</FieldDescription
                  >
                </div>
                <Select
                  :model-value="libraryDefaultFilter(store.settings.client, store.groups)"
                  @update:model-value="setLibraryDefaultFilter"
                >
                  <SelectTrigger id="library-default-filter" class="w-full max-w-xs"
                    ><SelectValue
                  /></SelectTrigger>
                  <SelectContent
                    ><SelectGroup>
                      <SelectItem value="all">全部</SelectItem>
                      <SelectItem value="favorites">收藏</SelectItem>
                      <SelectItem value="pinned">固定项</SelectItem>
                      <SelectItem value="common">系统启动项</SelectItem>
                      <SelectItem value="remember">记住上次选择</SelectItem>
                      <SelectItem
                        v-for="group in store.groups"
                        :key="group.id"
                        :value="`group:${group.id}`"
                        >分组 · {{ group.name }}</SelectItem
                      >
                    </SelectGroup></SelectContent
                  >
                </Select>
              </Field>
              <Field
                ><div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">显示方式</FieldLabel
                  ><FieldDescription>选择网格或列表布局。</FieldDescription>
                </div>
                <Tabs
                  class="max-w-fit"
                  :model-value="String(store.settings.client?.['lumo-library-display'] ?? 'grid')"
                  @update:model-value="setLibraryDisplay"
                  ><TabsList aria-label="应用库显示方式"
                    ><TabsTrigger value="grid"><LayoutGrid />网格</TabsTrigger
                    ><TabsTrigger value="list"><List />列表</TabsTrigger></TabsList
                  ></Tabs
                ></Field
              >
              <Field>
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">图标大小</FieldLabel>
                  <FieldDescription>调整应用库网格和列表中的启动项图标。</FieldDescription>
                </div>
                <Tabs
                  class="max-w-fit"
                  :model-value="libraryIconSize(store.settings.client)"
                  @update:model-value="setLibraryIconSize"
                >
                  <TabsList aria-label="应用库图标大小">
                    <TabsTrigger value="small">小</TabsTrigger>
                    <TabsTrigger value="standard">中（默认）</TabsTrigger>
                    <TabsTrigger value="large">大</TabsTrigger>
                  </TabsList>
                </Tabs>
              </Field>
              <Field>
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold" for="library-grid-names"
                    >网格显示应用名称</FieldLabel
                  >
                  <FieldDescription>关闭后网格仅显示图标；列表仍显示名称。</FieldDescription>
                </div>
                <div class="flex items-center">
                  <Switch
                    id="library-grid-names"
                    :model-value="store.settings.client?.['lumo-library-grid-show-names'] !== false"
                    @update:model-value="setLibraryGridNames"
                  />
                </div>
              </Field>
              <Field>
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">列表附加信息</FieldLabel>
                  <FieldDescription
                    >选择列表中显示的信息，与应用库的“显示信息”菜单同步。</FieldDescription
                  >
                </div>
                <div class="flex flex-wrap gap-x-6 gap-y-3">
                  <div
                    v-for="column in libraryColumnOptions"
                    :key="column.id"
                    class="flex items-center gap-2"
                  >
                    <Checkbox
                      :id="`library-column-${column.id}`"
                      :model-value="libraryColumnDraft.includes(column.id)"
                      @update:model-value="(value) => setLibraryColumn(column.id, value === true)"
                    />
                    <FieldLabel :for="`library-column-${column.id}`">{{ column.name }}</FieldLabel>
                  </div>
                </div>
              </Field>
              <Field
                ><div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold" for="library-density"
                    >网格密度</FieldLabel
                  ><FieldDescription>调整网格布局中每个启动项的占用空间。</FieldDescription>
                </div>
                <Select
                  :model-value="
                    String(store.settings.client?.['lumo-library-density'] ?? 'standard')
                  "
                  @update:model-value="setLibraryDensity"
                  ><SelectTrigger id="library-density" class="w-full max-w-xs"
                    ><SelectValue /></SelectTrigger
                  ><SelectContent
                    ><SelectGroup
                      ><SelectItem value="compact">紧凑</SelectItem
                      ><SelectItem value="standard">标准</SelectItem
                      ><SelectItem value="comfortable">宽松</SelectItem></SelectGroup
                    ></SelectContent
                  ></Select
                ></Field
              >
            </FieldGroup>
          </div>
          <div v-else-if="section === 'desktop-cards'" class="flex min-w-0 flex-col gap-6">
            <FieldGroup>
              <Field orientation="horizontal" class="items-start">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="desktop-cards-enabled" class="text-base font-semibold"
                    >启用桌面卡片</FieldLabel
                  >
                  <FieldDescription
                    >拖入的文件会移入桌面旁的专属文件夹，移除时移回；备份不含这些文件，关闭仅隐藏卡片。</FieldDescription
                  >
                </div>
                <div class="flex shrink-0 flex-col items-end gap-2">
                  <Switch
                    id="desktop-cards-enabled"
                    :model-value="store.settings.desktop_cards?.enabled === true"
                    @update:model-value="(value) => setDesktopCards(value)"
                  />
                  <Button variant="outline" size="sm" @click="openDesktopCardStorage"
                    ><FolderOpen data-icon="inline-start" />打开卡片文件夹</Button
                  >
                </div>
              </Field>
              <Field orientation="horizontal">
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">图标显示方式</FieldLabel>
                  <FieldDescription>选择桌面卡片条目的排列方式。</FieldDescription>
                </div>
                <Tabs
                  :model-value="store.settings.desktop_cards?.item_view ?? 'grid'"
                  @update:model-value="setDesktopCardView"
                >
                  <TabsList aria-label="图标显示方式"
                    ><TabsTrigger value="grid">网格</TabsTrigger
                    ><TabsTrigger value="list">列表</TabsTrigger></TabsList
                  >
                </Tabs>
              </Field>
              <Field
                v-if="store.settings.desktop_cards?.item_view !== 'list'"
                orientation="horizontal"
              >
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">图标显示大小</FieldLabel>
                  <FieldDescription>调整桌面卡片中条目的图标大小。</FieldDescription>
                </div>
                <Tabs
                  :model-value="store.settings.desktop_cards?.item_icon_size ?? 'medium'"
                  @update:model-value="setDesktopCardIconSize"
                >
                  <TabsList aria-label="图标显示大小"
                    ><TabsTrigger value="small">小</TabsTrigger
                    ><TabsTrigger value="medium">中</TabsTrigger
                    ><TabsTrigger value="large">大</TabsTrigger></TabsList
                  >
                </Tabs>
              </Field>
              <Field orientation="horizontal">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="desktop-cards-show-names" class="text-base font-semibold"
                    >显示图标名称</FieldLabel
                  >
                  <FieldDescription>在桌面卡片中显示条目名称。</FieldDescription>
                </div>
                <Switch
                  id="desktop-cards-show-names"
                  :model-value="store.settings.desktop_cards?.show_item_names !== false"
                  @update:model-value="
                    (value) => updateDesktopCardsSettings({ show_item_names: value })
                  "
                />
              </Field>
              <Field orientation="horizontal" class="items-start">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="desktop-cards-auto-fit" class="text-base font-semibold"
                    >内容区域自动伸缩</FieldLabel
                  >
                  <FieldDescription>鼠标移入标题栏时展开内容，移出卡片后收缩。</FieldDescription>
                </div>
                <Switch
                  id="desktop-cards-auto-fit"
                  :model-value="store.settings.desktop_cards?.auto_fit !== false"
                  :disabled="store.settings.desktop_cards?.enabled !== true"
                  @update:model-value="(value) => setDesktopCardsAutoFit(value)"
                />
              </Field>
              <Field orientation="horizontal" class="items-start">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="desktop-cards-adaptive-height" class="text-base font-semibold"
                    >自适应高度</FieldLabel
                  >
                  <FieldDescription>根据卡片中的图标数量自动调整内容区域高度。</FieldDescription>
                </div>
                <Switch
                  id="desktop-cards-adaptive-height"
                  :model-value="store.settings.desktop_cards?.adaptive_height !== false"
                  :disabled="store.settings.desktop_cards?.enabled !== true"
                  @update:model-value="(value) => setDesktopCardsAdaptiveHeight(value)"
                />
              </Field>
              <Field orientation="horizontal" class="items-start">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="desktop-cards-allow-overlap" class="text-base font-semibold"
                    >允许卡片重叠</FieldLabel
                  >
                  <FieldDescription>关闭后卡片会自动避让；空间不足时会提示。</FieldDescription>
                </div>
                <Switch
                  id="desktop-cards-allow-overlap"
                  :model-value="store.settings.desktop_cards?.allow_overlap !== false"
                  :disabled="store.settings.desktop_cards?.enabled !== true"
                  @update:model-value="(value) => setDesktopCardsAllowOverlap(value)"
                />
              </Field>
            </FieldGroup>
          </div>
          <div v-else-if="section === 'app'" class="flex min-w-0 flex-col gap-6">
            <FieldGroup>
              <Field orientation="horizontal">
                <div class="min-w-0 flex-1">
                  <FieldLabel class="text-base font-semibold">关闭窗口时</FieldLabel>
                  <FieldDescription
                    >最小化到托盘后，快捷启动和日期提醒会继续运行，可从托盘菜单退出。</FieldDescription
                  >
                </div>
                <RadioGroup
                  class="flex shrink-0 flex-wrap items-center gap-x-5 gap-y-2"
                  :model-value="store.settings.product.close_to_tray ? 'tray' : 'quit'"
                  aria-label="关闭窗口时"
                  @update:model-value="setCloseBehavior"
                >
                  <div class="flex items-center gap-2">
                    <RadioGroupItem id="close-tray" value="tray" /><FieldLabel for="close-tray"
                      >最小化到托盘</FieldLabel
                    >
                  </div>
                  <div class="flex items-center gap-2">
                    <RadioGroupItem id="close-quit" value="quit" /><FieldLabel for="close-quit"
                      >退出程序</FieldLabel
                    >
                  </div>
                </RadioGroup>
              </Field>
              <Field orientation="horizontal">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="auto-update-check" class="text-base font-semibold"
                    >自动检测更新</FieldLabel
                  >
                  <FieldDescription>启动时和运行期间定期检查新版本。</FieldDescription>
                </div>
                <Switch
                  id="auto-update-check"
                  :model-value="autoUpdateEnabled(store.settings.client)"
                  @update:model-value="
                    (value) => setUpdatePreference('lumo-auto-update-check', value)
                  "
                />
              </Field>
              <Field orientation="horizontal">
                <div class="min-w-0 flex-1">
                  <FieldLabel for="auto-update-download" class="text-base font-semibold"
                    >自动下载更新</FieldLabel
                  >
                  <FieldDescription
                    >发现新版本后在后台下载，下载完成后由你确认是否安装。</FieldDescription
                  >
                </div>
                <Switch
                  id="auto-update-download"
                  :model-value="autoDownloadEnabled(store.settings.client)"
                  @update:model-value="
                    (value) => setUpdatePreference('lumo-auto-update-download', value)
                  "
                />
              </Field>
              <Field v-if="autoUpdateEnabled(store.settings.client)" orientation="horizontal">
                <FieldLabel
                  for="update-check-interval"
                  class="min-w-0 flex-1 text-base font-semibold"
                  >检测时间间隔</FieldLabel
                >
                <Select
                  :model-value="String(updateInterval(store.settings.client))"
                  @update:model-value="setUpdateInterval"
                >
                  <SelectTrigger id="update-check-interval" class="w-48 max-w-[50%] shrink-0"
                    ><SelectValue
                  /></SelectTrigger>
                  <SelectContent
                    ><SelectGroup>
                      <SelectItem value="120">每 2 小时</SelectItem>
                      <SelectItem value="360">每 6 小时</SelectItem>
                      <SelectItem value="1440">每天</SelectItem>
                      <SelectItem value="4320">每三天</SelectItem>
                      <SelectItem value="10080">每周</SelectItem>
                      <SelectItem value="43200">每月（30 天）</SelectItem>
                    </SelectGroup></SelectContent
                  >
                </Select>
              </Field>
            </FieldGroup>
          </div>
          <div v-else-if="section === 'data'" class="flex min-w-0 flex-col gap-6">
            <div class="flex flex-col gap-5">
              <div class="rounded-2xl bg-muted/60 p-4">
                <p class="text-xs text-muted-foreground">当前数据目录</p>
                <p class="mt-1 break-all font-mono text-sm">
                  {{ store.bootstrap.data_dir || '读取中…' }}
                </p>
                <p class="mt-2 text-xs text-muted-foreground">
                  {{
                    store.bootstrap.data_follows_install
                      ? '默认保存在程序安装目录，可迁移到自定义目录。'
                      : '当前使用独立数据目录；可迁移到其他位置，后续启动会使用所选目录。'
                  }}
                </p>
              </div>
              <div>
                <Button variant="outline" :disabled="busy" @click="chooseDataPath"
                  ><FolderCog data-icon="inline-start" />迁移数据目录</Button
                >
                <p class="mt-2 break-all text-xs text-muted-foreground">
                  默认目录：{{ store.bootstrap.default_data_dir }}
                </p>
              </div>
              <div class="border-t border-border"></div>
              <div>
                <h3 class="mb-1 text-base font-semibold">备份与恢复</h3>
                <p class="mb-3 text-sm text-muted-foreground">
                  备份会包含数据库和用户资源。恢复前会自动备份当前数据。
                </p>
                <div class="flex gap-2">
                  <Button variant="outline" :disabled="busy" @click="createBackup"
                    ><Download data-icon="inline-start" />导出备份</Button
                  ><Button variant="outline" :disabled="busy" @click="restoreBackup"
                    ><Upload data-icon="inline-start" />恢复备份</Button
                  >
                </div>
              </div>
            </div>
          </div>
          <div
            v-else-if="['launcher', 'backup'].includes(section)"
            :key="section"
            class="flex min-w-0 flex-col gap-6"
          >
            <EnhancementSettings :section="section" @restore-backup="restoreBackup" />
          </div>
          <div v-else-if="section === 'about'" class="flex min-w-0 flex-col gap-6">
            <AboutPage ref="aboutPage" />
          </div>
        </CardContent>
      </ScrollArea>
    </Card>
  </div>
</template>
