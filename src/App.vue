<script setup lang="ts">
import { onMounted } from "vue";
import { RouterView } from "vue-router";
import { invoke } from "./invoke";
import DialogDisplay from "./components/DialogDisplay.vue";
import { useDialogStore } from "./stores/dialog";

const dialogStore = useDialogStore();
onMounted(async () => {
  while (true) {
    const notification = await invoke("poll_notification");
    switch (notification.type) {
      case "serverExit":
        dialogStore.alert(
          "サーバーが終了しました",
          `サーバーが終了コード${notification.code}で終了しました。ログを確認してください。`,
        );
        break;
    }
  }
});
</script>
<template>
  <DialogDisplay />
  <main un-p="4" un-min-h="screen" un-z="50" un-flex="~ col">
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
