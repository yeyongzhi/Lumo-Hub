<script setup lang="ts">
import { computed, ref } from 'vue'
import { Solar, HolidayUtil } from 'lunar-typescript'
import { addDays, addMonths, format, getDayOfYear, getISOWeek, startOfMonth, startOfWeek } from 'date-fns'
import { ChevronLeft, ChevronRight, CalendarDays } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import { useCurrentDay } from '@/lib/use-current-day'

const now=useCurrentDay()
const selected=ref(new Date())
const visible=ref(startOfMonth(selected.value))
const year=computed({get:()=>String(visible.value.getFullYear()),set:value=>{visible.value=new Date(Number(value),visible.value.getMonth(),1)}})
const month=computed({get:()=>String(visible.value.getMonth()+1),set:value=>{visible.value=new Date(visible.value.getFullYear(),Number(value)-1,1)}})
const years=Array.from({length:201},(_,i)=>1900+i)
const cells=computed(()=>{
  const start=startOfWeek(visible.value)
  return Array.from({length:42},(_,index)=>{
    const date=addDays(start,index), solar=Solar.fromDate(date), lunar=solar.getLunar()
    const holiday=HolidayUtil.getHoliday(solar.toYmd())
    const festival=[...solar.getFestivals(),...lunar.getFestivals()][0]
    const term=lunar.getJieQi()
    return {date,key:solar.toYmd(),day:date.getDate(),current:date.getMonth()===visible.value.getMonth(),weekend:date.getDay()===0||date.getDay()===6,holiday,label:term||festival||(lunar.getDay()===1?lunar.getMonthInChinese()+'月':lunar.getDayInChinese()),special:!!(term||festival)}
  })
})
const solar=computed(()=>Solar.fromDate(selected.value))
const lunar=computed(()=>solar.value.getLunar())
const selectedKey=computed(()=>format(selected.value,'yyyy-MM-dd'))
const todayKey=computed(()=>format(now.value,'yyyy-MM-dd'))
const festivals=computed(()=>[...solar.value.getFestivals(),...lunar.value.getFestivals(),lunar.value.getJieQi()].filter(Boolean))
const holiday=computed(()=>HolidayUtil.getHoliday(selectedKey.value))
const positions=computed(()=>[
  ['喜神',lunar.value.getPositionXiDesc()],['阳贵神',lunar.value.getPositionYangGuiDesc()],
  ['阴贵神',lunar.value.getPositionYinGuiDesc()],['福神',lunar.value.getPositionFuDesc()],['财神',lunar.value.getPositionCaiDesc()],
])
function move(offset:number){const next=addMonths(visible.value,offset);if(next.getFullYear()>=1900&&next.getFullYear()<=2100)visible.value=next}
function choose(date:Date){selected.value=date;visible.value=startOfMonth(date)}
function today(){choose(new Date(now.value))}
</script>

<template>
  <div class="grid min-w-0 items-start gap-4 min-[900px]:grid-cols-[minmax(0,1fr)_17rem]">
    <Card class="min-w-0 gap-5 p-4 shadow-none sm:p-5">
      <div class="flex flex-wrap items-center gap-2">
        <Select v-model="year"><SelectTrigger class="w-28" aria-label="选择年份"><CalendarDays /><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="y in years" :key="y" :value="String(y)">{{ y }} 年</SelectItem></SelectGroup></SelectContent></Select>
        <Button variant="secondary" size="icon-sm" aria-label="上个月" :disabled="year==='1900'&&month==='1'" @click="move(-1)"><ChevronLeft /></Button>
        <Select v-model="month"><SelectTrigger class="w-24" aria-label="选择月份"><SelectValue /></SelectTrigger><SelectContent><SelectGroup><SelectItem v-for="m in 12" :key="m" :value="String(m)">{{ m }} 月</SelectItem></SelectGroup></SelectContent></Select>
        <Button variant="secondary" size="icon-sm" aria-label="下个月" :disabled="year==='2100'&&month==='12'" @click="move(1)"><ChevronRight /></Button>
        <Button class="ml-auto" variant="outline" size="sm" @click="today">今天</Button>
      </div>
      <div class="relative">
        <span class="pointer-events-none absolute inset-0 flex items-center justify-center text-[16rem] leading-none font-bold text-muted/60" aria-hidden="true">{{ month }}</span>
        <div class="relative grid grid-cols-7 gap-1" role="group" aria-label="月历">
          <span v-for="(weekday,index) in ['日','一','二','三','四','五','六']" :key="weekday" :class="cn('py-3 text-center text-sm font-medium', (index===0||index===6)&&'text-destructive')">{{ weekday }}</span>
          <Button v-for="cell in cells" :key="cell.key" :variant="cell.key===selectedKey?'default':'ghost'" :aria-label="`${cell.key} ${cell.label}${cell.holiday?cell.holiday.isWork()?' 调休上班':' 休假':''}`" :aria-pressed="cell.key===selectedKey" :disabled="cell.date.getFullYear()<1900||cell.date.getFullYear()>2100" :class="cn('relative h-20 min-w-0 flex-col gap-1 px-1 tabular-nums',cell.key!==selectedKey&&cell.weekend&&'text-destructive',cell.key!==selectedKey&&cell.holiday&&(cell.holiday.isWork()?'bg-muted/70':'bg-destructive/5'),!cell.current&&'opacity-40',cell.key===todayKey&&cell.key!==selectedKey&&'ring-1 ring-inset ring-primary')" @click="choose(cell.date)">
            <span class="text-lg font-semibold">{{ String(cell.day).padStart(2,'0') }}</span>
            <span :class="cn('max-w-full truncate text-xs',cell.key!==selectedKey&&(cell.special?'text-primary':'text-muted-foreground'))">{{ cell.label }}</span>
            <span v-if="cell.holiday" :class="cn('absolute top-1 right-1 rounded px-0.5 text-[10px]',cell.key===selectedKey?'bg-primary-foreground/20 text-primary-foreground':cell.holiday.isWork()?'bg-muted text-muted-foreground':'bg-destructive text-white')">{{ cell.holiday.isWork()?'班':'休' }}</span>
          </Button>
        </div>
      </div>
      <p class="text-xs text-muted-foreground">休 / 班：内置中国大陆节假日数据；未收录年份不标注调休。</p>
    </Card>
    <Card class="gap-4 p-5 shadow-none" aria-label="所选日期详情" aria-live="polite">
      <div class="flex flex-col items-center gap-2 text-center">
        <p class="text-sm">{{ selectedKey }} 周{{ solar.getWeekInChinese() }}</p>
        <div class="relative flex size-24 items-center justify-center rounded-2xl bg-primary pt-2 text-5xl font-medium text-primary-foreground"><span class="absolute top-3 left-4 size-1.5 rounded-full bg-primary-foreground" /><span class="absolute top-3 right-4 size-1.5 rounded-full bg-primary-foreground" />{{ selected.getDate() }}</div>
        <p class="text-sm">{{ lunar.getYearInChinese() }}年{{ lunar.getMonthInChinese() }}月{{ lunar.getDayInChinese() }}</p>
        <p class="text-sm text-muted-foreground">{{ lunar.getYearInGanZhi() }}（{{ lunar.getYearShengXiao() }}）年 · {{ lunar.getDayInGanZhi() }}日</p>
        <p class="text-xs text-muted-foreground">本年第 {{ getISOWeek(selected) }} 周 · 第 {{ getDayOfYear(selected) }} 天</p>
        <div v-if="festivals.length||holiday" class="flex flex-wrap justify-center gap-1"><Badge v-for="festival in festivals" :key="festival" variant="secondary">{{ festival }}</Badge><Badge v-if="holiday" variant="outline">{{ holiday.getName() }} · {{ holiday.isWork()?'调休上班':'休假' }}</Badge></div>
      </div>
      <Separator />
      <div class="flex items-center gap-3 text-sm"><Badge variant="secondary">生肖</Badge>{{ lunar.getYearShengXiao() }}<Badge variant="secondary" class="ml-auto">星座</Badge>{{ solar.getXingZuo() }}座</div>
      <Separator />
      <div class="flex items-start gap-3"><Badge class="shrink-0" variant="secondary">宜</Badge><p class="text-sm leading-6">{{ lunar.getDayYi().join('、') }}</p></div>
      <div class="flex items-start gap-3"><Badge class="shrink-0" variant="destructive">忌</Badge><p class="text-sm leading-6">{{ lunar.getDayJi().join('、') }}</p></div>
      <Separator />
      <div class="flex items-center gap-3 text-sm"><Badge variant="secondary">月相</Badge>{{ lunar.getYueXiang() }}</div>
      <div class="flex items-start gap-3 text-sm"><Badge variant="secondary" class="shrink-0">物候</Badge>{{ lunar.getWuHou() }}</div>
      <div class="grid grid-cols-2 gap-2 text-xs"><p v-for="[name,direction] in positions" :key="name"><span class="text-muted-foreground">{{ name }}方位：</span>{{ direction }}</p></div>
      <p class="text-xs text-muted-foreground">黄历宜忌与方位为传统民俗信息，仅供文化参考。</p>
    </Card>
  </div>
</template>
