<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import {
  AppWindow,
  ArrowDown,
  ArrowUp,
  Check,
  Eye,
  EyeOff,
  Ellipsis,
  FileText,
  Globe,
  LayoutGrid,
  Lock,
  LockOpen,
  Pencil,
  Plus,
  Trash2,
} from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
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
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { api } from '@/lib/api';
import { desktopCardIcon } from '@/lib/desktop-card-icons';
import DesktopCardIdentityPicker from '@/components/DesktopCardIdentityPicker.vue';
import { useWorkspaceStore } from '@/stores/workspace';
import type { DesktopCard, DesktopCardItem, DesktopCardOpacity } from '@/lib/types';

const store = useWorkspaceStore();
const busy = ref(false);
const cardSearch = ref('');
const createOpen = ref(false);
const createTitle = ref('');
const createPaths = ref<string[]>([]);
const editPaths = ref<string[]>([]);
const createIconEnabled = ref(false);
const editIconEnabled = ref(false);
const createIcon = ref<string | null>(null);
const createEmoji = ref<string | null>(null);
const editIcon = ref<string | null>(null);
const editEmoji = ref<string | null>(null);
const editOpacity = ref<DesktopCardOpacity>('frosted');
const opacityOptions: { value: DesktopCardOpacity; label: string }[] = [
  { value: 'frosted', label: '磨砂' },
  { value: 'translucent', label: '半透明' },
  { value: 'solid', label: '纯色' },
];
const filteredCards = computed(() => {
  const query = cardSearch.value.trim().toLocaleLowerCase();
  if (!query) return store.cards;
  return store.cards.filter((card) => card.title.toLocaleLowerCase().includes(query));
});
const renameOpen = ref(false);
const renameTitle = ref('');
const editSaveError = ref('');
const renameTarget = ref<DesktopCard | null>(null);
const deleteTarget = ref<DesktopCard | null>(null);
const deleteOpen = ref(false);
const batchCardId = ref<string | null>(null);
const selectedItems = ref<string[]>([]);
const draggingItem = ref<{ cardId: string; itemId: string } | null>(null);
const removers: (() => void)[] = [];
let disposed = false;
onMounted(async () => {
  try {
    const unlisten = await listen('lumo-cards-changed', () => void store.reloadCards());
    if (disposed) {
      unlisten();
      return;
    }
    removers.push(unlisten);
    await store.reloadCards();
  } catch (error) {
    store.notify(`桌面卡片初始化失败：${String(error)}`);
  }
});
onUnmounted(() => {
  disposed = true;
  removers.forEach((remove) => remove());
});

function openCreateDialog() {
  createTitle.value = '';
  createPaths.value = [];
  createIcon.value = null;
  createEmoji.value = null;
  createIconEnabled.value = false;
  createOpen.value = true;
}

async function chooseCreateFiles() {
  const result = await open({ multiple: true });
  if (!result) return;
  const paths = Array.isArray(result) ? result : [result];
  createPaths.value = [...new Set([...createPaths.value, ...paths])];
}

async function chooseEditFiles() {
  const result = await open({ multiple: true });
  if (!result) return;
  editPaths.value = [
    ...new Set([...editPaths.value, ...(Array.isArray(result) ? result : [result])]),
  ];
}

async function createCard() {
  if (busy.value) return;
  const title = createTitle.value.trim();
  if (!title) return;
  busy.value = true;
  let created: DesktopCard | null = null;
  try {
    created = await api.createDesktopCard(
      title,
      createIconEnabled.value ? createIcon.value : null,
      createIconEnabled.value ? createEmoji.value : null,
    );
    if (createPaths.value.length) {
      await api.addDesktopCardItems(created.id, createPaths.value);
    }
    createOpen.value = false;
    store.notify('桌面卡片已创建');
    await store.reloadCards();
  } catch (error) {
    if (created) {
      try {
        await api.deleteDesktopCard(created.id);
      } catch {
        // Keep the original failure as the user-facing error.
      }
    }
    store.notify(`创建桌面卡片失败：${String(error)}`);
  } finally {
    busy.value = false;
  }
}
function startRename(card: DesktopCard) {
  renameTarget.value = { ...card, style: { ...card.style } };
  editSaveError.value = '';
  renameTitle.value = card.title;
  editIcon.value = card.icon;
  editEmoji.value = card.emoji;
  editIconEnabled.value = !!(card.icon || card.emoji);
  editPaths.value = [];
  editOpacity.value = card.style.opacity;
  renameOpen.value = true;
}

async function confirmRename() {
  const card = renameTarget.value;
  const title = renameTitle.value.trim();
  if (!card || !title || busy.value) return;
  const icon = editIconEnabled.value ? editIcon.value : null;
  const emoji = editIconEnabled.value ? editEmoji.value : null;
  const opacity = editOpacity.value;
  const iconSize = card.style.icon_size;
  const paths = [...editPaths.value];
  const completed: string[] = [];
  let step = '名称和图标';
  editSaveError.value = '';
  busy.value = true;
  try {
    if (card.title !== title || card.icon !== icon || card.emoji !== emoji) {
      await api.updateDesktopCardIdentity(card.id, title, icon, emoji);
      card.title = title;
      card.icon = icon;
      card.emoji = emoji;
      completed.push('名称和图标');
    }
    step = '文件';
    if (paths.length) {
      await api.addDesktopCardItems(card.id, paths);
      editPaths.value = editPaths.value.filter((path) => !paths.includes(path));
      completed.push('文件');
    }
    step = '卡片外观';
    if (card.style.opacity !== opacity || card.style.icon_size !== iconSize) {
      await api.setDesktopCardStyle(card.id, { opacity, icon_size: iconSize });
      card.style = { opacity, icon_size: iconSize };
      completed.push('卡片外观');
    }
    renameOpen.value = false;
    await store.reloadCards();
    store.notify('卡片设置已更新');
  } catch (error) {
    const saved = completed.length ? `已保存：${completed.join('、')}。` : '本次没有完成保存。';
    editSaveError.value = `${saved}${step}保存失败：${String(error)}。未完成的修改仍保留，可以调整后重试。`;
    await store.reloadCards();
  } finally {
    busy.value = false;
  }
}

async function removeCard() {
  const card = deleteTarget.value;
  if (!card || busy.value) return;
  busy.value = true;
  try {
    await api.deleteDesktopCard(card.id);
    deleteOpen.value = false;
    await store.reloadCards();
    store.notify('桌面卡片已删除');
  } catch (error) {
    store.notify(`删除桌面卡片失败：${String(error)}`);
  } finally {
    busy.value = false;
  }
}

async function addFiles(card: DesktopCard) {
  const result = await open({ multiple: true });
  if (!result) return;
  const paths = Array.isArray(result) ? result : [result];
  if (!paths.length) return;
  try {
    const added = await api.addDesktopCardItems(card.id, paths);
    await store.reloadCards();
    store.notify(added ? `已添加 ${added} 个条目` : '所选内容均已在卡片中');
  } catch (error) {
    store.notify(`添加条目失败：${String(error)}`);
  }
}

async function sortItemsByName(card: DesktopCard) {
  try {
    await api.reorderDesktopCardItems(
      card.id,
      [...card.items].sort((a, b) => a.name.localeCompare(b.name, 'zh-CN')).map((item) => item.id),
    );
    await store.reloadCards();
  } catch (error) {
    store.notify(`排序失败：${String(error)}`);
  }
}

async function reorderCards(card: DesktopCard, direction: -1 | 1) {
  const cards = [...store.cards];
  const index = cards.findIndex((item) => item.id === card.id);
  const target = index + direction;
  if (target < 0 || target >= cards.length) return;
  [cards[index], cards[target]] = [cards[target], cards[index]];
  try {
    await api.reorderDesktopCards(cards.map((item) => item.id));
    await store.reloadCards();
  } catch (error) {
    store.notify(`卡片排序失败：${String(error)}`);
  }
}

async function toggleLock(card: DesktopCard) {
  try {
    await api.setDesktopCardLocked(card.id, !card.locked);
    await store.reloadCards();
  } catch (error) {
    store.notify(`切换锁定状态失败：${String(error)}`);
  }
}

async function removeItem(card: DesktopCard, item: DesktopCardItem) {
  try {
    await api.removeDesktopCardItem(card.id, item.id);
    await store.reloadCards();
  } catch (error) {
    store.notify(`移除条目失败：${String(error)}`);
  }
}

async function relocateItem(card: DesktopCard, item: DesktopCardItem, directory: boolean) {
  try {
    const path = await open({ directory, multiple: false });
    if (!path || Array.isArray(path)) return;
    await api.relocateDesktopCardItem(card.id, item.id, path);
    await store.reloadCards();
    store.notify('条目位置已更新');
  } catch (error) {
    store.notify(`重新定位失败：${String(error)}`);
  }
}

async function openItem(card: DesktopCard, item: DesktopCardItem) {
  try {
    await api.openDesktopCardItem(card.id, item.id);
  } catch (error) {
    store.notify(String(error));
  }
}

async function revealItem(card: DesktopCard, item: DesktopCardItem) {
  try {
    await api.revealDesktopCardItem(card.id, item.id);
  } catch (error) {
    store.notify(String(error));
  }
}
async function revealOriginal(card: DesktopCard, item: DesktopCardItem) {
  try {
    await api.revealDesktopCardOriginal(card.id, item.id);
  } catch (error) {
    store.notify(String(error));
  }
}
async function copyItem(item: DesktopCardItem) {
  try {
    await navigator.clipboard.writeText(item.target);
    store.notify('路径已复制');
  } catch (error) {
    store.notify(`复制失败：${String(error)}`);
  }
}
async function moveItemTo(cardId: string, itemId: string, destination: string, before?: string) {
  try {
    await api.moveDesktopCardItems(cardId, [itemId], destination, before);
    await store.reloadCards();
  } catch (error) {
    store.notify(`移动失败：${String(error)}`);
  }
}
async function moveItem(
  card: DesktopCard,
  item: DesktopCardItem,
  destination: string,
  before?: string,
) {
  await moveItemTo(card.id, item.id, destination, before);
}

function toggleBatch(card: DesktopCard) {
  batchCardId.value = batchCardId.value === card.id ? null : card.id;
  selectedItems.value = [];
}
function toggleSelected(itemId: string) {
  selectedItems.value = selectedItems.value.includes(itemId)
    ? selectedItems.value.filter((id) => id !== itemId)
    : [...selectedItems.value, itemId];
}
async function batchRemove(card: DesktopCard) {
  if (busy.value) return;
  busy.value = true;
  const selected = card.items.filter((item) => selectedItems.value.includes(item.id));
  const failed: string[] = [];
  const errors: string[] = [];
  for (const item of selected) {
    try {
      await api.removeDesktopCardItem(card.id, item.id);
    } catch (error) {
      failed.push(item.id);
      errors.push(`${item.name}：${String(error)}`);
    }
  }
  selectedItems.value = failed;
  try {
    await store.reloadCards();
  } finally {
    busy.value = false;
  }
  store.notify(
    `成功 ${selected.length - failed.length} 项，失败 ${failed.length} 项${errors.length ? `：${errors.join('；')}` : ''}`,
  );
}
async function batchMove(card: DesktopCard, destinationId: string) {
  if (busy.value) return;
  busy.value = true;
  try {
    await api.moveDesktopCardItems(card.id, selectedItems.value, destinationId);
    selectedItems.value = [];
    batchCardId.value = null;
    await store.reloadCards();
  } catch (error) {
    store.notify(`批量移动失败：${String(error)}`);
  } finally {
    busy.value = false;
  }
}

// 桌面端外置拖放（tauri 的 dragDropEnabled）会接管 OLE 拖拽，HTML5 draggable 在
// Windows 上不会触发，因此条目重排改用指针事件实现，鼠标拖动同样可用。
const pointerDrag = ref<{ cardId: string; itemId: string; x: number; y: number } | null>(null);
let dragController: AbortController | null = null;
let suppressClick = false;

function startItemDrag(card: DesktopCard, item: DesktopCardItem, event: PointerEvent) {
  if (event.button !== 0 || batchCardId.value === card.id) return;
  dragController?.abort();
  dragController = new AbortController();
  const { signal } = dragController;
  pointerDrag.value = { cardId: card.id, itemId: item.id, x: event.clientX, y: event.clientY };
  window.addEventListener('pointermove', moveItemDrag, { signal });
  window.addEventListener('pointerup', endItemDrag, { signal });
  window.addEventListener('pointercancel', cancelItemDrag, { signal });
}

function moveItemDrag(event: PointerEvent) {
  const drag = pointerDrag.value;
  if (!drag || draggingItem.value) return;
  if (Math.hypot(event.clientX - drag.x, event.clientY - drag.y) > 6) {
    draggingItem.value = { cardId: drag.cardId, itemId: drag.itemId };
  }
}

function cancelItemDrag() {
  dragController?.abort();
  dragController = null;
  pointerDrag.value = null;
  draggingItem.value = null;
}

function endItemDrag(event: PointerEvent) {
  const drag = pointerDrag.value;
  const wasDragging = draggingItem.value !== null;
  cancelItemDrag();
  if (!drag || !wasDragging) return;
  suppressClick = true;
  setTimeout(() => (suppressClick = false), 0);
  const target = document
    .elementFromPoint(event.clientX, event.clientY)
    ?.closest<HTMLElement>('[data-card-item]');
  const targetItemId = target?.dataset.cardItem;
  const targetCardId = target?.dataset.cardId;
  if (!targetItemId || !targetCardId || targetItemId === drag.itemId) return;
  void moveItemTo(drag.cardId, drag.itemId, targetCardId, targetItemId);
}

function onItemClick(card: DesktopCard, item: DesktopCardItem) {
  if (suppressClick) {
    suppressClick = false;
    return;
  }
  if (batchCardId.value === card.id) toggleSelected(item.id);
  else void openItem(card, item);
}

async function toggleDesktopVisibility(card: DesktopCard) {
  const visible = !card.visible_on_desktop;
  try {
    await api.setDesktopCardVisibility(card.id, visible);
    await store.reloadCards();
    store.notify(visible ? '卡片已设为在桌面显示' : '卡片已从桌面隐藏');
  } catch (error) {
    store.notify(`切换桌面显示状态失败：${String(error)}`);
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-1 flex-col gap-4 overflow-hidden">
    <div
      class="flex shrink-0 flex-wrap items-center justify-between gap-3"
      aria-label="桌面卡片操作"
    >
      <Input
        v-model="cardSearch"
        class="max-w-xs"
        placeholder="搜索卡片名称"
        aria-label="搜索卡片名称"
      />
      <div class="flex flex-wrap items-center gap-2">
        <Button :disabled="busy" @click="openCreateDialog"
          ><Plus data-icon="inline-start" />新建卡片</Button
        >
      </div>
    </div>
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <Empty
        v-if="!store.cards.length"
        class="absolute inset-0 items-center justify-center"
        role="status"
      >
        <EmptyHeader>
          <EmptyMedia variant="icon"><LayoutGrid /></EmptyMedia>
          <EmptyTitle>还没有桌面卡片</EmptyTitle>
          <EmptyDescription>
            新建一张卡片后，它会固定在桌面上；把常用文件拖进卡片即可分类整理。
          </EmptyDescription>
        </EmptyHeader>
        <Button :disabled="busy" @click="openCreateDialog"
          ><Plus data-icon="inline-start" />新建卡片</Button
        >
      </Empty>
      <ScrollArea v-else class="absolute inset-0 h-full w-full">
        <div class="flex min-w-0 flex-col gap-4 pr-3 pb-3">
          <p v-if="!filteredCards.length" class="py-8 text-center text-sm text-muted-foreground">
            没有匹配的卡片
          </p>
          <ContextMenu v-for="card in filteredCards" :key="card.id">
            <ContextMenuTrigger as-child>
              <Card class="gap-0 p-4">
                <div class="flex items-center gap-2">
                  <span
                    v-if="card.emoji"
                    class="shrink-0 text-base leading-none"
                    aria-hidden="true"
                    >{{ card.emoji }}</span
                  >
                  <component
                    v-else-if="desktopCardIcon(card.icon)"
                    :is="desktopCardIcon(card.icon)"
                    class="size-4 shrink-0 text-primary"
                    aria-hidden="true"
                  />
                  <h3 class="min-w-0 flex-1 truncate text-sm font-semibold">{{ card.title }}</h3>
                  <Badge variant="secondary" class="text-sm">{{ card.items.length }} 项</Badge>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    :aria-label="`编辑卡片 ${card.title}`"
                    title="编辑"
                    @click="startRename(card)"
                    ><Pencil
                  /></Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    :aria-label="`删除卡片 ${card.title}`"
                    title="删除卡片"
                    @click="
                      deleteTarget = card;
                      deleteOpen = true;
                    "
                    ><Trash2
                  /></Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child
                      ><Button
                        variant="ghost"
                        size="icon-sm"
                        class="ml-1"
                        :aria-label="`更多卡片操作 ${card.title}`"
                        title="更多"
                        ><Ellipsis /></Button
                    ></DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        :disabled="store.cards[0]?.id === card.id"
                        @click="reorderCards(card, -1)"
                        ><ArrowUp data-icon="inline-start" />卡片上移</DropdownMenuItem
                      >
                      <DropdownMenuItem
                        :disabled="store.cards[store.cards.length - 1]?.id === card.id"
                        @click="reorderCards(card, 1)"
                        ><ArrowDown data-icon="inline-start" />卡片下移</DropdownMenuItem
                      >
                      <DropdownMenuItem @click="toggleLock(card)">
                        <component :is="card.locked ? Lock : LockOpen" data-icon="inline-start" />{{
                          card.locked ? '解锁位置' : '锁定位置'
                        }}
                      </DropdownMenuItem>
                      <DropdownMenuItem @click="toggleDesktopVisibility(card)">
                        <component
                          :is="card.visible_on_desktop ? EyeOff : Eye"
                          data-icon="inline-start"
                        />{{ card.visible_on_desktop ? '从桌面隐藏' : '在桌面显示' }}
                      </DropdownMenuItem>
                      <DropdownMenuItem @click="toggleBatch(card)">{{
                        batchCardId === card.id ? '退出批量管理' : '批量管理条目'
                      }}</DropdownMenuItem>
                      <DropdownMenuItem @click="sortItemsByName(card)">按名称排序</DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
                <div
                  v-if="card.items.length"
                  class="mt-2 grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] content-start gap-4 px-2 py-1"
                >
                  <ContextMenu v-for="item in card.items" :key="item.id">
                    <ContextMenuTrigger as-child>
                      <div
                        class="group relative min-w-0 rounded-2xl transition-opacity"
                        :class="draggingItem?.itemId === item.id ? 'opacity-40' : ''"
                        :data-card-item="item.id"
                        :data-card-id="card.id"
                        @pointerdown="startItemDrag(card, item, $event)"
                      >
                        <Button
                          variant="ghost"
                          class="h-28 w-full min-w-0 flex-col items-center justify-center gap-3 rounded-2xl px-3 py-3"
                          :title="item.target"
                          :class="
                            batchCardId === card.id && selectedItems.includes(item.id)
                              ? 'rounded-md bg-primary/15 ring-2 ring-primary'
                              : ''
                          "
                          @click="onItemClick(card, item)"
                        >
                          <img
                            v-if="item.icon_data"
                            :src="item.icon_data"
                            alt=""
                            class="size-10 object-contain"
                          />
                          <AppWindow v-else-if="item.kind === 'app'" class="size-10 text-primary" />
                          <Globe v-else-if="item.kind === 'web'" class="size-10 text-primary" />
                          <FileText v-else class="size-10 text-primary" />
                          <span class="w-full truncate text-center text-sm text-foreground">{{
                            item.name
                          }}</span>
                          <span v-if="item.missing" class="text-xs text-destructive"
                            >文件不存在</span
                          >
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
                        </Button>
                        <div
                          v-if="item.missing && !item.managed"
                          class="flex flex-wrap justify-center gap-1 text-xs text-primary"
                        >
                          <button @click="relocateItem(card, item, false)">选择文件</button>
                          <button @click="relocateItem(card, item, true)">选择文件夹</button>
                        </div>
                      </div>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem @select="openItem(card, item)">打开</ContextMenuItem>
                      <ContextMenuItem
                        :disabled="item.kind === 'web' || item.missing"
                        @select="revealItem(card, item)"
                        >打开所在位置</ContextMenuItem
                      >
                      <ContextMenuItem @select="copyItem(item)">复制路径</ContextMenuItem>
                      <ContextMenuItem
                        v-if="item.original_path"
                        @select="revealOriginal(card, item)"
                        >查看原桌面位置</ContextMenuItem
                      >
                      <ContextMenuItem
                        :disabled="card.items[0]?.id === item.id"
                        @select="
                          moveItem(
                            card,
                            item,
                            card.id,
                            card.items[card.items.findIndex((entry) => entry.id === item.id) - 1]
                              ?.id,
                          )
                        "
                        >向前移动</ContextMenuItem
                      >
                      <ContextMenuItem
                        :disabled="card.items[card.items.length - 1]?.id === item.id"
                        @select="
                          moveItem(
                            card,
                            item,
                            card.id,
                            card.items[card.items.findIndex((entry) => entry.id === item.id) + 2]
                              ?.id,
                          )
                        "
                        >向后移动</ContextMenuItem
                      >
                      <ContextMenuSub v-if="store.cards.length > 1">
                        <ContextMenuSubTrigger>移动到其他卡片</ContextMenuSubTrigger>
                        <ContextMenuSubContent>
                          <ContextMenuItem
                            v-for="destination in store.cards.filter(
                              (entry) => entry.id !== card.id,
                            )"
                            :key="destination.id"
                            @select="moveItem(card, item, destination.id)"
                            >{{ destination.title }}</ContextMenuItem
                          >
                        </ContextMenuSubContent>
                      </ContextMenuSub>
                      <ContextMenuItem @select="removeItem(card, item)">{{
                        item.managed ? '移回原位置' : '移除条目'
                      }}</ContextMenuItem>
                    </ContextMenuContent>
                  </ContextMenu>
                </div>
                <div
                  v-if="batchCardId === card.id"
                  class="mt-2 flex flex-wrap items-center gap-2 border-t px-2 pt-3 pb-1"
                >
                  <span class="text-sm text-muted-foreground"
                    >已选 {{ selectedItems.length }} 项</span
                  >
                  <Button
                    size="sm"
                    variant="ghost"
                    @click="
                      selectedItems =
                        selectedItems.length === card.items.length
                          ? []
                          : card.items.map((item) => item.id)
                    "
                    >{{ selectedItems.length === card.items.length ? '取消全选' : '全选' }}</Button
                  >
                  <Button
                    size="sm"
                    variant="outline"
                    :disabled="!selectedItems.length"
                    @click="batchRemove(card)"
                    ><Check data-icon="inline-start" />移除 / 移回</Button
                  >
                  <DropdownMenu v-if="store.cards.some((entry) => entry.id !== card.id)">
                    <DropdownMenuTrigger as-child
                      ><Button size="sm" variant="outline" :disabled="!selectedItems.length"
                        >移动到卡片</Button
                      ></DropdownMenuTrigger
                    >
                    <DropdownMenuContent align="end"
                      ><DropdownMenuItem
                        v-for="destination in store.cards.filter((entry) => entry.id !== card.id)"
                        :key="destination.id"
                        @click="batchMove(card, destination.id)"
                        >{{ destination.title }}</DropdownMenuItem
                      ></DropdownMenuContent
                    >
                  </DropdownMenu>
                  <Button size="sm" variant="ghost" @click="toggleBatch(card)">完成</Button>
                </div>
                <p
                  v-if="!card.items.length"
                  class="mt-3 rounded-lg border border-dashed border-border p-4 text-center text-sm text-muted-foreground"
                >
                  桌面卡片还是空的：从桌面把文件拖进卡片，或
                  <button class="text-primary underline underline-offset-2" @click="addFiles(card)">
                    选择文件添加</button
                  >。
                </p>
                <div v-if="card.items.length" class="mt-2 flex justify-end">
                  <Button variant="outline" size="sm" @click="addFiles(card)"
                    ><Plus data-icon="inline-start" />添加文件</Button
                  >
                </div>
              </Card>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @select="addFiles(card)">添加文件</ContextMenuItem>
              <ContextMenuItem @select="startRename(card)">编辑卡片</ContextMenuItem>
              <ContextMenuItem @select="toggleBatch(card)">批量管理条目</ContextMenuItem>
              <ContextMenuItem
                @select="
                  deleteTarget = card;
                  deleteOpen = true;
                "
                >删除卡片</ContextMenuItem
              >
            </ContextMenuContent>
          </ContextMenu>
        </div>
      </ScrollArea>
    </div>
    <Dialog v-model:open="createOpen">
      <DialogContent class="max-h-[90vh] overflow-y-auto sm:max-w-md">
        <DialogHeader>
          <DialogTitle>新建桌面卡片</DialogTitle>
          <DialogDescription>填写卡片名称，也可以选择要添加到卡片的文件。</DialogDescription>
        </DialogHeader>
        <div class="space-y-5">
          <div class="space-y-2">
            <label for="create-card-title" class="text-sm font-medium"
              >名称 <span class="text-destructive">*</span></label
            >
            <Input
              id="create-card-title"
              v-model="createTitle"
              placeholder="卡片名称"
              aria-label="卡片名称"
              maxlength="60"
              autofocus
              @keydown.enter="createCard"
            />
          </div>
          <div class="space-y-2">
            <p class="text-sm font-medium">
              文件 <span class="font-normal text-muted-foreground">（可选）</span>
            </p>
            <div class="flex items-center gap-2">
              <Button variant="outline" type="button" @click="chooseCreateFiles">选择文件</Button>
              <span class="min-w-0 truncate text-sm text-muted-foreground">
                {{ createPaths.length ? `已选择 ${createPaths.length} 个文件` : '未选择文件' }}
              </span>
              <Button
                v-if="createPaths.length"
                variant="ghost"
                size="sm"
                class="ml-auto shrink-0"
                @click="createPaths = []"
                >清空</Button
              >
            </div>
            <ul
              v-if="createPaths.length"
              class="max-h-32 space-y-1 overflow-y-auto rounded-md border bg-muted/30 p-2"
              aria-label="已选择的文件"
            >
              <li
                v-for="path in createPaths"
                :key="path"
                class="flex items-center gap-2 rounded px-1 py-1 text-sm"
              >
                <FileText class="size-4 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate" :title="path">{{
                  path.split(/[\\/]/).pop() || path
                }}</span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  class="h-7 shrink-0 px-2"
                  :aria-label="`移除 ${path}`"
                  @click="createPaths = createPaths.filter((item) => item !== path)"
                  >移除</Button
                >
              </li>
            </ul>
          </div>
          <div class="space-y-3">
            <div class="flex items-center justify-between gap-4">
              <div>
                <label for="create-card-icon" class="text-sm font-medium">图标</label>
                <p class="text-xs text-muted-foreground">在卡片标题旁显示图标或 Emoji</p>
              </div>
              <Switch
                id="create-card-icon"
                v-model="createIconEnabled"
                @update:model-value="
                  (enabled: boolean) => {
                    if (enabled && !createIcon && !createEmoji) createIcon = 'layout-grid';
                  }
                "
              />
            </div>
            <DesktopCardIdentityPicker
              v-if="createIconEnabled"
              v-model:icon="createIcon"
              v-model:emoji="createEmoji"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" :disabled="busy" @click="createOpen = false">取消</Button>
          <Button :disabled="busy || !createTitle.trim()" @click="createCard">
            {{ busy ? '保存中…' : '保存' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
    <Dialog v-model:open="renameOpen">
      <DialogContent
        class="max-h-[90vh] overflow-y-auto sm:max-w-md"
        :show-close-button="!busy"
        @escape-key-down="
          (event) => {
            if (busy) event.preventDefault();
          }
        "
        @interact-outside="
          (event) => {
            if (busy) event.preventDefault();
          }
        "
      >
        <DialogHeader>
          <DialogTitle>编辑桌面卡片</DialogTitle>
          <DialogDescription>修改卡片名称、标题栏标识与桌面卡片外观。</DialogDescription>
        </DialogHeader>
        <fieldset :disabled="busy" class="min-w-0 space-y-5">
          <div class="space-y-2">
            <label for="edit-card-title" class="text-sm font-medium"
              >名称 <span class="text-destructive">*</span></label
            >
            <Input
              id="edit-card-title"
              v-model="renameTitle"
              placeholder="卡片名称"
              aria-label="卡片名称"
              maxlength="60"
              @keydown.enter="confirmRename"
            />
          </div>
          <div class="space-y-2">
            <p class="text-sm font-medium">
              文件 <span class="font-normal text-muted-foreground">（可选）</span>
            </p>
            <div class="flex items-center gap-2">
              <Button variant="outline" type="button" @click="chooseEditFiles">选择文件</Button>
              <span class="min-w-0 truncate text-sm text-muted-foreground">{{
                editPaths.length ? `已选择 ${editPaths.length} 个文件` : '可继续添加文件'
              }}</span>
              <Button
                v-if="editPaths.length"
                variant="ghost"
                size="sm"
                class="ml-auto shrink-0"
                @click="editPaths = []"
                >清空</Button
              >
            </div>
            <ul
              v-if="editPaths.length"
              class="max-h-32 space-y-1 overflow-y-auto rounded-md border bg-muted/30 p-2"
              aria-label="已选择的文件"
            >
              <li
                v-for="path in editPaths"
                :key="path"
                class="flex items-center gap-2 rounded px-1 py-1 text-sm"
              >
                <FileText class="size-4 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate" :title="path">{{
                  path.split(/[\\/]/).pop() || path
                }}</span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  class="h-7 shrink-0 px-2"
                  :aria-label="`移除 ${path}`"
                  @click="editPaths = editPaths.filter((item) => item !== path)"
                  >移除</Button
                >
              </li>
            </ul>
          </div>
          <div class="space-y-3">
            <div class="flex items-center justify-between gap-4">
              <div>
                <label for="edit-card-icon" class="text-sm font-medium">图标</label>
                <p class="text-xs text-muted-foreground">在卡片标题旁显示图标或 Emoji</p>
              </div>
              <Switch
                id="edit-card-icon"
                v-model="editIconEnabled"
                @update:model-value="
                  (enabled: boolean) => {
                    if (enabled && !editIcon && !editEmoji) editIcon = 'layout-grid';
                  }
                "
              />
            </div>
            <DesktopCardIdentityPicker
              v-if="editIconEnabled"
              v-model:icon="editIcon"
              v-model:emoji="editEmoji"
              :initial-tab="editEmoji ? 'emoji' : 'icon'"
            />
          </div>
          <div class="space-y-2">
            <p class="text-sm font-medium text-foreground">卡片外观</p>
            <div class="flex items-center gap-2">
              <span class="w-14 shrink-0 text-sm text-muted-foreground">背景</span>
              <div class="flex flex-wrap gap-1.5" role="group" aria-label="卡片背景样式">
                <Button
                  v-for="option in opacityOptions"
                  :key="option.value"
                  type="button"
                  size="sm"
                  :variant="editOpacity === option.value ? 'default' : 'outline'"
                  :aria-pressed="editOpacity === option.value"
                  @click="editOpacity = option.value"
                  >{{ option.label }}</Button
                >
              </div>
            </div>
          </div>
        </fieldset>
        <p
          v-if="editSaveError"
          role="alert"
          class="rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive"
        >
          {{ editSaveError }}
        </p>
        <DialogFooter>
          <Button variant="outline" :disabled="busy" @click="renameOpen = false">取消</Button>
          <Button :disabled="busy || !renameTitle.trim()" @click="confirmRename">
            {{ busy ? '保存中…' : '保存' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
    <ConfirmDialog
      v-model:open="deleteOpen"
      title="删除桌面卡片"
      :description="`将删除桌面卡片「${deleteTarget?.title ?? ''}」；移入的文件会移回原桌面位置，其他引用不受影响。如有同名文件，请先重命名。`"
      confirm-text="删除"
      destructive
      @confirm="removeCard"
    />
  </div>
</template>
