<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template>
  <main un-p="4" un-min-h="screen" un-z="50">
    <RouterView v-slot="{ Component }">
      <Transition name="fade">
        <Component :is="Component" />
      </Transition>
    </RouterView>
  </main>
  <div un-absolute un-bottom="0" un-right="0" un-pointer-events="none" un-z="0">
    TODO: ずんだもん立ち絵
  </div>
</template>
