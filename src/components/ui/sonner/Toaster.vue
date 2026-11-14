<script setup lang="ts">
import { Toaster } from 'vue-sonner';
</script>

<template>
  <Teleport to="body">
    <Toaster
      class="lumo-sonner"
      position="top-center"
      :offset="16"
      :duration="8000"
      :toast-options="{
        style: {
          background: 'var(--toast-background)',
          color: 'var(--foreground)',
          borderColor: 'var(--border)',
        },
        actionButtonStyle: {
          background: 'var(--primary)',
          color: 'var(--primary-foreground)',
        },
      }"
    />
  </Teleport>
</template>
