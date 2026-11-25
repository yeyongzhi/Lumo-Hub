<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize, PhysicalPosition } from '@tauri-apps/api/dpi';
import { open } from '@tauri-apps/plugin-dialog';
import {
  AppWindow,
  Ellipsis,
  FileText,
  Globe,
  Lock,
  LockOpen,
  Pencil,
  RotateCcw,
  Save,
  Search,
  Trash2,
  X,
} from '@lucide/vue';
import { api } from '@/lib/api';
import { desktopCardIcon } from '@/lib/desktop-card-icons';
import { applySettings, watchSystemTheme } from '@/lib/theme';
import { useToastMessage } from '@/lib/toast-message';
import type { AppSettings, DesktopCardIconSize } from '@/lib/types';
import type { DesktopCard, DesktopCardItem } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSub,
  ContextMenuSubTrigger,
  ContextMenuSubContent,
  ContextMenuTrigger,
} from '@/components/ui/context-menu';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

const cardId = new URLSearchParams(location.search).get('desktop-card') ?? '';
const card = ref<DesktopCard | null>(null);
const allCards = ref<DesktopCard[]>([]);
const dropHint = ref(false);
const error = useToastMessage();
const removers: (() => void)[] = [];
let disposed = false;
let settings: AppSettings | undefined;
let geometryTimer: ReturnType<typeof setTimeout> | undefined;
const lastGeometry = { x: 0, y: 0, width: 0, height: 0 };

const searchOpen = ref(false);
const searchQuery = ref('');
const searchInput = ref<ComponentPublicInstance | null>(null);
const renameOpen = ref(false);
const renameTitle = ref('');
const renameInput = ref<ComponentPublicInstance | null>(null);
const renameSaving = ref(false);
const deleteOpen = ref(false);
const autoFit = ref(true);
const adaptiveHeight = ref(true);
const itemView = ref<'grid' | 'list'>('grid');
const itemIconSize = ref<DesktopCardIconSize>('medium');
const showItemNames = ref(true);
const expanded = ref(false);
let collapseTimer: ReturnType<typeof setTimeout> | undefined;
let pointerInside = false;
const headerRef = ref<HTMLElement | null>(null);
const sectionRef = ref<HTMLElement | null>(null);
const gridRef = ref<HTMLElement | null>(null);
let fitTimer: ReturnType<typeof setTimeout> | undefined;

const filteredItems = computed<DesktopCardItem[]>(() => {
  const items = card.value?.items ?? [];
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return items;
  return items.filter(
    (item) => item.name.toLowerCase().includes(query) || item.target.toLowerCase().includes(query),
  );
});

const opacityClass = computed(() => {
  const opacity = card.value?.style.opacity ?? 'frosted';
  if (opacity === 'solid') return 'bg-card';
  if (opacity === 'translucent') return 'bg-card/45 backdrop-blur-sm';
  return 'bg-card/80 backdrop-blur-xl';
});
const itemGridClass = computed(() => {
  if (itemView.value === 'list') return 'grid-cols-1 gap-1';
  const size = itemIconSize.value;
  if (size === 'small') return 'grid-cols-[repeat(auto-fill,minmax(56px,1fr))] gap-0.5';
  if (size === 'large') return 'grid-cols-[repeat(auto-fill,minmax(96px,1fr))] gap-1.5';
  return 'grid-cols-[repeat(auto-fill,minmax(72px,1fr))] gap-1';
});
const itemIconClass = computed(() => {
  const size = itemView.value === 'list' ? 'medium' : itemIconSize.value;
  if (size === 'small') return 'size-6';
  if (size === 'large') return 'size-10';
  return 'size-8';
});

async function loadCard() {
  try {
    const cards = await api.listDesktopCards();
    if (disposed) return;
    allCards.value = cards;
    card.value = cards.find((item) => item.id === cardId) ?? null;
    await nextTick();
    void updateWindowHeight();
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

function startDrag(event: PointerEvent) {
  if (card.value?.locked) return;
  if (event.button !== 0) return;
  if ((event.target as HTMLElement).closest('button, input')) return;
  event.preventDefault();
  void getCurrentWindow().startDragging();
}

function startResize(event: PointerEvent) {
  if (card.value?.locked) return;
  if (event.button !== 0) return;
  event.preventDefault();
  // 自动伸缩时高度由内容决定，只允许拖动宽度。
  void getCurrentWindow().startResizeDragging(
    adaptiveHeight.value || (autoFit.value && !expanded.value) ? 'East' : 'SouthEast',
  );
}

// 自动伸缩：高度 = 标题栏 + 内容区自然高度。窗口较矮时条目会被压缩，
// 所以先把卡片临时撑到最大高度再量一次，量完立刻还原（同一帧内完成，不会闪烁）。
async function applyAutoFit(attempt = 0) {
  if (disposed || !card.value || (autoFit.value && !expanded.value) || !adaptiveHeight.value)
    return;
  const current = getCurrentWindow();
  const inner = await current.innerSize().catch(() => null);
  const section = sectionRef.value;
  if (!inner || inner.width === 0 || inner.height === 0 || !section) {
    if (attempt < 8) scheduleAutoFit(120, attempt + 1);
    return;
  }
  const previous = section.style.height;
  section.style.height = '800px';
  const header = headerRef.value?.offsetHeight ?? 0;
  const grid = gridRef.value;
  const content = grid ? grid.offsetHeight : 0;
  section.style.height = previous;
  const empty = !card.value.items.length || !filteredItems.value.length;
  if (header === 0 || (!empty && content === 0)) {
    if (attempt < 8) scheduleAutoFit(120, attempt + 1);
    return;
  }
  // 内容区上下内边距 20 + 卡片外内边距 12 + 边框 2。
  const target = Math.min(
    800,
    Math.max(120, Math.round(empty ? header + 184 : header + content + 34)),
  );
  const scale = window.devicePixelRatio || 1;
  if (Math.abs(inner.height / scale - target) < 1) return;
  await current.setSize(new LogicalSize(inner.width / scale, target)).catch(() => {});
  // 缩放后布局可能还没更新，再校验一次收敛。
  if (attempt < 4) scheduleAutoFit(160, attempt + 1);
}

function scheduleAutoFit(delay = 120, attempt = 0) {
  if (fitTimer) clearTimeout(fitTimer);
  fitTimer = setTimeout(() => void applyAutoFit(attempt), delay);
}

// 关闭自动伸缩时回到用户手动调整过的高度。
async function restoreManualSize() {
  const stored = card.value;
  if (!stored) return;
  const current = getCurrentWindow();
  const inner = await current.innerSize().catch(() => null);
  if (!inner) return;
  const scale = window.devicePixelRatio || 1;
  await current
    .setSize(new LogicalSize(inner.width / scale, stored.height / scale))
    .catch(() => {});
}

async function updateWindowHeight() {
  if (!card.value || disposed) return;
  if (autoFit.value && !expanded.value) {
    if (fitTimer) clearTimeout(fitTimer);
    const current = getCurrentWindow();
    const inner = await current.innerSize().catch(() => null);
    if (!inner || disposed || expanded.value || !autoFit.value) return;
    const scale = window.devicePixelRatio || 1;
    await current
      .setSize(new LogicalSize(inner.width / scale, (headerRef.value?.offsetHeight ?? 48) + 14))
      .catch(() => {});
  } else if (adaptiveHeight.value) {
    await nextTick();
    await applyAutoFit();
  } else {
    await restoreManualSize();
  }
}

function expandCard() {
  if (collapseTimer) clearTimeout(collapseTimer);
  if (!autoFit.value || expanded.value) return;
  expanded.value = true;
  void updateWindowHeight();
}

function onCardEnter() {
  pointerInside = true;
  if (collapseTimer) clearTimeout(collapseTimer);
}

function scheduleCollapse() {
  pointerInside = false;
  if (!autoFit.value) return;
  if (collapseTimer) clearTimeout(collapseTimer);
  collapseTimer = setTimeout(() => {
    if (
      renameOpen.value ||
      deleteOpen.value ||
      searchOpen.value ||
      dropHint.value ||
      document.querySelector('[data-state="open"][role="menu"]')
    ) {
      if (!pointerInside) scheduleCollapse();
      return;
    }
    if (pointerInside) return;
    expanded.value = false;
    void updateWindowHeight();
  }, 250);
}

function scheduleGeometrySave() {
  if (geometryTimer) clearTimeout(geometryTimer);
  geometryTimer = setTimeout(async () => {
    if (settings?.desktop_cards?.allow_overlap === false) {
      try {
        await api.avoidDesktopCardOverlap();
        const position = await getCurrentWindow().outerPosition();
        lastGeometry.x = position.x;
        lastGeometry.y = position.y;
      } catch (e) {
        if (!disposed) error.value = String(e);
        if (card.value && (lastGeometry.x !== card.value.x || lastGeometry.y !== card.value.y)) {
          await getCurrentWindow()
            .setPosition(new PhysicalPosition(card.value.x, card.value.y))
            .catch(() => {});
        }
        return;
      }
    }
    // 自动伸缩时高度来自内容，不能覆盖用户之前保存的手动高度。
    const width = lastGeometry.width;
    const height =
      adaptiveHeight.value || (autoFit.value && !expanded.value)
        ? (card.value?.height ?? lastGeometry.height)
        : lastGeometry.height;
    try {
      await api.saveDesktopCardGeometry(lastGeometry.x, lastGeometry.y, width, height);
      if (card.value) {
        card.value.x = lastGeometry.x;
        card.value.y = lastGeometry.y;
        card.value.width = width;
        card.value.height = height;
      }
    } catch (e) {
      if (!disposed) error.value = `保存卡片位置失败：${String(e)}`;
    }
  }, 400);
}

async function dropFiles(paths: string[]) {
  if (!card.value) return;
  try {
    const result = await api.dropDesktopCardItems(card.value.id, paths);
    await loadCard();
    if (result.errors.length)
      error.value = `已处理 ${result.added} 个，失败 ${result.errors.length} 个：\n${result.errors.join('\n')}`;
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

async function addFilesToCard() {
  if (!card.value) return;
  try {
    await setInteractive(true);
    const paths = await open({ multiple: true });
    if (!paths) return;
    await api.addDesktopCardItems(card.value.id, Array.isArray(paths) ? paths : [paths]);
    await loadCard();
  } catch (e) {
    error.value = String(e);
  } finally {
    await setInteractive(false);
  }
}

async function openItem(item: DesktopCardItem) {
  if (!card.value) return;
  try {
    await api.openDesktopCardItem(card.value.id, item.id);
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

async function revealItem(item: DesktopCardItem) {
  try {
    await api.revealDesktopCardItem(cardId, item.id);
  } catch (e) {
    error.value = String(e);
  }
}
async function revealOriginal(item: DesktopCardItem) {
  try {
    await api.revealDesktopCardOriginal(cardId, item.id);
  } catch (e) {
    error.value = String(e);
  }
}
async function copyItem(item: DesktopCardItem) {
  try {
    await navigator.clipboard.writeText(item.target);
  } catch (e) {
    error.value = `复制失败：${String(e)}`;
  }
}
async function moveItem(item: DesktopCardItem, destination: string, before?: string) {
  try {
    await api.moveDesktopCardItems(cardId, [item.id], destination, before);
    await loadCard();
  } catch (e) {
    error.value = `移动失败：${String(e)}`;
  }
}

async function removeItem(item: DesktopCardItem) {
  if (!card.value) return;
  try {
    await api.removeDesktopCardItem(card.value.id, item.id);
    await loadCard();
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

async function relocateItem(item: DesktopCardItem, directory: boolean) {
  if (!card.value) return;
  try {
    await setInteractive(true);
    const path = await open({ directory, multiple: false });
    if (!path || Array.isArray(path)) return;
    await api.relocateDesktopCardItem(card.value.id, item.id, path);
    await loadCard();
  } catch (e) {
    if (!disposed) error.value = `重新定位失败：${String(e)}`;
  } finally {
    await setInteractive(false);
  }
}

async function setInteractive(active: boolean) {
  try {
    await api.setDesktopCardInteractive(active);
  } catch (e) {
    if (!disposed) error.value = `切换输入模式失败：${String(e)}`;
  }
}

async function toggleSearch() {
  if (searchOpen.value) {
    closeSearch();
    return;
  }
  searchOpen.value = true;
  await setInteractive(true);
  await nextTick();
  (searchInput.value?.$el as HTMLInputElement | undefined)?.focus();
}

function closeSearch() {
  if (!searchOpen.value) return;
  searchOpen.value = false;
  searchQuery.value = '';
  void setInteractive(false);
}

async function startRename() {
  if (!card.value) return;
  renameTitle.value = card.value.title;
  renameOpen.value = true;
  await setInteractive(true);
  await nextTick();
  (renameInput.value?.$el as HTMLInputElement | undefined)?.focus();
}

async function confirmRename() {
  const title = renameTitle.value.trim();
  if (!card.value || !title || renameSaving.value) return;
  renameSaving.value = true;
  try {
    await api.renameDesktopCard(card.value.id, title);
    renameOpen.value = false;
    await loadCard();
  } catch (e) {
    if (!disposed) error.value = String(e);
  } finally {
    renameSaving.value = false;
  }
}

async function removeCard() {
  if (!card.value) return;
  try {
    await api.deleteDesktopCard(card.value.id);
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

async function toggleLock() {
  if (!card.value) return;
  try {
    await api.setDesktopCardLocked(card.value.id, !card.value.locked);
    await loadCard();
  } catch (e) {
    error.value = String(e);
  }
}

async function resetPosition() {
  if (!card.value) return;
  try {
    await api.resetDesktopCardPosition(card.value.id);
    await loadCard();
  } catch (e) {
    error.value = String(e);
  }
}

async function refreshSettings() {
  try {
    const next = (await api.bootstrap()).settings;
    if (disposed) return;
    if (!settings) removers.push(watchSystemTheme(() => settings ?? next));
    settings = next;
    const nextAutoFit = next.desktop_cards?.auto_fit !== false;
    const nextAdaptiveHeight = next.desktop_cards?.adaptive_height !== false;
    const autoFitChanged = autoFit.value !== nextAutoFit;
    autoFit.value = nextAutoFit;
    adaptiveHeight.value = nextAdaptiveHeight;
    itemView.value = next.desktop_cards?.item_view === 'list' ? 'list' : 'grid';
    itemIconSize.value = next.desktop_cards?.item_icon_size ?? 'medium';
    showItemNames.value = next.desktop_cards?.show_item_names !== false;
    if (autoFitChanged || !nextAutoFit) expanded.value = !nextAutoFit;
    if (!nextAutoFit && !nextAdaptiveHeight) void restoreManualSize();
    applySettings(settings);
    await nextTick();
    void updateWindowHeight();
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
}

watch(renameOpen, (open) => {
  if (!open) void setInteractive(false);
});
// 搜索会改变可见条目数量，自动伸缩时高度随之变化。
watch(searchQuery, async () => {
  await nextTick();
  void applyAutoFit();
});

onMounted(async () => {
  const current = getCurrentWindow();
  try {
    const [moved, resized, changed, themeChanged, dropListener, focusListener] = await Promise.all([
      current.onMoved((event) => {
        lastGeometry.x = event.payload.x;
        lastGeometry.y = event.payload.y;
        scheduleGeometrySave();
      }),
      current.onResized((event) => {
        lastGeometry.width = event.payload.width;
        lastGeometry.height = event.payload.height;
        scheduleGeometrySave();
        // 宽度变化会改变每行可放下的条目数，自动伸缩需要重新贴合高度。
        if (adaptiveHeight.value && (!autoFit.value || expanded.value)) scheduleAutoFit();
      }),
      listen('lumo-cards-changed', () => void loadCard()),
      listen('lumo-settings-changed', () => void refreshSettings()),
      current.onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === 'enter') {
          dropHint.value = true;
        } else if (payload.type === 'drop') {
          dropHint.value = false;
          void dropFiles(payload.paths);
        } else if (payload.type === 'leave') {
          dropHint.value = false;
        }
      }),
      current.onFocusChanged((event) => {
        if (!event.payload && searchOpen.value) closeSearch();
      }),
    ]);
    if (disposed) {
      [moved, resized, changed, themeChanged, dropListener, focusListener].forEach((remove) =>
        remove(),
      );
      return;
    }
    removers.push(moved, resized, changed, themeChanged, dropListener, focusListener);
  } catch (e) {
    if (!disposed) error.value = String(e);
  }
  const position = await current.outerPosition().catch(() => null);
  const size = await current.outerSize().catch(() => null);
  if (position) {
    lastGeometry.x = position.x;
    lastGeometry.y = position.y;
  }
  if (size) {
    lastGeometry.width = size.width;
    lastGeometry.height = size.height;
  }
  await refreshSettings();
  await loadCard();
});
onUnmounted(() => {
  disposed = true;
  if (geometryTimer) clearTimeout(geometryTimer);
  if (fitTimer) clearTimeout(fitTimer);
  if (collapseTimer) clearTimeout(collapseTimer);
  removers.forEach((remove) => remove());
});
</script>

<template>
  <div class="h-dvh w-full p-1.5">
    <ContextMenu>
      <ContextMenuTrigger as-child>
        <section
          v-if="card"
          ref="sectionRef"
          class="relative flex h-full w-full flex-col overflow-hidden rounded-2xl border text-foreground shadow-xl"
          @pointerenter="onCardEnter"
          @pointerleave="scheduleCollapse"
          :class="[
            opacityClass,
            dropHint ? 'border-primary ring-2 ring-primary/60' : 'border-border',
          ]"
        >
          <header
            ref="headerRef"
            class="flex h-12 shrink-0 select-none items-center gap-1.5 border-b border-border/60 px-4"
            @pointerenter="expandCard"
            @pointerdown="startDrag"
          >
            <span v-if="card.emoji" class="mr-1 shrink-0 text-sm leading-none" aria-hidden="true">{{
              card.emoji
            }}</span>
            <component
              v-else-if="desktopCardIcon(card.icon)"
              :is="desktopCardIcon(card.icon)"
              class="mr-1 size-4 shrink-0 text-primary"
            />
            <template v-if="searchOpen">
              <Input
                ref="searchInput"
                v-model="searchQuery"
                class="h-7 min-w-0 flex-1 text-sm"
                placeholder="搜索卡片条目"
                aria-label="搜索卡片条目"
                @pointerdown.stop
                @keydown.esc="closeSearch"
              />
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="关闭搜索"
                title="关闭搜索"
                @pointerdown.stop
                @click="closeSearch"
                ><X
              /></Button>
            </template>
            <template v-else>
              <template v-if="renameOpen">
                <Input
                  ref="renameInput"
                  v-model="renameTitle"
                  class="h-7 min-w-0 flex-1 text-sm"
                  aria-label="卡片名称"
                  maxlength="60"
                  @pointerdown.stop
                  @keydown.enter.prevent="confirmRename"
                  @keydown.esc="renameOpen = false"
                />
                <Button
                  variant="ghost"
                  size="icon-xs"
                  aria-label="保存卡片名称"
                  title="保存"
                  :disabled="renameSaving || !renameTitle.trim()"
                  @click="confirmRename"
                  ><Save
                /></Button>
              </template>
              <h1 v-else class="min-w-0 flex-1 truncate text-sm font-semibold">{{ card.title }}</h1>
              <span class="shrink-0 text-sm text-muted-foreground">{{ card.items.length }} 项</span>
              <Button
                variant="ghost"
                size="icon-sm"
                :aria-pressed="searchOpen"
                aria-label="搜索卡片条目"
                title="搜索"
                @click="toggleSearch"
                ><Search
              /></Button>
              <DropdownMenu>
                <DropdownMenuTrigger as-child
                  ><Button variant="ghost" size="icon-sm" aria-label="更多操作" title="更多操作"
                    ><Ellipsis
                  /></Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" class="w-44">
                  <DropdownMenuItem @click="startRename"
                    ><Pencil data-icon="inline-start" />重命名卡片</DropdownMenuItem
                  >
                  <DropdownMenuItem @click="toggleLock"
                    ><component :is="card.locked ? LockOpen : Lock" data-icon="inline-start" />{{
                      card.locked ? '解锁位置' : '锁定位置'
                    }}</DropdownMenuItem
                  >
                  <DropdownMenuItem @click="resetPosition"
                    ><RotateCcw data-icon="inline-start" />重置位置</DropdownMenuItem
                  >
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    class="text-destructive focus:text-destructive"
                    @click="deleteOpen = true"
                    ><Trash2 data-icon="inline-start" />删除卡片</DropdownMenuItem
                  >
                </DropdownMenuContent>
              </DropdownMenu>
            </template>
          </header>
          <div v-show="!autoFit || expanded" class="min-h-0 flex-1 overflow-hidden p-2.5">
            <p
              v-if="!card.items.length"
              class="flex h-full flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-border/70 p-4 text-center text-sm text-muted-foreground"
            >
              将桌面文件拖到这里
              <span class="text-xs">桌面文件会移入卡片；其他位置的文件保留原位</span>
            </p>
            <template v-else-if="!filteredItems.length">
              <p class="flex h-full items-center justify-center text-sm text-muted-foreground">
                没有匹配「{{ searchQuery.trim() }}」的条目
              </p>
            </template>
            <div
              v-else
              ref="gridRef"
              :class="itemGridClass"
              class="grid max-h-full content-start overflow-y-auto p-1"
            >
              <ContextMenu v-for="item in filteredItems" :key="item.id">
                <ContextMenuTrigger as-child>
                  <div
                    class="group relative flex min-w-0 items-center gap-1 rounded-lg border border-transparent p-2 transition-colors hover:border-border hover:bg-accent"
                    :class="itemView === 'list' ? 'flex-row' : 'flex-col'"
                  >
                    <button
                      class="flex min-w-0 w-full items-center gap-1"
                      :class="itemView === 'list' ? 'flex-row text-left' : 'flex-col'"
                      :aria-label="item.name"
                      :title="item.target"
                      @click="openItem(item)"
                    >
                      <img
                        v-if="item.icon_data"
                        :src="item.icon_data"
                        alt=""
                        :class="itemIconClass"
                        class="shrink-0 rounded object-contain"
                      />
                      <AppWindow
                        v-else-if="item.kind === 'app'"
                        :class="itemIconClass"
                        class="shrink-0 text-muted-foreground"
                      />
                      <Globe
                        v-else-if="item.kind === 'web'"
                        :class="itemIconClass"
                        class="shrink-0 text-muted-foreground"
                      />
                      <FileText
                        v-else
                        :class="itemIconClass"
                        class="shrink-0 text-muted-foreground"
                      />
                      <span
                        v-if="showItemNames"
                        class="min-w-0 w-full truncate text-xs"
                        :class="itemView === 'list' ? 'text-left' : 'text-center'"
                        >{{ item.name }}</span
                      >
                      <span v-if="item.missing" class="text-xs text-destructive">文件不存在</span>
                      <span
                        v-if="item.recovery_state === 'conflict'"
                        class="text-xs text-destructive"
                        >原位置存在同名文件</span
                      >
                      <span
                        v-else-if="item.recovery_state === 'missing'"
                        class="text-xs text-destructive"
                        >卡片文件与原位置都未找到</span
                      >
                    </button>
                    <div
                      v-if="item.missing && !item.managed"
                      class="flex flex-wrap justify-center gap-1 text-xs text-primary"
                    >
                      <button @click="relocateItem(item, false)">选择文件</button>
                      <button @click="relocateItem(item, true)">选择文件夹</button>
                    </div>
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem @select="openItem(item)">打开</ContextMenuItem>
                  <ContextMenuItem
                    :disabled="item.kind === 'web' || item.missing"
                    @select="revealItem(item)"
                    >打开所在位置</ContextMenuItem
                  >
                  <ContextMenuItem @select="copyItem(item)">复制路径</ContextMenuItem>
                  <ContextMenuItem v-if="item.original_path" @select="revealOriginal(item)"
                    >查看原桌面位置</ContextMenuItem
                  >
                  <ContextMenuItem
                    :disabled="card.items[0]?.id === item.id"
                    @select="
                      moveItem(
                        item,
                        cardId,
                        card.items[card.items.findIndex((entry) => entry.id === item.id) - 1]?.id,
                      )
                    "
                    >向前移动</ContextMenuItem
                  >
                  <ContextMenuItem
                    :disabled="card.items[card.items.length - 1]?.id === item.id"
                    @select="
                      moveItem(
                        item,
                        cardId,
                        card.items[card.items.findIndex((entry) => entry.id === item.id) + 2]?.id,
                      )
                    "
                    >向后移动</ContextMenuItem
                  >
                  <ContextMenuSub v-if="allCards.length > 1">
                    <ContextMenuSubTrigger>移动到其他卡片</ContextMenuSubTrigger>
                    <ContextMenuSubContent>
                      <ContextMenuItem
                        v-for="destination in allCards.filter((entry) => entry.id !== cardId)"
                        :key="destination.id"
                        @select="moveItem(item, destination.id)"
                        >{{ destination.title }}</ContextMenuItem
                      >
                    </ContextMenuSubContent>
                  </ContextMenuSub>
                  <ContextMenuItem @select="removeItem(item)">{{
                    item.managed ? '移回原位置' : '移除条目'
                  }}</ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            </div>
          </div>
          <div
            v-if="!card.locked"
            class="absolute z-10"
            :class="
              adaptiveHeight || (autoFit && !expanded)
                ? 'bottom-1/2 right-0 h-16 w-2 translate-y-1/2 cursor-ew-resize rounded-l bg-border/40'
                : 'bottom-0 right-0 size-5 cursor-nwse-resize'
            "
            aria-hidden="true"
            @pointerdown="startResize"
          />
        </section>
      </ContextMenuTrigger>
      <ContextMenuContent v-if="card">
        <ContextMenuItem @select="addFilesToCard">添加文件</ContextMenuItem>
        <ContextMenuItem @select="toggleLock">{{
          card.locked ? '解锁位置' : '锁定位置'
        }}</ContextMenuItem>
        <ContextMenuItem @select="startRename">重命名卡片</ContextMenuItem>
        <ContextMenuItem @select="deleteOpen = true">删除卡片</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
    <ConfirmDialog
      v-model:open="deleteOpen"
      title="删除桌面卡片"
      :description="`将删除桌面卡片「${card?.title ?? ''}」；移入的文件会移回原桌面位置，其他引用不受影响。如有同名文件，请先重命名。`"
      confirm-text="删除"
      destructive
      @confirm="removeCard"
    />
  </div>
</template>
