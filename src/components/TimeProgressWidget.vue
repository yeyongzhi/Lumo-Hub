<script setup lang="ts">
import { computed } from 'vue';
import { ProgressIndicator, ProgressRoot } from 'reka-ui';
import { ScrollArea } from '@/components/ui/scroll-area';

const props = defineProps<{ now: Date; compact?: boolean }>();
const periods = computed(() => {
  const now = props.now;
  const year = now.getFullYear();
  const month = now.getMonth();
  const day = now.getDate();
  const today = new Date(year, month, day);
  const tomorrow = new Date(year, month, day + 1);
  const weekStart = new Date(year, month, day - ((now.getDay() + 6) % 7));
  const weekEnd = new Date(year, month, day + 7 - ((now.getDay() + 6) % 7));
  const remainingMinutes = Math.max(0, Math.ceil((tomorrow.getTime() - now.getTime()) / 60000));
  const hours = Math.floor(remainingMinutes / 60);
  const minutes = remainingMinutes % 60;
  const remaining = hours
    ? `还剩 ${hours} 小时${minutes ? ` ${minutes} 分钟` : ''}`
    : `还剩 ${minutes} 分钟`;
  const makePeriod = (label: string, start: Date, end: Date, detail?: string) => ({
    label,
    detail,
    value: Math.max(
      0,
      Math.min(100, ((now.getTime() - start.getTime()) / (end.getTime() - start.getTime())) * 100),
    ),
  });
  return [
    makePeriod('今日', today, tomorrow, remaining),
    makePeriod('本周', weekStart, weekEnd),
    makePeriod('本月', new Date(year, month, 1), new Date(year, month + 1, 1)),
    makePeriod('今年', new Date(year, 0, 1), new Date(year + 1, 0, 1)),
  ];
});
</script>

<template>
  <div v-if="compact" class="flex min-h-0 flex-1 flex-col justify-evenly gap-0.5">
    <div
      v-for="period in periods"
      :key="period.label"
      class="flex min-h-0 items-center gap-1 text-xs"
    >
      <span class="shrink-0 font-medium">{{ period.label }}</span>
      <ProgressRoot
        :model-value="period.value"
        :aria-label="`${period.label}进度`"
        class="relative h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-muted"
      >
        <ProgressIndicator
          class="h-full rounded-full bg-primary"
          :style="{ width: `${period.value}%` }"
        />
      </ProgressRoot>
      <span class="shrink-0 tabular-nums text-muted-foreground"
        >{{ Math.floor(period.value) }}%</span
      >
      <span v-if="period.detail" class="shrink-0 tabular-nums text-muted-foreground">{{
        period.detail.replace('还剩 ', '').replace(' 小时', '时').replace(' 分钟', '分')
      }}</span>
    </div>
  </div>
  <ScrollArea v-else class="min-h-0 flex-1">
    <div class="flex flex-col gap-3 pr-2">
      <div v-for="period in periods" :key="period.label" class="flex flex-col gap-1">
        <div class="flex items-center justify-between gap-2 text-xs">
          <span class="font-medium">{{ period.label }}</span>
          <span class="text-right tabular-nums text-muted-foreground"
            >{{ period.detail ? `${period.detail} · ` : '' }}{{ Math.floor(period.value) }}%</span
          >
        </div>
        <ProgressRoot
          :model-value="period.value"
          :aria-label="`${period.label}进度`"
          class="relative h-2 w-full overflow-hidden rounded-full bg-muted"
        >
          <ProgressIndicator
            class="h-full rounded-full bg-primary transition-[width]"
            :style="{ width: `${period.value}%` }"
          />
        </ProgressRoot>
      </div>
    </div>
  </ScrollArea>
</template>
