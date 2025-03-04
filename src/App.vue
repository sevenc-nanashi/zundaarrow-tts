<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterView } from "vue-router";
import { invoke } from "./invoke";
import DialogDisplay from "./components/DialogDisplay.vue";
import { useDialogStore } from "./stores/dialog";

const zundamonImagesRaw = import.meta.glob<string>("./assets/zundamon/*.webp", {
  eager: true,
  query: "?url",
  import: "default",
});
const zundamonImages = Object.fromEntries(
  Object.entries(zundamonImagesRaw).map(([key, value]) => [
    key.match(/\.\/assets\/zundamon\/(.+)\.webp/)![1],
    value,
  ]),
);

const dialogStore = useDialogStore();
const currentZundamonImage = ref<"sleeping" | null>("sleeping");

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
  <main un-p="4" un-min-h="screen" un-relative un-z="50" un-flex="~ col">
    <RouterView v-slot="{ Component }">
      <Transition name="fade">
        <Component
          :is="Component"
          v-model:currentZundamonImage="currentZundamonImage"
        />
      </Transition>
    </RouterView>
  </main>
  <img
    v-if="currentZundamonImage"
    :src="zundamonImages[currentZundamonImage]"
    un-height="50vh"
    un-absolute
    un-fixed
    un-bottom="0"
    un-right="0"
    un-pointer-events="none"
    un-z="0"
    un-op="30%"
  />
</template>
