<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
type Clip={id:string;kind:string;content:string;created:string}
const entries=ref<Clip[]>([]),paused=ref(false),query=ref(''),filter=ref('all'),message=ref('')
const visible=computed(()=>entries.value.filter(e=>(filter.value==='all'||e.kind===filter.value)&&(e.kind==='image'?!query.value:e.content.toLowerCase().includes(query.value.toLowerCase()))))
let unlisten:(()=>void)|undefined
async function refresh(){try{const data=await invoke<{entries:Clip[];paused:boolean}>('clipboard_history');entries.value=data.entries;paused.value=data.paused}catch(e){message.value=String(e)}}
async function pause(value:boolean){try{await invoke('clipboard_pause',{paused:value});paused.value=value}catch(e){message.value=String(e)}}
async function remove(id:string|null){try{await invoke('clipboard_delete',{id});await refresh()}catch(e){message.value=String(e)}}
async function copy(id:string){try{await invoke('clipboard_copy',{id});message.value='已复制，可切换到目标应用粘贴'}catch(e){message.value=String(e)}}
onMounted(async()=>{unlisten=await listen('lumo-clipboard-changed',refresh);await refresh()})
onUnmounted(()=>unlisten?.())
</script>
<template>
 <div class="flex flex-col gap-4"><p class="text-sm text-muted-foreground">应用运行期间自动记录文本和图片，仅保存在本机。最多 200 条 / 32 MB；单条文本 1 MB、图片原始像素 16 MB。</p>
 <div class="flex flex-wrap items-center gap-3"><Input v-model="query" class="min-w-40 flex-1" placeholder="搜索文本历史…" aria-label="搜索剪贴板"/><Switch id="pause" :model-value="paused" @update:model-value="pause"/><Label for="pause">暂停记录</Label><ConfirmDialog title="清空剪贴板历史？" description="删除本工具保存的全部记录，系统剪贴板内容不受影响。" destructive @confirm="remove(null)"><template #trigger><Button variant="outline" :disabled="!entries.length">清空历史</Button></template></ConfirmDialog></div>
 <Tabs v-model="filter"><TabsList><TabsTrigger value="all">全部</TabsTrigger><TabsTrigger value="text">文本</TabsTrigger><TabsTrigger value="image">图片</TabsTrigger></TabsList></Tabs>
 <p role="status" class="text-sm text-muted-foreground">{{message||`共 ${visible.length} 条记录`}}</p>
 <p v-if="!visible.length" class="py-8 text-center text-sm text-muted-foreground">暂无记录，复制一段文本或一张图片试试。</p>
 <article v-for="entry in visible" :key="entry.id" class="flex flex-col gap-3 rounded-xl border bg-card p-4"><div class="flex items-center gap-2"><span class="flex-1 text-xs text-muted-foreground">{{entry.kind==='image'?'图片':'文本'}} · {{new Date(entry.created).toLocaleString()}}</span><Button size="sm" variant="outline" @click="copy(entry.id)">复制</Button><Button size="sm" variant="ghost" @click="remove(entry.id)">删除</Button></div><img v-if="entry.kind==='image'" :src="entry.content" alt="剪贴板图片" class="max-h-72 max-w-full self-start rounded-lg object-contain"/><pre v-else class="max-h-48 overflow-hidden whitespace-pre-wrap break-all text-sm">{{entry.content}}</pre></article>
 </div>
</template>
