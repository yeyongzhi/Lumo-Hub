<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { Pin, Minus, X } from '@lucide/vue'
import { builtinTools } from '@/lib/tools'
import { api } from '@/lib/api'
import { applySettings, watchSystemTheme } from '@/lib/theme'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
const CalendarTool=defineAsyncComponent(()=>import('@/components/tools/CalendarTool.vue'))
const ColorTool=defineAsyncComponent(()=>import('@/components/tools/ColorTool.vue'))
const ClipboardTool=defineAsyncComponent(()=>import('@/components/tools/ClipboardTool.vue'))
const QrTool=defineAsyncComponent(()=>import('@/components/tools/QrTool.vue'))
const TextTool=defineAsyncComponent(()=>import('@/components/tools/TextTool.vue'))
const FloatingTool=defineAsyncComponent(()=>import('@/components/tools/FloatingTool.vue'))
const TimestampTool=defineAsyncComponent(()=>import('@/components/tools/TimestampTool.vue'))
const id=new URLSearchParams(location.search).get('tool')??'color'
const tool=builtinTools.find(t=>t.id===id)??builtinTools[0]
const components={calendar:CalendarTool,color:ColorTool,clipboard:ClipboardTool,qrcode:QrTool,text:TextTool,floating:FloatingTool,timestamp:TimestampTool}
const pinned=ref(id==='floating'),error=ref('')
const removers:(()=>void)[]=[]
async function pin(){try{await getCurrentWindow().setAlwaysOnTop(!pinned.value);pinned.value=!pinned.value}catch(e){error.value=String(e)}}
function drag(event:MouseEvent){if(event.button===0&&!(event.target as HTMLElement).closest('button'))void getCurrentWindow().startDragging()}
onMounted(async()=>{
 try{let settings=(await api.bootstrap()).settings;applySettings(settings);removers.push(watchSystemTheme(()=>settings));removers.push(await listen('lumo-settings-changed',async()=>{settings=(await api.bootstrap()).settings;applySettings(settings)}))}catch(e){error.value=String(e)}
})
onUnmounted(()=>removers.forEach(remove=>remove()))
</script>
<template>
 <div class="flex h-dvh min-w-0 flex-col bg-background text-foreground">
  <header class="flex h-14 shrink-0 items-center gap-3 border-b px-4" @mousedown="drag">
   <component :is="tool.icon" class="size-5 text-primary"/><h1 class="flex-1 text-sm font-semibold">{{tool.title}}</h1>
   <Button variant="ghost" size="icon-sm" :aria-pressed="pinned" :title="pinned?'取消置顶':'置顶窗口'" aria-label="切换置顶" @click="pin"><Pin :class="pinned?'text-primary':''"/></Button>
   <Button variant="ghost" size="icon-sm" aria-label="最小化" @click="getCurrentWindow().minimize()"><Minus/></Button>
   <Button variant="ghost" size="icon-sm" aria-label="关闭" @click="getCurrentWindow().close()"><X/></Button>
  </header>
  <p v-if="error" role="alert" class="px-5 pt-3 text-sm text-destructive">{{error}}</p>
  <ScrollArea class="min-h-0 flex-1"><div class="p-5"><component :is="components[tool.id]"/></div></ScrollArea>
 </div>
</template>
