<script setup lang="ts">
import { Card } from '@/components/ui/card';
</script>
<template>
  <Card
    class="@container relative min-h-0 min-w-0 gap-0 overflow-hidden border-border/70 bg-card py-0 shadow-[0_1px_2px_rgba(15,23,42,0.04),0_8px_24px_rgba(15,23,42,0.035)] transition-shadow hover:shadow-[0_2px_5px_rgba(15,23,42,0.06),0_12px_28px_rgba(15,23,42,0.06)]"
    ><slot
  /></Card>
</template>
