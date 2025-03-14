<script setup lang="ts">
import { onMounted } from "vue";
import { useServerStore } from "../stores/server";
import { invoke } from "../invoke";
import { useDialogStore } from "../stores/dialog";
import { useRouter } from "vue-router";
import { faqUrl } from "../consts";

const serverStore = useServerStore();
const dialogStore = useDialogStore();

const router = useRouter();

const currentZundamonImage = defineModel("currentZundamonImage");

onMounted(async () => {
  currentZundamonImage.value = "sleeping";
  try {
    await serverStore.launch();
  } catch (e) {
    dialogStore.alert("サーバーの起動に失敗しました", String(e));
  }

  await serverStore.wait({ timeout: 300 });

  router.push("/main");
});

const openAppFolder = async () => {
  await invoke("open_app_folder");
};
const openLogFolder = async () => {
  await invoke("open_log_folder");
};
</script>
<template>
  <div
    un-flex-grow
    un-flex="~ col"
    un-justify="center"
    un-items="center"
    un-gap="2"
  >
    <p un-text="lg">起動中...</p>
    <a
      @click="openAppFolder"
      un-items="center"
      un-flex
      un-text="sm green-600"
      un-border="b-1 hover:green-600 transparent"
      un-cursor="pointer"
      ><div
        un-i-material-symbols-folder
        un-inline-block
        un-w="4"
        un-h="4"
        un-m="r-1"
      />
      アプリのフォルダを開く</a
    >
    <a
      @click="openLogFolder"
      un-items="center"
      un-flex
      un-text="sm green-600"
      un-border="b-1 hover:green-600 transparent"
      un-cursor="pointer"
      ><div
        un-i-material-symbols-monitor-heart
        un-inline-block
        un-w="4"
        un-h="4"
        un-m="r-1"
      />
      ログのフォルダを開く</a
    >
    <a
      target="_blank"
      :href="faqUrl"
      un-items="center"
      un-flex
      un-text="sm green-600"
      un-border="b-1 hover:green-600 transparent"
      un-cursor="pointer"
      ><div
        un-i-material-symbols-help
        un-inline-block
        un-w="4"
        un-h="4"
        un-m="r-1"
      />
      FAQ</a
    >
  </div>
</template>
