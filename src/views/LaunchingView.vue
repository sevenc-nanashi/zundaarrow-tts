<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";

const openFolder = async () => {
  await invoke("openFolder");
};
</script>
<template>
  <div
    un-flex="~ col"
    un-justify="center"
    un-items="center"
    un-gap="2"
    un-min-h="screen"
  >
    <p un-text="lg">起動中...</p>
    <a
      @click="openFolder"
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
      フォルダを開く</a
    >
    <a
      target="_blank"
      href="https://example.com"
      un-items="center"
      un-flex
      un-text="sm green-600"
      un-border="b-1 hover:green-600 transparent"
      un-cursor="pointer"
      ><div
        un-i-material-symbols-link
        un-inline-block
        un-w="4"
        un-h="4"
        un-m="r-1"
      />
      FAQ</a
    >
  </div>
  <p
    un-absolute
    un-left="0"
    un-right="0"
    un-bottom="2"
    un-text="sm center"
    un-op="50%"
  >
    ZundaArrow TTS - Developed by
    <a
      target="_blank"
      href="https://sevenc7c.com"
      un-text="#48b0d5"
      un-underline="hover:~"
      >Nanashi.</a
    >, Based on Zundamon Speech by
    <a
      target="_blank"
      href="https://zunko.jp"
      un-text="green-600"
      un-underline="hover:~"
      >Tohoku Zunko / Zundamon Project</a
    >.
  </p>
</template>
