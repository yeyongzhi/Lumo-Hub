<script setup lang="ts">
import { onMounted, onBeforeUnmount, computed, ref, watch } from 'vue';
import { MapPin, Check } from '@lucide/vue';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { readLocal, writeLocal } from '@/lib/local-files';
import { api } from '@/lib/api';
import { parseFeed, type FeedData, type FeedKind } from '@/lib/home-feeds';
import type { HomeWidgetSize } from '@/lib/types';
const props = defineProps<{ kind: FeedKind; refreshKey: number; size: HomeWidgetSize }>();
const emit = defineEmits<{
  metadata: [value: string];
  updated: [value: string];
  loading: [value: boolean];
}>();
const data = ref<FeedData | null>(null);
watch(data, (value) => emit('metadata', value?.date || ''));
const loading = ref(false),
  error = ref(''),
  city = ref(''),
  cityDraft = ref(city.value),
  changingCity = ref(false);
watch(loading, (value) => emit('loading', value), { immediate: true });
const offline = ref(!navigator.onLine);
const currentDay = ref(new Date());
let dayTimer: ReturnType<typeof setInterval> | undefined;
function networkChanged() {
  offline.value = !navigator.onLine;
  if (!offline.value) void load(true);
}
const newsAge = computed(() => {
  if (props.kind !== 'news' || !data.value?.date) return '';
  const today = new Date(
    currentDay.value.getFullYear(),
    currentDay.value.getMonth(),
    currentDay.value.getDate(),
  );
  const date = new Date(data.value.date + 'T00:00:00');
  const days = Math.round((today.getTime() - date.getTime()) / 86400000);
  return days === 1 ? '昨日新闻' : days > 1 ? '历史新闻' : '';
});
let disposed = false;
let version = 0;
const loadedAt = ref(0);
watch(loadedAt, (value) =>
  emit(
    'updated',
    value
      ? new Date(value).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }) +
          ' 更新'
      : '',
  ),
);
const stale = computed(
  () =>
    !!loadedAt.value &&
    currentDay.value.getTime() - loadedAt.value >
      (props.kind === 'weather'
        ? 30 * 60_000
        : props.kind === 'news'
          ? 6 * 60 * 60_000
          : 60 * 60_000),
);
const feedStatus = computed(() => {
  if (loading.value) return data.value ? '正在更新' : '正在加载';
  if (offline.value) return data.value ? '离线 · 显示缓存' : '当前离线，暂无缓存';
  if (error.value) return data.value ? '更新失败 · 显示缓存' : '加载失败';
  if (stale.value && data.value) return '缓存可能已过期';
  if (!data.value) return '暂无内容';
  return '';
});
const cacheKey = () => `lumo-feed-${props.kind}-${props.kind === 'weather' ? city.value : ''}`;
async function readCache() {
  data.value = null;
  loadedAt.value = 0;
  try {
    const cached = await readLocal<{ raw: unknown; time: number }>(cacheKey());
    if (cached?.raw && typeof cached.time === 'number') {
      data.value = parseFeed(props.kind, cached.raw);
      loadedAt.value = cached.time;
    }
  } catch {
    /* Invalid cache is replaced by the next successful request. */
  }
}
async function load(force = false) {
  if (disposed || loading.value) return;
  const lifetime =
    props.kind === 'weather' ? 30 * 60_000 : props.kind === 'news' ? 6 * 60 * 60_000 : 60 * 60_000;
  const now = new Date();
  const today =
    now.getFullYear() +
    '-' +
    String(now.getMonth() + 1).padStart(2, '0') +
    '-' +
    String(now.getDate()).padStart(2, '0');
  if (
    !force &&
    data.value &&
    Date.now() - loadedAt.value < lifetime &&
    (props.kind !== 'news' || data.value.date === today)
  )
    return;
  const request = ++version;
  loading.value = true;
  error.value = '';
  try {
    const raw = await api.fetchHomeFeed(props.kind, city.value);
    const parsed = parseFeed(props.kind, raw);
    if (request !== version) return;
    data.value = parsed;
    loadedAt.value = Date.now();
    try {
      await writeLocal(cacheKey(), { raw, time: loadedAt.value });
    } catch {
      /* Cache is optional. */
    }
  } catch (reason) {
    if (request === version)
      error.value = String(reason instanceof Error ? reason.message : reason);
  } finally {
    if (request === version) loading.value = false;
  }
}
async function saveCity() {
  const nextCity = cityDraft.value.trim();
  try {
    await writeLocal('lumo-weather-city', nextCity);
  } catch (reason) {
    error.value = String(reason);
    return;
  }
  city.value = nextCity;
  changingCity.value = false;
  version++;
  loading.value = false;
  await readCache();
  void load(true);
}
watch(
  () => props.refreshKey,
  () => void load(true),
);
onMounted(async () => {
  window.addEventListener('online', networkChanged);
  window.addEventListener('offline', networkChanged);
  dayTimer = setInterval(() => {
    currentDay.value = new Date();
    void load();
  }, 60000);
  if (props.kind === 'weather') {
    try {
      const savedCity = await readLocal<string>('lumo-weather-city');
      if (disposed) return;
      city.value = typeof savedCity === 'string' ? savedCity : '';
      cityDraft.value = city.value;
      changingCity.value = savedCity === null;
    } catch {
      /* Location lookup still works without saved preferences. */
    }
  }
  if (disposed) return;
  await readCache();
  void load();
});
onBeforeUnmount(() => {
  disposed = true;
  version++;
  if (dayTimer) clearInterval(dayTimer);
  window.removeEventListener('online', networkChanged);
  window.removeEventListener('offline', networkChanged);
});
</script>
<template>
  <div class="flex min-h-full flex-1 flex-col gap-2">
    <div
      v-if="feedStatus"
      role="status"
      class="flex shrink-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground"
    >
      <span>{{ feedStatus }}</span>
      <Button
        v-if="!loading && (error || offline)"
        variant="outline"
        size="sm"
        class="h-6 px-2 text-xs"
        @click="load(true)"
        >重试</Button
      >
    </div>
    <template v-if="kind === 'weather'">
      <div
        class="flex flex-1 flex-wrap items-center justify-between gap-x-4 gap-y-2"
        :class="size.endsWith('x2') ? 'content-center gap-y-4' : ''"
      >
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <form
            v-if="changingCity"
            class="flex min-w-0 items-center gap-2"
            @submit.prevent="saveCity"
          >
            <Input
              v-model="cityDraft"
              aria-label="天气城市"
              placeholder="输入城市名，留空按网络位置定位"
              :maxlength="80"
            /><Button type="submit" size="icon" aria-label="保存天气城市"><Check /></Button>
          </form>
          <Button
            v-else
            variant="ghost"
            size="sm"
            class="h-auto max-w-full justify-start self-start px-0"
            @click="changingCity = true"
            ><MapPin data-icon="inline-start" /><span class="truncate">{{
              city || data?.location || '按网络位置定位'
            }}</span></Button
          >
          <p
            v-if="data"
            class="text-xl font-semibold tracking-tight tabular-nums @min-[11rem]:text-3xl @min-[20rem]:text-4xl"
          >
            {{ data.temperature }}<span class="text-xl">°C</span>
          </p>
          <p v-if="data" class="text-sm leading-5">
            <span class="font-medium">{{ data.content }}</span
            ><span class="ml-3 text-muted-foreground">体感 {{ data.feelsLike }}°C</span>
          </p>
        </div>
        <div
          v-if="data"
          class="flex flex-wrap gap-x-3 gap-y-1 rounded-xl bg-muted/40 px-3 py-2 text-xs text-muted-foreground @min-[24rem]:flex-col @min-[24rem]:text-right"
        >
          <p>湿度 {{ data.humidity }}%</p>
          <p>风速 {{ data.wind }} km/h</p>
        </div>
      </div>
    </template>
    <template v-else-if="data && kind === 'news'">
      <ol class="flex flex-col gap-1">
        <li
          v-for="(line, index) in data.news"
          :key="index"
          class="flex items-start gap-3 rounded-lg px-1 py-1 text-sm leading-6"
        >
          <span class="shrink-0 text-xs font-semibold leading-6 tabular-nums text-primary">{{
            String(index + 1).padStart(2, '0')
          }}</span
          ><span class="min-w-0 break-words">{{ line }}</span>
        </li>
      </ol>
    </template>
    <template v-else-if="data">
      <p
        class="whitespace-pre-wrap break-words text-sm leading-7 text-foreground/90 @min-[24rem]:text-base"
        :class="size.endsWith('x2') ? 'my-auto' : ''"
      >
        {{ data.content }}
      </p>
    </template>
    <p v-if="data && (stale || offline || error) && loadedAt" class="text-xs text-muted-foreground">
      上次更新：{{ new Date(loadedAt).toLocaleString('zh-CN') }}
    </p>
    <details v-if="error" class="shrink-0 text-xs text-muted-foreground">
      <summary class="cursor-pointer">查看原因</summary>
      <p class="mt-1 break-words">{{ error }}</p>
    </details>
    <div
      v-if="data && (kind !== 'weather' || offline || error)"
      class="mt-auto flex shrink-0 flex-wrap items-baseline justify-end gap-x-4 gap-y-2 text-sm leading-6 text-muted-foreground"
      :class="
        kind === 'news' ? 'border-t border-border pt-3' : kind === 'weather' ? 'pt-1' : 'pt-3'
      "
    >
      <span v-if="newsAge" class="text-xs">{{ newsAge }}</span>
      <p v-if="kind === 'news' && data.content" class="min-w-0 flex-1">{{ data.content }}</p>
      <p
        v-if="['hitokoto', 'poetry'].includes(kind) && (data.author || data.source)"
        class="ml-auto min-w-0 text-right text-sm leading-6 text-muted-foreground"
      >
        {{ [data.author, data.source].filter(Boolean).join(' · ') }}
      </p>
      <p v-if="kind === 'news'" class="ml-auto shrink-0 text-right">
        {{ new Date(loadedAt).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }) }}
        更新
      </p>
    </div>
  </div>
</template>
