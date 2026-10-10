<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { buttonVariants } from '@/components/ui/button'

withDefaults(defineProps<{ title: string; description: string; confirmText?: string; destructive?: boolean }>(), {
  confirmText: '确认',
  destructive: false,
})
const emit = defineEmits<{ confirm: [] }>()
const open = defineModel<boolean>('open')
</script>

<template>
  <AlertDialog v-model:open="open">
    <AlertDialogTrigger v-if="$slots.trigger" as-child><slot name="trigger" /></AlertDialogTrigger>
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>{{ title }}</AlertDialogTitle>
        <AlertDialogDescription class="whitespace-pre-wrap">{{ description }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>取消</AlertDialogCancel>
        <AlertDialogAction :class="destructive ? buttonVariants({ variant: 'destructive' }) : undefined" @click="emit('confirm')">{{ confirmText }}</AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
