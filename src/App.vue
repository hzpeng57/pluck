<script setup lang="ts">
import { ref } from "vue";

const message = ref("git-lite");
</script>

<template>
  <main class="flex h-full items-center justify-center bg-neutral-50 dark:bg-neutral-900">
    <div class="text-center">
      <h1 class="text-3xl font-semibold text-neutral-900 dark:text-neutral-50">
        {{ message }}
      </h1>
      <p class="mt-2 text-sm text-neutral-500">
        Tauri 2 + Vue 3 + Pinia + Tailwind workspace ready.
      </p>
    </div>
  </main>
</template>
