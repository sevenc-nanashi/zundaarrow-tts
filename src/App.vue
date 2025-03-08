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
      <Transition name="page" mode="out-in">
        <div un-flex-grow un-flex="~ col" :key="$route.fullPath">
          <Component
            :is="Component"
            v-model:currentZundamonImage="currentZundamonImage"
          />
        </div>
      </Transition>
    </RouterView>
    <p
      un-absolute
      un-left="0"
      un-right="0"
      un-bottom="2"
      un-text="xs center slate-500"
      un-drop-shadow="md"
      un-pointer-events="auto"
      un-z="10"
    >
      <a
        target="_blank"
        href="https://github.com/sevenc-nanashi/zundaarrow-tts"
        un-text="green-600"
        un-border="b-1 hover:green-600 transparent"
        >ZundaArrow TTS</a
      >
      - Developed by
      <a
        target="_blank"
        href="https://sevenc7c.com"
        un-text="#48b0d5"
        un-border="b-1 hover:green-600 transparent"
        >Nanashi.</a
      >, Based on Zundamon Speech by
      <a
        target="_blank"
        href="https://zunko.jp"
        un-text="green-600"
        un-border="b-1 hover:green-600 transparent"
        >Tohoku Zunko / Zundamon Project</a
      >.
    </p>
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

<style>
.page-enter-active,
.page-leave-active {
  transition:
    transform 0.3s,
    opacity 0.3s;
}

.page-enter-from,
.page-leave-to {
  transform: translateY(1rem);

  opacity: 0;

  @media (prefers-reduced-motion) {
    transform: none;
  }
}
</style>
