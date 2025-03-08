<script setup lang="ts">
import { ref } from "vue";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import {
  ElNotification,
  ElButton,
  ElSelect,
  ElOption,
  ElTooltip,
} from "element-plus";
import { useServerStore } from "../stores/server.ts";
import { useDialogStore } from "../stores/dialog.ts";
import { faqUrl } from "../consts.ts";
import { invoke } from "../invoke.ts";
import { useRouter } from "vue-router";

const serverStore = useServerStore();
const dialogStore = useDialogStore();
const text = ref("");

const router = useRouter();

const languages = [
  { label: "自動", value: "auto" },
  { label: "日本語", value: "ja" },
  { label: "英語", value: "en" },
  { label: "中国語", value: "zh" },
  { label: "広東語", value: "yue" },
  { label: "韓国語", value: "ko" },
  { label: "日本語+英語", value: "ja+en" },
  { label: "中国語+英語", value: "zh+en" },
  { label: "広東語+英語", value: "yue+en" },
  { label: "韓国語+英語", value: "ko+en" },
];

const language = ref<string>("auto");

const state = ref<"idle" | "synthesizing" | "done" | "error">("idle");
const error = ref<string | null>(null);
const audioUrl = ref<string | null>(null);

const synthesize = async () => {
  state.value = "synthesizing";
  try {
    const wav = await serverStore.upfetch("/tts", {
      method: "POST",
      body: {
        text: text.value,
        target_language: language.value,
      },
      parseResponse: (response) => response.blob(),
    });

    const url = URL.createObjectURL(wav);
    audioUrl.value = url;
    state.value = "done";
  } catch (e) {
    error.value = String(e);
    state.value = "error";
  }
};

const saveAudio = async () => {
  if (!audioUrl.value) return;
  try {
    const path = await saveDialog({
      filters: [
        {
          name: "My Filter",
          extensions: ["png", "jpeg"],
        },
      ],
    });

    if (path) {
      const response = await fetch(audioUrl.value);
      const blob = await response.blob();
      const buffer = await blob.arrayBuffer();
      await writeFile(path, new Uint8Array(buffer));

      ElNotification({
        title: "保存しました",
        message: `音声を ${path} に保存しました`,
        position: "bottom-right",
      });
    }
  } catch (e) {
    dialogStore.alert("保存に失敗しました", String(e));
  }
};

const onEnter = (e: KeyboardEvent) => {
  if (e.key === "Enter" && e.ctrlKey) {
    synthesize();
  }
};

const openFolder = async () => {
  await invoke("open_folder");
};
</script>
<template>
  <nav un-absolute un-top="2" un-right="2" un-flex="~ col" un-gap="2">
    <ElTooltip content="フォルダを開く" placement="left">
      <ElButton circle @click="openFolder">
        <div
          un-i-material-symbols-folder
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
    <ElTooltip content="エンジンを再起動" placement="left">
      <ElButton circle un-ml="!0" @click="router.push('/launch')">
        <div
          un-i-material-symbols-refresh
          un-inline-block
          un-w="4"
          un-h="4"
          un-text="sm green-600"
        />
      </ElButton>
    </ElTooltip>
  </nav>
  <main un-w="100%" un-flex="~ col" un-items="center" un-gap="4">
    <textarea
      v-model="text"
      placeholder="ここに文章を入力するのだ！"
      rows="10"
      un-w="50%"
      un-min-w="100"
      un-resize="none"
      un-p="2"
      un-rounded="2"
      un-shadow="md"
      un-border="1 green-600"
      @keydown.enter="onEnter"
    />
    <div un-flex un-gap="1">
      <ElSelect v-model="language" un-w="!32" size="default">
        <ElOption
          v-for="lang in languages"
          :key="lang.value"
          :value="lang.value"
          :label="lang.label"
        />
      </ElSelect>
      <span />
      <ElButton
        type="success"
        un-text="white"
        un-flex="~"
        un-items="center"
        :disabled="!text || state === 'synthesizing'"
        @click.prevent="synthesize"
      >
        <div
          un-i-material-symbols-volume-up
          un-inline-block
          un-w="4"
          un-h="4"
          un-relative
          un-top="[1px]"
          un-m="r-1"
        />
        合成
      </ElButton>
      <span />
      <ElButton
        color="#fda5d5"
        un-flex="~"
        un-items="center"
        un-text="!white"
        :disabled="!audioUrl || state === 'synthesizing'"
        @click.prevent="saveAudio"
      >
        <div
          un-i-material-symbols-save
          un-inline-block
          un-w="4"
          un-h="4"
          un-relative
          un-top="[1px]"
          un-m="r-1"
        />
        保存
      </ElButton>
    </div>

    <div un-text="sm slate-500">
      <div v-if="state === 'synthesizing'">合成中...</div>
      <audio v-if="state === 'done' && audioUrl" :src="audioUrl" controls />
      <div v-if="state === 'error'" un-text="red-600">{{ error }}</div>
    </div>
  </main>
</template>
