<script setup lang="ts">
import { useDialogStore } from "../stores/dialog";
import { ElButton, ElDialog } from "element-plus";

const dialogStore = useDialogStore();
</script>
<template>
  <ElDialog
    v-for="dialog in dialogStore.dialogs"
    :key="dialog.nonce"
    :modelValue="true"
    :title="dialog.title"
  >
    {{ dialog.message }}
    <template #footer>
      <div class="dialog-footer">
        <ElButton type="primary" @click="dialogStore.close(dialog.nonce)">
          閉じる
        </ElButton>
      </div>
    </template>
  </ElDialog>
</template>
