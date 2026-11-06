<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Field, FieldLabel } from '@/components/ui/field';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';

type Mode = 'pomodoro' | 'countdown' | 'stopwatch';
const mode = ref<Mode>('pomodoro');
const phase = ref<'focus' | 'break'>('focus');
const minutes = ref<number | string>(5);
const seconds = ref<number | string>(0);
const started = ref(false);
const running = ref(false);
const complete = ref(false);
const elapsed = ref(0);
const message = ref('');
let startedAt = 0;
let previouslyElapsed = 0;
let ticker: ReturnType<typeof setInterval> | undefined;

const countingDown = computed(() => mode.value !== 'stopwatch');
const duration = computed(() => {
  if (mode.value === 'pomodoro') return (phase.value === 'focus' ? 25 : 5) * 60_000;
  const min = Number(minutes.value);
  const sec = Number(seconds.value);
  if (
    !Number.isInteger(min) ||
    min < 0 ||
    min > 999 ||
    !Number.isInteger(sec) ||
    sec < 0 ||
    sec > 59
  )
    return 0;
  return (min * 60 + sec) * 1000;
});
const display = computed(() => {
  const total = countingDown.value
    ? Math.ceil(Math.max(0, duration.value - elapsed.value) / 1000)
    : Math.floor(elapsed.value / 1000);
  return [Math.floor(total / 3600), Math.floor(total / 60) % 60, total % 60]
    .map((value) => String(value).padStart(2, '0'))
    .join(':');
});

function stopTicker() {
  clearInterval(ticker);
  ticker = undefined;
  running.value = false;
}
function reset() {
  stopTicker();
  started.value = false;
  complete.value = false;
  elapsed.value = 0;
  previouslyElapsed = 0;
  message.value = '';
}
function tick() {
  elapsed.value = previouslyElapsed + Math.max(0, Date.now() - startedAt);
  if (!countingDown.value || elapsed.value < duration.value) return;
  stopTicker();
  complete.value = true;
  const title = mode.value === 'pomodoro' ? phase.value : 'countdown';
  const doneMessage = {
    focus: '专注时间结束。',
    break: '休息时间结束。',
    countdown: '倒计时结束。',
  }[title];
  message.value = doneMessage;
  void invoke('notify_timer_finished', { title }).catch(() => {
    if (complete.value && message.value === doneMessage) {
      message.value = doneMessage + ' 系统通知发送失败。';
    }
  });
}
function toggle() {
  if (running.value) {
    tick();
    previouslyElapsed = elapsed.value;
    stopTicker();
    return;
  }
  if (countingDown.value && duration.value === 0) return;
  if (complete.value) reset();
  started.value = true;
  running.value = true;
  startedAt = Date.now();
  ticker = setInterval(tick, 100);
}
function changeMode(value: unknown) {
  if (value !== 'pomodoro' && value !== 'countdown' && value !== 'stopwatch') return;
  if (value === mode.value) return;
  reset();
  mode.value = value;
  phase.value = 'focus';
}
function switchPhase() {
  reset();
  phase.value = phase.value === 'focus' ? 'break' : 'focus';
}
onUnmounted(stopTicker);
</script>

<template>
  <div class="flex flex-col items-center gap-4 text-center">
    <Tabs :model-value="mode" @update:model-value="changeMode">
      <TabsList>
        <TabsTrigger value="pomodoro">番茄钟</TabsTrigger>
        <TabsTrigger value="countdown">倒计时</TabsTrigger>
        <TabsTrigger value="stopwatch">正计时</TabsTrigger>
      </TabsList>
    </Tabs>
    <div v-if="mode === 'pomodoro'" class="flex items-center gap-3">
      <span class="text-sm text-muted-foreground">{{
        phase === 'focus' ? '专注 · 25 分钟' : '休息 · 5 分钟'
      }}</span>
      <Button variant="outline" size="sm" :disabled="running" @click="switchPhase">
        {{ phase === 'focus' ? '切换到休息' : '切换到专注' }}
      </Button>
    </div>
    <div v-else-if="mode === 'countdown'" class="grid w-full max-w-60 grid-cols-2 gap-3 text-left">
      <Field>
        <FieldLabel for="timer-minutes">分钟</FieldLabel>
        <Input
          id="timer-minutes"
          v-model="minutes"
          type="number"
          min="0"
          max="999"
          step="1"
          :disabled="started"
        />
      </Field>
      <Field>
        <FieldLabel for="timer-seconds">秒</FieldLabel>
        <Input
          id="timer-seconds"
          v-model="seconds"
          type="number"
          min="0"
          max="59"
          step="1"
          :disabled="started"
        />
      </Field>
    </div>
    <p role="timer" aria-label="计时时间" class="font-mono text-5xl tabular-nums">{{ display }}</p>
    <div class="flex gap-2">
      <Button :disabled="!running && countingDown && duration === 0" @click="toggle">
        {{ running ? '暂停' : complete ? '重新开始' : started ? '继续' : '开始' }}
      </Button>
      <Button variant="outline" :disabled="!started" @click="reset">重置</Button>
    </div>
    <p v-if="message" role="status" class="text-sm text-muted-foreground">{{ message }}</p>
  </div>
</template>
