<script setup lang="ts">
import { ref } from "vue";
import SelectInput from "../components/SelectInput.vue";
import { useServerStore } from "../stores/server";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useDialogStore } from "../stores/dialog";

const serverStore = useServerStore();
const dialogStore = useDialogStore();
const text = ref("");

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

      dialogStore.alert("保存しました", `音声を ${path} に保存しました`);
    }
  } catch (e) {
    dialogStore.alert("保存に失敗しました", String(e));
  }
};
</script>
<template>
  <form
    un-w="100%"
    un-flex="~ col"
    un-items="center"
    un-gap="4"
    @submit.prevent="synthesize"
  >
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
    />
    <div un-flex un-gap="2">
      <SelectInput :items="languages" v-model="language" un-p="x-4 y-2" />
      <button
        type="submit"
        un-bg="!green-600 hover:!green-700 disabled:!green-600/50"
        un-text="white"
        un-p="x-4 y-2"
        un-rounded="2"
        un-flex="~"
        un-cursor="pointer disabled:not-allowed"
        un-items="center"
        :disabled="!text || state === 'synthesizing'"
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
      </button>
      <button
        un-bg="pink-400 hover:pink-500 disabled:!pink-400/50"
        un-text="white"
        un-p="x-4 y-2"
        un-rounded="2"
        un-flex="~"
        un-cursor="pointer disabled:not-allowed"
        un-items="center"
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
      </button>
    </div>

    <div un-text="sm slate-500">
      <div v-if="state === 'synthesizing'">合成中...</div>
      <audio v-if="state === 'done' && audioUrl" :src="audioUrl" controls />
      <div v-if="state === 'error'" un-text="red-600">{{ error }}</div>
    </div>
  </form>
</template>
