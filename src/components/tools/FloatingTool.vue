<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { readLocal, writeLocal } from '@/lib/local-files'
const text=ref(''),status=ref('正在读取…'),ready=ref(false),fontSize=ref(18)
let timer:ReturnType<typeof setTimeout>|undefined,removeClose:(()=>void)|undefined
let queue=Promise.resolve()
let editVersion=0, disposed=false, closing=ref(false)
function save(){
  const version=editVersion,value={text:text.value,fontSize:fontSize.value}
  status.value='保存中…'
  queue=queue.catch(()=>{}).then(()=>writeLocal('lumo-tool-floating',value)).then(
    ()=>{if(!disposed&&version===editVersion)status.value='已自动保存'},
    e=>{if(!disposed&&version===editVersion)status.value='保存失败：'+String(e);throw e},
  )
  return queue
}
watch([text,fontSize],()=>{if(!ready.value)return;editVersion++;status.value='待保存';if(timer)clearTimeout(timer);timer=setTimeout(()=>void save().catch(()=>{}),400)},{flush:'sync'})
onMounted(async()=>{
  try{
    const value=await readLocal<{text:string;fontSize:number}>('lumo-tool-floating')
    if(disposed)return
    if(value){text.value=value.text;fontSize.value=value.fontSize}
    status.value='已自动保存'
    const unlisten=await getCurrentWindow().onCloseRequested(async event=>{
      event.preventDefault()
      if(closing.value||!ready.value)return
      closing.value=true
      if(timer)clearTimeout(timer)
      try{await save();await getCurrentWindow().destroy()}catch{closing.value=false}
    })
    if(disposed){unlisten();return}
    removeClose=unlisten;ready.value=true
  }catch(e){if(!disposed)status.value=String(e)}
})
onUnmounted(()=>{disposed=true;if(timer)clearTimeout(timer);removeClose?.()})
</script>
<template><div class="flex flex-col gap-3"><Textarea v-model="text" :disabled="!ready||closing" placeholder="写下需要一直看见的内容…" class="min-h-[calc(100dvh-190px)] resize-none border-0 bg-transparent leading-relaxed shadow-none focus-visible:ring-0" :style="{fontSize:fontSize+'px'}" aria-label="悬浮文本"/><div class="flex items-center gap-2"><span role="status" class="flex-1 text-xs text-muted-foreground">{{status}}</span><Button variant="ghost" size="sm" :disabled="!ready||closing||fontSize<=12" aria-label="缩小字号" @click="fontSize-=2">A−</Button><Button variant="ghost" size="sm" :disabled="!ready||closing||fontSize>=40" aria-label="放大字号" @click="fontSize+=2">A+</Button></div><p class="text-xs text-muted-foreground">默认置顶；拖动标题栏移动窗口，点击图钉取消置顶。</p></div></template>
