<script setup lang="ts">
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
} from "radix-vue";

const open = defineModel<boolean>("open");
</script>

<template>
  <AlertDialogRoot v-model:open="open">
    <AlertDialogPortal>
      <AlertDialogOverlay un-bg="black/50" un-z="500" />
      <AlertDialogContent>
        <AlertDialogTitle />
        <AlertDialogDescription />
        <AlertDialogCancel />
        <AlertDialogAction />
      </AlertDialogContent>
    </AlertDialogPortal>
  </AlertDialogRoot>
</template>
