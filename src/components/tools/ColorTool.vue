<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Pipette, Copy, Plus, X } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { colorPalettes } from '@/lib/color-palettes'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Slider } from '@/components/ui/slider'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { readLocal, writeLocal } from '@/lib/local-files'
const hue=ref([260]),saturation=ref([70]),lightness=ref([60]),hex=ref('#9366E0'),colors=ref<string[]>([]),message=ref('')
const paletteTab=ref('classic')
const rgb=computed(()=>{const h=hex.value.replace('#','');return [0,2,4].map(i=>parseInt(h.slice(i,i+2),16))})
function fromHsl(h:number,s:number,l:number){s/=100;l/=100;const a=s*Math.min(l,1-l);const f=(n:number)=>{const k=(n+h/30)%12;return Math.round(255*(l-a*Math.max(-1,Math.min(k-3,9-k,1)))).toString(16).padStart(2,'0')};return '#'+f(0)+f(8)+f(4)}
function updateHsl(){hex.value=fromHsl(hue.value[0]!,saturation.value[0]!,lightness.value[0]!).toUpperCase()}
function setHex(value:string){if(!/^#?[\da-f]{6}$/i.test(value)){message.value='请输入六位 HEX 颜色';return}hex.value=('#'+value.replace('#','')).toUpperCase();const [r,g,b]=rgb.value.map(v=>v/255) as [number,number,number];const max=Math.max(r,g,b),min=Math.min(r,g,b),d=max-min,l=(max+min)/2;let h=0;if(d)h=max===r?((g-b)/d+(g<b?6:0)):max===g?(b-r)/d+2:(r-g)/d+4;hue.value=[Math.round(h*60)];saturation.value=[Math.round(d?d/(1-Math.abs(2*l-1))*100:0)];lightness.value=[Math.round(l*100)];message.value=''}
const formats=computed(()=>({HEX:hex.value,RGB:`rgb(${rgb.value.join(', ')})`,HSL:`hsl(${hue.value[0]}, ${saturation.value[0]}%, ${lightness.value[0]}%)`}))
const related=computed(()=>[0,30,180,210,330].map(offset=>fromHsl((hue.value[0]!+offset)%360,saturation.value[0]!,lightness.value[0]!)))
async function copy(text:string){try{await invoke('clipboard_copy',{text});message.value='已复制 '+text}catch(e){message.value=String(e)}}
async function pick(){const EyeDropper=(window as unknown as {EyeDropper?:new()=>{open:()=>Promise<{sRGBHex:string}>}}).EyeDropper;if(!EyeDropper){message.value='当前系统不支持屏幕取色，请使用颜色滑块或输入 HEX';return}try{const result=await new EyeDropper().open();setHex(result.sRGBHex)}catch(e){if((e as Error).name!=='AbortError')message.value=String(e)}}
async function save(next:string[]){try{await writeLocal('lumo-tool-palette',next);colors.value=next;message.value='色板已保存'}catch(e){message.value=String(e)}}
onMounted(async()=>{try{colors.value=await readLocal<string[]>('lumo-tool-palette')??[];setHex(hex.value)}catch(e){message.value=String(e)}})
</script>
<template>
 <div class="flex flex-col gap-6">
  <div class="grid gap-6 sm:grid-cols-2">
   <div class="flex flex-col gap-4"><div class="flex h-44 items-end justify-between rounded-2xl p-4" :style="{background:hex}"><span class="rounded-lg bg-background/90 px-3 py-2 font-mono text-sm text-foreground">{{hex}}</span><Button variant="secondary" @click="pick"><Pipette data-icon="inline-start"/>屏幕取色</Button></div>
    <FieldGroup><Field><FieldLabel>色相 · {{hue[0]}}°</FieldLabel><Slider v-model="hue" :max="360" @update:model-value="updateHsl"/></Field><Field><FieldLabel>饱和度 · {{saturation[0]}}%</FieldLabel><Slider v-model="saturation" :max="100" @update:model-value="updateHsl"/></Field><Field><FieldLabel>亮度 · {{lightness[0]}}%</FieldLabel><Slider v-model="lightness" :max="100" @update:model-value="updateHsl"/></Field></FieldGroup>
   </div>
   <FieldGroup><Field><FieldLabel for="hex">HEX 颜色</FieldLabel><Input id="hex" :model-value="hex" @change="setHex(($event.target as HTMLInputElement).value)"/></Field><Field v-for="(value,key) in formats" :key="key"><FieldLabel>{{key}}</FieldLabel><div class="flex gap-2"><Input :model-value="value" readonly :aria-label="key"/><Button variant="outline" size="icon" :aria-label="'复制 '+key" @click="copy(value)"><Copy/></Button></div></Field><Button variant="outline" :disabled="colors.includes(hex)" @click="save([...colors,hex])"><Plus data-icon="inline-start"/>收藏到色板</Button></FieldGroup>
  </div>
  <section class="flex flex-col gap-3"><h2 class="text-sm font-semibold">配色参考 · 原色 / 类似色 / 互补色</h2><div class="flex gap-2"><Button v-for="(color,index) in related" :key="index" class="h-14 flex-1 border" :style="{background:color}" :aria-label="color" :title="color" @click="setHex(color)"/></div></section>
  <section class="flex flex-col gap-3" aria-label="颜色色板"><h2 class="text-sm font-semibold">颜色色板</h2>
   <Tabs v-model="paletteTab" class="gap-4"><TabsList class="h-auto w-full flex-wrap justify-start"><TabsTrigger v-for="palette in colorPalettes" :key="palette.id" :value="palette.id">{{palette.name}}</TabsTrigger><TabsTrigger value="favorites">收藏 · {{colors.length}}</TabsTrigger></TabsList>
    <TabsContent v-for="palette in colorPalettes" :key="palette.id" :value="palette.id" class="flex flex-col gap-3"><p class="text-xs text-muted-foreground">{{palette.description}}点击颜色即可选用，也可收藏或复制色值。</p><div class="grid grid-cols-3 gap-2 sm:grid-cols-5"><Button v-for="color in palette.colors" :key="color.hex" variant="outline" class="h-auto min-w-0 flex-col gap-2 p-2" :aria-label="color.name+' '+color.hex" :aria-pressed="hex===color.hex" :title="color.name+' '+color.hex" @click="setHex(color.hex)"><span class="h-12 w-full rounded-md border" :style="{background:color.hex}"/><span class="text-xs">{{color.name}}</span><span class="font-mono text-[10px] text-muted-foreground">{{color.hex}}</span></Button></div></TabsContent>
    <TabsContent value="favorites" class="flex flex-col gap-3"><p v-if="!colors.length" class="text-sm text-muted-foreground">收藏常用颜色，下次打开继续使用。</p><div class="flex flex-wrap gap-3"><div v-for="color in colors" :key="color" class="flex items-center gap-1"><Button variant="outline" @click="setHex(color)"><span class="size-4 rounded border" :style="{background:color}"/>{{color}}</Button><Button variant="ghost" size="icon-sm" :aria-label="'移除 '+color" @click="save(colors.filter(c=>c!==color))"><X/></Button></div></div></TabsContent>
   </Tabs>
  </section>
  <p v-if="message" role="status" class="text-sm text-muted-foreground">{{message}}</p>
 </div>
</template>
