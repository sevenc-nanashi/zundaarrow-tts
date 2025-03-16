<script setup lang="ts">
import { useRouter } from "vue-router";
import { faqUrl, termsUrl } from "../consts.ts";
import { ElButton, ElTooltip } from "element-plus";
import { invoke } from "../invoke.ts";
import { useDialogStore } from "../stores/dialog.ts";

const router = useRouter();
const dialog = useDialogStore();

const openAppFolder = async () => {
  await invoke("open_app_folder");
};

const openLogFolder = async () => {
  await invoke("open_log_folder");
};

const showAppInfo = async () => {
  const appInfo = await invoke("app_info");
  const versionInfo = [
    `バージョン：${appInfo.version}`,
    `ハードウェア：${appInfo.device}`,
    `ビルド時刻：${appInfo.buildTimestamp}`,
    `Git SHA：${appInfo.commitSha}`,
    `rustc：${appInfo.rustcVersion}`,
  ].join("\n");
  dialog.alert("このアプリについて", versionInfo);
};
</script>

<template>
  <nav un-absolute un-top="2" un-right="2" un-flex="~ col" un-gap="2">
    <ElTooltip content="利用規約" placement="left">
      <ElButton circle target="_blank" :href="termsUrl" tag="a" un-ml="!0">
        <div
          un-i-material-symbols-balance
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
    <ElTooltip content="FAQ" placement="left">
      <ElButton circle target="_blank" :href="faqUrl" tag="a" un-ml="!0">
        <div
          un-i-material-symbols-help
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
    <ElTooltip content="アプリの情報" placement="left">
      <ElButton circle un-ml="!0" @click="showAppInfo">
        <div
          un-i-material-symbols-info
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
    <hr />
    <ElTooltip content="アプリのフォルダを開く" placement="left">
      <ElButton circle @click="openAppFolder">
        <div
          un-i-material-symbols-folder
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
    <ElTooltip content="ログのフォルダを開く" placement="left">
      <ElButton circle @click="openLogFolder" un-ml="!0">
        <div
          un-i-material-symbols-monitor-heart
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
    <ElTooltip content="エンジンを再起動" placement="left">
      <ElButton circle un-ml="!0" @click="router.push('/launch')">
        <div
          un-i-material-symbols-refresh
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm pink-400"
        />
      </ElButton>
    </ElTooltip>
  </nav>
</template>
