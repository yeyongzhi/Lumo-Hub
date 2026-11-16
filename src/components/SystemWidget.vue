<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{ kind: 'hardware' | 'memory' }>();
type Overview = {
  cpu: string;
  logical_cores: number;
  architecture: string;
  total_memory: number;
  used_memory: number;
};
const overview = ref<Overview | null>(null);
const error = ref(false);
let timer: ReturnType<typeof setInterval> | undefined;
let disposed = false;
let loading = false;
async function refresh() {
  if (loading || document.hidden) return;
  loading = true;
  try {
    const result = await invoke<Overview>('system_overview');
    if (!disposed) {
      overview.value = result;
      error.value = false;
    }
  } catch {
    if (!disposed) error.value = true;
  } finally {
    loading = false;
  }
}
const percent = computed(() =>
  overview.value?.total_memory
    ? Math.min(100, Math.round((overview.value.used_memory / overview.value.total_memory) * 100))
    : 0,
);
function gib(value: number) {
  return `${(value / 1024 ** 3).toFixed(1)} GB`;
}
onMounted(() => {
  void refresh();
  timer = setInterval(refresh, props.kind === 'memory' ? 5000 : 60000);
});
onUnmounted(() => {
  disposed = true;
  clearInterval(timer);
});
</script>

<template>
  <div v-if="overview" class="flex min-h-0 flex-1 flex-col justify-center gap-2">
    <template v-if="kind === 'memory'">
      <div class="flex items-baseline justify-between gap-2">
        <span class="text-2xl font-semibold tabular-nums text-primary">{{ percent }}%</span
        ><span class="text-xs text-muted-foreground"
          >{{ gib(overview.used_memory) }} / {{ gib(overview.total_memory) }}</span
        >
      </div>
      <div
        class="h-2 overflow-hidden rounded-full bg-muted"
        role="progressbar"
        aria-label="物理内存占用率"
        :aria-valuenow="percent"
        :aria-valuemin="0"
        :aria-valuemax="100"
      >
        <div
          class="h-full rounded-full bg-primary transition-[width]"
          :style="{ width: `${percent}%` }"
        />
      </div>
      <span class="text-xs text-muted-foreground">物理内存 · 每 5 秒更新</span>
    </template>
    <template v-else>
      <p class="line-clamp-2 break-words text-sm font-semibold" :title="overview.cpu">
        {{ overview.cpu }}
      </p>
      <p class="text-xs text-muted-foreground">
        {{ overview.logical_cores }} 个逻辑处理器 · {{ overview.architecture }}
      </p>
      <p class="text-xs text-muted-foreground">物理内存 {{ gib(overview.total_memory) }}</p>
    </template>
  </div>
  <p v-else class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
    {{ error ? '无法读取系统信息' : '正在读取系统信息…' }}
  </p>
</template>
