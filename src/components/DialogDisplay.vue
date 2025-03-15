<script setup lang="ts">
import { ref, watch } from "vue";
import { useDialogStore } from "../stores/dialog";
import { ElButton, ElDialog } from "element-plus";

const dialogStore = useDialogStore();

const dialogOpenStates = ref<Record<string, boolean>>({});
watch(() => dialogStore.dialogs, (dialogs) => {
  for (const dialog of dialogs) {
    if (!(dialog.nonce in dialogOpenStates.value)) {
      dialogOpenStates.value[dialog.nonce] = true;
    }
  }
});
</script>
<template>
  <ElDialog
    v-for="dialog in dialogStore.dialogs"
    :key="dialog.nonce"
    :modelValue="dialogOpenStates[dialog.nonce] ?? true"
    :title="dialog.title"
    @closed="dialogStore.close(dialog.nonce)"
    alignCenter
  >
    <p un-whitespace="pre-wrap">{{ dialog.message }}</p>
    <template #footer>
      <div class="dialog-footer">
        <ElButton
          type="success"
          @click="dialogOpenStates[dialog.nonce] = false"
        >
          閉じる
        </ElButton>
      </div>
    </template>
  </ElDialog>
</template>
