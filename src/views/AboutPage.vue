<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm'
import { onMounted, onUnmounted, shallowRef, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { Update } from '@tauri-apps/plugin-updater'
import { Download, RefreshCw, ShieldCheck } from '@lucide/vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
const version=ref('读取中…')
const checking=ref(false)
const installing=ref(false)
const progress=ref(0)
const updateAvailable=shallowRef<Update|null>(null)
const status=ref('')
onMounted(async()=>{try{version.value=await getVersion()}catch{version.value='未知版本'}})
async function checkUpdate(){checking.value=true;status.value='';try{await updateAvailable.value?.close();updateAvailable.value=null;const metadata=await invoke<ConstructorParameters<typeof Update>[0]|null>('check_app_update');updateAvailable.value=metadata?new Update(metadata):null;status.value=updateAvailable.value?`发现新版本 ${updateAvailable.value.version}`:'当前已是最新版本。'}catch(error){status.value=/404|not found/i.test(String(error))?'当前发布源尚未提供更新清单，请稍后重试。':`检查更新失败：${String(error)}，可再次点击检查更新重试。`}finally{checking.value=false}}
onUnmounted(()=>{void updateAvailable.value?.close().catch(()=>{})})
async function installUpdate(){if(!updateAvailable.value||!await askConfirmation(`将 Lumo-Hub 更新到 ${updateAvailable.value.version}。用户数据会保留。现在安装吗？`))return;installing.value=true;let received=0;let total=0;try{await updateAvailable.value.downloadAndInstall(event=>{if(event.event==='Started'){total=event.data.contentLength??0}else if(event.event==='Progress'){received+=event.data.chunkLength;if(total>0)progress.value=Math.min(100,Math.round(received/total*100))}else if(event.event==='Finished'){progress.value=100}});await relaunch()}catch(error){status.value=`安装更新失败：${String(error)}`}finally{installing.value=false}}
</script>

<template>

    
    <div class="flex flex-col items-start gap-5 text-left">
      <div class="flex size-20 items-center justify-center rounded-2xl bg-primary text-3xl font-bold text-primary-foreground shadow-lg">L</div>
      <div><h2 class="text-2xl font-semibold">Lumo-Hub</h2><p class="mt-1 text-sm text-muted-foreground">本地优先的个人桌面效率工作台</p></div>
      <Badge>版本 {{ version }}</Badge>
      <div class="flex flex-wrap items-center gap-3"><Button :disabled="checking||installing" @click="checkUpdate"><RefreshCw data-icon="inline-start" />{{ checking?'正在检查…':'检查更新' }}</Button><Button v-if="updateAvailable" variant="outline" :disabled="installing" @click="installUpdate"><Download data-icon="inline-start" />{{ installing?`正在下载 ${progress}%`:'下载并安装' }}</Button></div>
      <p v-if="status" class="max-w-xl text-sm text-muted-foreground">{{ status }}</p>
      <div v-if="updateAvailable" class="w-full max-w-xl rounded-2xl border border-border bg-muted/50 p-4 text-left"><div class="flex items-center gap-2 font-medium"><ShieldCheck class="size-4 text-primary" />版本 {{ updateAvailable.version }}</div><p class="mt-2 whitespace-pre-wrap text-sm text-muted-foreground">{{ updateAvailable.body||updateAvailable.rawJson?.notes||'此版本没有附带更新说明。' }}</p></div>
    </div>
</template>
