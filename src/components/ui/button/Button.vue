<script setup lang="ts">
import type { PrimitiveProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { useAttrs } from "vue"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import type { ButtonVariants } from "."
import { Primitive } from "reka-ui"
import { cn } from "@/lib/utils"
import { buttonVariants } from "."

interface Props extends PrimitiveProps {
  variant?: ButtonVariants["variant"]
  size?: ButtonVariants["size"]
  class?: HTMLAttributes["class"]
}

defineOptions({ inheritAttrs: false })
const attrs = useAttrs()
const tooltip = () => String(attrs["aria-label"] || attrs.title || "")
const showTooltip = () => Boolean(tooltip()) && (props.size?.startsWith("icon") || Boolean(attrs.title))
const buttonAttrs = () => { const { title, ...rest } = attrs; return rest }

const props = withDefaults(defineProps<Props>(), {
  as: "button",
})
</script>

<template>
  <TooltipProvider v-if="showTooltip()" :delay-duration="300"><Tooltip><TooltipTrigger as-child>
    <Primitive v-bind="buttonAttrs()" data-slot="button" :data-variant="variant" :data-size="size" :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)"><slot /></Primitive>
  </TooltipTrigger><TooltipContent>{{ tooltip() }}</TooltipContent></Tooltip></TooltipProvider>
  <Primitive v-else v-bind="attrs" data-slot="button" :data-variant="variant" :data-size="size" :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)"><slot /></Primitive>
</template>
