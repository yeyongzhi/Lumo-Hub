<script setup lang="ts">
import { ref, watch } from 'vue'
import QRCode from 'qrcode'
import jsQR from 'jsqr'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
const text=ref(''),image=ref(''),decoded=ref(''),message=ref(''),file=ref<HTMLInputElement>()
let generation=0
watch(text,async value=>{const current=++generation;image.value='';message.value='';if(!value)return;try{const result=await QRCode.toDataURL(value,{width:320,margin:3,errorCorrectionLevel:'M'});if(current===generation)image.value=result}catch{if(current===generation)message.value='内容过长，无法生成二维码，请缩短文本。'}})
async function download(){try{const path=await save({defaultPath:'lumo-qrcode.png',filters:[{name:'PNG 图片',extensions:['png']}]});if(path){await invoke('save_tool_png',{path,data:image.value});message.value='图片已保存'}}catch(e){message.value='保存失败：'+String(e)}}
async function readFile(event:Event){const selected=(event.target as HTMLInputElement).files?.[0];if(!selected)return;decoded.value='';message.value='';try{const bitmap=await createImageBitmap(selected);const scale=Math.min(1,2000/Math.max(bitmap.width,bitmap.height));const canvas=document.createElement('canvas');canvas.width=Math.round(bitmap.width*scale);canvas.height=Math.round(bitmap.height*scale);const ctx=canvas.getContext('2d')!;ctx.drawImage(bitmap,0,0,canvas.width,canvas.height);bitmap.close();const data=ctx.getImageData(0,0,canvas.width,canvas.height);const result=jsQR(data.data,data.width,data.height,{inversionAttempts:'attemptBoth'});if(result)decoded.value=result.data;else message.value='未识别到二维码，请选择清晰、完整的二维码图片。'}catch(e){message.value='无法读取图片：'+String(e)}finally{(event.target as HTMLInputElement).value=''}}
async function copy(){try{await invoke('clipboard_copy',{text:decoded.value});message.value='已复制识别结果'}catch(e){message.value=String(e)}}
</script>
<template>
 <div class="flex flex-col gap-6"><FieldGroup><Field><FieldLabel for="qr-text">生成二维码</FieldLabel><Textarea id="qr-text" v-model="text" placeholder="输入链接或文本，实时生成二维码…" class="min-h-28"/></Field></FieldGroup>
 <div v-if="image" class="flex flex-col items-center gap-3"><img :src="image" alt="生成的二维码" class="size-64 rounded-xl"/><Button variant="outline" @click="download">保存 PNG 图片</Button></div>
 <section class="flex flex-col gap-3 rounded-xl border p-4"><h2 class="text-sm font-semibold">从图片识别</h2><p class="text-sm text-muted-foreground">选择包含二维码的图片，离线识别，不会自动打开其中的链接。</p><input ref="file" type="file" accept="image/*" class="hidden" aria-label="选择二维码图片" @change="readFile"/><Button variant="outline" class="self-start" @click="file?.click()">选择图片</Button><Textarea v-if="decoded" :model-value="decoded" readonly aria-label="识别结果"/><Button v-if="decoded" variant="outline" class="self-start" @click="copy">复制识别结果</Button></section>
 <p v-if="message" role="status" class="text-sm text-muted-foreground">{{message}}</p></div>
</template>
