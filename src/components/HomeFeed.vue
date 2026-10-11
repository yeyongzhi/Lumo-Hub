<script setup lang="ts">
import { onMounted, onBeforeUnmount, computed, ref, watch } from 'vue'
import { MapPin, Check } from '@lucide/vue'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { readLocal, writeLocal } from '@/lib/local-files'
import { api } from '@/lib/api'
import { parseFeed, type FeedData, type FeedKind } from '@/lib/home-feeds'
const props = defineProps<{ kind: FeedKind; refreshKey: number }>()
const emit = defineEmits<{ metadata: [value: string]; updated: [value: string] }>()
const data = ref<FeedData | null>(null)
watch(data, value => emit('metadata', value?.date || ''))
const loading = ref(false), error = ref(''), city = ref(''), cityDraft = ref(city.value), changingCity = ref(false)
const offline=ref(!navigator.onLine)
const currentDay=ref(new Date())
let dayTimer:ReturnType<typeof setInterval>|undefined
function networkChanged(){offline.value=!navigator.onLine;if(!offline.value)void load(true)}
const newsAge=computed(()=>{if(props.kind!=='news'||!data.value?.date)return '';const today=new Date(currentDay.value.getFullYear(),currentDay.value.getMonth(),currentDay.value.getDate());const date=new Date(data.value.date+'T00:00:00');const days=Math.round((today.getTime()-date.getTime())/86400000);return days===1?'昨日新闻':days>1?'历史新闻':''})
let version = 0
const loadedAt = ref(0)
watch(loadedAt, value => emit('updated', value ? new Date(value).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'})+' 更新' : ''))
const cacheKey = () => `lumo-feed-${props.kind}-${props.kind === 'weather' ? city.value : ''}`
async function readCache() {
  data.value = null; loadedAt.value = 0
  try {
    const cached = await readLocal<{raw:unknown;time:number}>(cacheKey())
    if (cached?.raw && typeof cached.time === 'number') { data.value = parseFeed(props.kind, cached.raw); loadedAt.value = cached.time }
  } catch { /* Invalid cache is replaced by the next successful request. */ }
}
async function load(force = false) {
  if (loading.value) return
  const lifetime = props.kind === 'weather' ? 30 * 60_000 : props.kind === 'news' ? 6 * 60 * 60_000 : 60 * 60_000
  const now = new Date()
  const today = now.getFullYear()+'-'+String(now.getMonth()+1).padStart(2,'0')+'-'+String(now.getDate()).padStart(2,'0')
  if (!force && data.value && Date.now() - loadedAt.value < lifetime && (props.kind !== 'news' || data.value.date === today)) return
  const request = ++version
  loading.value = true; error.value = ''
  try {
    const raw = await api.fetchHomeFeed(props.kind, city.value)
    const parsed = parseFeed(props.kind, raw)
    if (request !== version) return
    data.value = parsed; loadedAt.value = Date.now()
    try { await writeLocal(cacheKey(), { raw, time: loadedAt.value }) } catch { /* Cache is optional. */ }
  } catch (reason) { if (request === version) error.value = String(reason instanceof Error ? reason.message : reason) }
  finally { if (request === version) loading.value = false }
}
async function saveCity() {
  city.value = cityDraft.value.trim(); await writeLocal('lumo-weather-city', city.value); changingCity.value = false
  version++; loading.value = false; await readCache(); void load(true)
}
watch(() => props.refreshKey, () => void load(true))
onMounted(async () => {window.addEventListener('online',networkChanged);window.addEventListener('offline',networkChanged);dayTimer=setInterval(()=>{currentDay.value=new Date()},60000);const savedCity=await readLocal<string>('lumo-weather-city');city.value=savedCity||'';cityDraft.value=city.value;changingCity.value=props.kind==='weather'&&savedCity===null;await readCache(); void load() })
onBeforeUnmount(() => { version++;if(dayTimer)clearInterval(dayTimer);window.removeEventListener('online',networkChanged);window.removeEventListener('offline',networkChanged) })
</script>
<template>
  <div class="flex flex-1 flex-col gap-2">
    <template v-if="kind === 'weather'">
      <div class="grid flex-1 grid-cols-[minmax(0,1fr)_auto] items-center gap-4">
        <div class="flex min-w-0 flex-col gap-1">
          <form v-if="changingCity" class="flex items-center gap-2" @submit.prevent="saveCity"><Input v-model="cityDraft" aria-label="天气城市" placeholder="输入城市名，留空按网络位置定位" :maxlength="80" /><Button type="submit" size="icon" aria-label="保存天气城市"><Check /></Button></form>
          <Button v-else variant="ghost" size="sm" class="h-auto max-w-full justify-start self-start px-0" @click="changingCity=true"><MapPin data-icon="inline-start" /><span class="truncate">{{ city || data?.location || '按网络位置定位' }}</span></Button>
          <p v-if="data" class="text-4xl font-semibold tabular-nums">{{ data.temperature }}<span class="text-xl">°C</span></p>
          <p v-if="data" class="text-sm leading-5"><span class="font-medium">{{ data.content }}</span><span class="ml-3 text-muted-foreground">体感 {{ data.feelsLike }}°C</span></p>
        </div>
        <div v-if="data" class="flex flex-col gap-2 text-right text-sm text-muted-foreground"><p>湿度 {{ data.humidity }}%</p><p>风速 {{ data.wind }} km/h</p></div>
      </div>
    </template>
    <template v-else-if="data && kind === 'news'">
      <ol class="flex flex-col gap-3"><li v-for="(line,index) in data.news" :key="index" class="flex items-start gap-3 text-sm leading-6"><span class="shrink-0 text-xs font-semibold leading-6 tabular-nums text-primary">{{ String(index+1).padStart(2,'0') }}</span><span>{{ line }}</span></li></ol>
    </template>
    <template v-else-if="data">
      <p class="whitespace-pre-wrap break-words leading-7" :class="kind==='poetry'?'text-lg font-medium':'text-sm'">{{ data.content }}</p>
    </template>
    <p v-if="loading" role="status" class="text-xs text-muted-foreground">{{ data?'正在刷新…':'正在获取内容…' }}</p>
    <details v-if="error" class="shrink-0 text-sm leading-6 text-muted-foreground"><summary class="cursor-pointer">{{ data?'暂时无法更新，已保留上次内容':'暂时无法获取内容' }} · 点击右上角重试</summary><p class="mt-1 break-words text-xs">{{ error }}</p></details>
    <div v-if="data&&(kind!=='weather'||offline||error)" class="mt-auto flex shrink-0 flex-wrap items-baseline justify-end gap-x-4 gap-y-2 text-sm leading-6 text-muted-foreground" :class="kind==='news'?'border-t border-border pt-3':kind==='weather'?'pt-1':'pt-3'">
      <span v-if="newsAge||offline||error" class="text-sm">{{ newsAge }}{{ newsAge&&(offline||error)?' · ':'' }}{{ offline?'离线内容':error?'缓存内容':'' }}</span>
      <p v-if="kind==='news'&&data.content" class="min-w-0 flex-1">{{ data.content }}</p>
      <p v-if="['hitokoto','poetry'].includes(kind)&&(data.author||data.source)" class="ml-auto min-w-0 text-right text-sm leading-6 text-muted-foreground">{{ [data.author,data.source].filter(Boolean).join(' · ') }}</p>
      <p v-if="kind=== 'news'" class="ml-auto shrink-0 text-right">{{ new Date(loadedAt).toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'}) }} 更新</p>
    </div>
  </div>
</template>
