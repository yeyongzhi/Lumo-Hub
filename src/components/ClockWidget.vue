<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ChevronLeft, ChevronRight } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { formatLongDate } from '@/lib/dates';
import type { HomeWidgetSize } from '@/lib/types';
defineProps<{ size: HomeWidgetSize }>();
const now = ref(new Date());
let timer: ReturnType<typeof setInterval>;
onMounted(() => {
  timer = setInterval(() => {
    now.value = new Date();
  }, 1000);
});
onUnmounted(() => clearInterval(timer));
const time = computed(() =>
  now.value.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
);
const longDate = computed(() => formatLongDate(now.value));
const clockCalendarView = ref(new Date(now.value.getFullYear(), now.value.getMonth(), 1));
const clockCalendarMonth = computed(
  () => `${clockCalendarView.value.getFullYear()} 年 ${clockCalendarView.value.getMonth() + 1} 月`,
);
const clockCalendarIsCurrent = computed(
  () =>
    clockCalendarView.value.getFullYear() === now.value.getFullYear() &&
    clockCalendarView.value.getMonth() === now.value.getMonth(),
);
function moveClockCalendar(delta: number) {
  clockCalendarView.value = new Date(
    clockCalendarView.value.getFullYear(),
    clockCalendarView.value.getMonth() + delta,
    1,
  );
}
function clockCalendarToday() {
  clockCalendarView.value = new Date(now.value.getFullYear(), now.value.getMonth(), 1);
}
const currentMonthDays = computed(() => {
  const year = clockCalendarView.value.getFullYear(),
    month = clockCalendarView.value.getMonth();
  const offset = (new Date(year, month, 1).getDay() + 6) % 7,
    days = new Date(year, month + 1, 0).getDate();
  return Array.from({ length: Math.ceil((offset + days) / 7) * 7 }, (_, index) => {
    const day = index - offset + 1;
    return day > 0 && day <= days ? day : null;
  });
});
</script>
<template>
  <div
    class="min-h-0 min-w-0 flex-1 gap-4"
    :class="
      size === '2x2'
        ? 'clock-expanded grid grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] items-stretch'
        : 'flex flex-col'
    "
  >
    <div
      class="flex min-w-0 flex-col justify-center"
      :class="
        size === '2x2'
          ? 'items-start gap-4 border-r border-border/60 px-2 pr-4'
          : 'w-full flex-1 gap-1'
      "
    >
      <p
        class="text-2xl font-semibold tabular-nums tracking-tight @min-[20rem]:text-3xl @min-[32rem]:text-4xl"
      >
        {{ time }}
      </p>
      <div
        v-if="size === '2x2'"
        class="flex shrink-0 flex-col items-start gap-1.5 text-sm text-muted-foreground"
      >
        <span>{{ now.getFullYear() }} 年 {{ now.getMonth() + 1 }} 月 {{ now.getDate() }} 日</span
        ><span>{{ now.toLocaleDateString('zh-CN', { weekday: 'long' }) }}</span>
      </div>
      <p v-else class="text-sm text-muted-foreground">{{ longDate }}</p>
    </div>
    <div
      v-if="size === '2x2'"
      class="flex min-h-0 min-w-0 flex-1 flex-col rounded-xl bg-muted/30 p-2"
      :aria-label="`${clockCalendarMonth}日历`"
    >
      <div class="flex shrink-0 items-center justify-between gap-1">
        <Button variant="ghost" size="icon-sm" aria-label="上个月" @click="moveClockCalendar(-1)"
          ><ChevronLeft /></Button
        ><span class="text-sm font-semibold tabular-nums">{{ clockCalendarMonth }}</span>
        <div class="flex items-center">
          <Button
            variant="ghost"
            size="sm"
            class="px-2 text-xs"
            :disabled="clockCalendarIsCurrent"
            @click="clockCalendarToday"
            >今天</Button
          ><Button variant="ghost" size="icon-sm" aria-label="下个月" @click="moveClockCalendar(1)"
            ><ChevronRight
          /></Button>
        </div>
      </div>
      <div class="grid min-h-0 flex-1 grid-cols-7 auto-rows-fr text-center text-xs">
        <span
          v-for="day in ['一', '二', '三', '四', '五', '六', '日']"
          :key="day"
          class="flex items-center justify-center text-muted-foreground"
          >{{ day }}</span
        ><span
          v-for="(day, index) in currentMonthDays"
          :key="index"
          class="flex min-h-7 items-center justify-center"
          ><span
            v-if="day"
            class="flex size-7 items-center justify-center rounded-full tabular-nums"
            :class="
              clockCalendarIsCurrent && day === now.getDate()
                ? 'bg-primary font-semibold text-primary-foreground'
                : 'transition-colors hover:bg-muted'
            "
            >{{ day }}</span
          ></span
        >
      </div>
    </div>
  </div>
</template>
<style scoped>
@container (max-width: 25rem) {
  .clock-expanded {
    grid-template-columns: minmax(0, 1fr);
    gap: 0.5rem;
  }
  .clock-expanded > :first-child {
    border-right: 0;
    padding: 0.25rem;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .clock-expanded > :first-child > p {
    font-size: 1.5rem;
  }
  .clock-expanded > :first-child > div {
    font-size: 0.75rem;
    gap: 0;
  }
}
</style>
