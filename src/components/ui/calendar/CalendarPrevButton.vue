<script lang="ts" setup>
import type { CalendarPrevProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { ChevronLeft } from "@lucide/vue"
import { reactiveOmit } from "@vueuse/core"
import { CalendarPrev, useForwardProps } from "reka-ui"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"
import { buttonVariants } from '@/components/ui/button'

const props = defineProps<CalendarPrevProps & { class?: HTMLAttributes["class"] }>()

const delegatedProps = reactiveOmit(props, "class")

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TooltipProvider :delay-duration="300"><Tooltip><TooltipTrigger as-child>
  <CalendarPrev
    data-slot="calendar-prev-button"
    :class="cn(
      buttonVariants({ variant: 'outline' }),
      'pointer-events-auto size-7 bg-transparent p-0 opacity-50 hover:opacity-100',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <slot>
      <ChevronLeft class="size-4" />
    </slot>
  </CalendarPrev>
  </TooltipTrigger><TooltipContent>上个月</TooltipContent></Tooltip></TooltipProvider>
</template>
