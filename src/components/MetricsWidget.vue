<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { useWorkspaceStore } from '@/stores/workspace';
const props = defineProps<{ kind: 'disk' | 'cpu' | 'network' }>();
const store = useWorkspaceStore();
type Disk = { name: string; path: string; total: number; available: number };
type Metrics = {
  cpu: number;
  download: number;
  upload: number;
  interfaces: number;
  connected: boolean | null;
  disks: Disk[];
};
const metrics = ref<Metrics | null>(null);
const cpuName = ref('');
const history = ref<number[]>([]);
const error = ref(false);
let timer: ReturnType<typeof setInterval> | undefined;
let disposed = false;
let loading = false;
async function refresh() {
  if (loading || document.hidden) return;
  loading = true;
  try {
    const value = await invoke<Metrics>('home_metrics');
    if (!disposed) {
      metrics.value = value;
      if (props.kind === 'cpu') history.value = [...history.value, value.cpu].slice(-20);
      error.value = false;
    }
  } catch {
    if (!disposed) error.value = true;
  } finally {
    loading = false;
  }
}
const bars = computed(() => history.value.map((value) => Math.max(5, value)));
function bytes(value: number) {
  if (value >= 1024 ** 3) return `${(value / 1024 ** 3).toFixed(1)} GB`;
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB`;
  if (value >= 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${Math.round(value)} B`;
}
async function openNetwork() {
  try {
    await invoke('open_network_settings');
  } catch (e) {
    store.notify(String(e));
  }
}
async function openDisk(disk: Disk) {
  try {
    await invoke('open_home_disk', { path: disk.path });
  } catch (e) {
    store.notify(String(e));
  }
}
onMounted(() => {
  void refresh();
  if (props.kind === 'cpu')
    void invoke<{ cpu: string }>('system_overview')
      .then((value) => {
        if (!disposed) cpuName.value = value.cpu;
      })
      .catch(() => {});
  timer = setInterval(refresh, 5000);
});
onUnmounted(() => {
  disposed = true;
  clearInterval(timer);
});
</script>
<template>
  <div v-if="metrics" class="flex min-h-0 flex-1 flex-col justify-center gap-2">
    <template v-if="kind === 'cpu'">
      <div class="flex items-end justify-between gap-2">
        <span class="text-2xl font-semibold tabular-nums text-primary"
          >{{ Math.round(metrics.cpu) }}%</span
        >
        <div class="flex h-8 items-end gap-0.5" aria-label="CPU 使用率近期趋势">
          <span
            v-for="(bar, index) in bars"
            :key="index"
            class="w-1 rounded-sm bg-primary/60"
            :style="{ height: `${bar}%` }"
          />
        </div>
      </div>
      <p class="line-clamp-2 text-xs text-muted-foreground" :title="cpuName">
        {{ cpuName || '处理器使用率' }}
      </p>
    </template>
    <template v-else-if="kind === 'network'">
      <p class="text-xs text-muted-foreground">
        {{
          metrics.connected === null
            ? '连接状态不可用'
            : metrics.connected
              ? '系统报告已连接'
              : '系统报告未连接'
        }}
        · {{ metrics.interfaces }} 个网络接口
      </p>
      <div class="flex flex-wrap gap-x-5 gap-y-1 text-sm tabular-nums">
        <span>↓ {{ bytes(metrics.download) }}/s</span><span>↑ {{ bytes(metrics.upload) }}/s</span>
      </div>
      <Button variant="ghost" size="sm" class="h-7 self-start px-1 text-xs" @click="openNetwork"
        >查看网络设置</Button
      >
    </template>
    <template v-else>
      <div v-if="metrics.disks.length" class="flex min-h-0 flex-col gap-1 overflow-y-auto">
        <Button
          v-for="disk in metrics.disks"
          :key="disk.path"
          variant="ghost"
          class="h-auto min-w-0 justify-between gap-2 px-2 py-1 text-left"
          :title="`打开 ${disk.path}`"
          @click="openDisk(disk)"
          ><span class="min-w-0 truncate">{{ disk.path }}</span
          ><span class="shrink-0 text-xs text-muted-foreground"
            >已用 {{ bytes(disk.total - disk.available) }} · 剩余 {{ bytes(disk.available) }}</span
          ></Button
        >
      </div>
      <p v-else class="text-sm text-muted-foreground">没有可用磁盘</p>
    </template>
  </div>
  <p v-else class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
    {{ error ? '无法读取系统状态' : '正在读取系统状态…' }}
  </p>
</template>
