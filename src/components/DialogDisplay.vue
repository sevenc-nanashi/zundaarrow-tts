<script setup lang="ts">
import { useDialogStore } from "../stores/dialog";
import AlertDialog from "./AlertDialog.vue";

const dialogStore = useDialogStore();
</script>
<template>
  <AlertDialog
    v-for="dialog in dialogStore.dialogs"
    :key="dialog.nonce"
    :open="true"
    @update:open="dialogStore.close(dialog.nonce)"
  >
    <template #title>{{ dialog.title }}</template>
    <template #description>{{ dialog.message }}</template>
  </AlertDialog>
</template>
