<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, X } from '@lucide/vue'
import { Button } from '@/components/ui/button'

const maximized=ref(false)
onMounted(async()=>{try{maximized.value=await getCurrentWindow().isMaximized()}catch{}})
async function minimize(){try{await getCurrentWindow().minimize()}catch{}}
async function toggleMaximize(){try{await getCurrentWindow().toggleMaximize();maximized.value=await getCurrentWindow().isMaximized()}catch{}}
async function closeWindow(){try{await getCurrentWindow().close()}catch{}}
</script>

<template>
  <div class="flex shrink-0 items-center gap-1" data-tauri-drag-region="false">
    <Button variant="ghost" size="icon" aria-label="最小化" title="最小化" @click="minimize"><Minus /></Button>
    <Button variant="ghost" size="icon" :aria-label="maximized?'还原':'最大化'" :title="maximized?'还原':'最大化'" @click="toggleMaximize"><span class="block size-3.5 rounded-[3px] border-2 border-current"></span></Button>
    <Button variant="ghost" size="icon" aria-label="关闭" title="关闭" class="hover:bg-destructive hover:text-white" @click="closeWindow"><X /></Button>
  </div>
</template>
