<script setup lang="ts">
import { onMounted, ref } from "vue";
import windowSrc from "./assets/zundaarrow.webp?format=webp&imagetools";
import { ElButton } from "element-plus";

const releaseData = ref<
  | {
      version: string;
      cpu: string;
      cuda: string;
    }
  | undefined
>(undefined);

onMounted(() => {
  import("../node_modules/budoux/module/webcomponents/budoux-ja.js");

  fetch(
    "https://ungh.cc/repos/sevenc-nanashi/zundaarrow-tts/releases/latest",
  ).then(async (res) => {
    const data = await res.json();
    releaseData.value = {
      version: data.release.name,
      cpu: data.release.assets.find(
        (asset: any) =>
          asset.downloadUrl.includes("cpu") &&
          asset.downloadUrl.endsWith(".exe"),
      )?.downloadUrl,
      cuda: data.release.assets.find(
        (asset: any) =>
          asset.downloadUrl.includes("cuda") &&
          asset.downloadUrl.endsWith(".exe"),
      )?.browser_download_url,
    };
  });
});
</script>

<template>
  <div
    un-absolute
    un-top="1/2"
    un-right="4"
    un-translate-y="-1/2"
    un-z="0"
    un-opacity="50"
    un-filter="blur-2"
  >
    <img :src="windowSrc" un-h="50vh" />
  </div>
  <main
    un-w="[min(100vw,_640px)]"
    un-mx="auto"
    un-gap-4
    un-pl="4"
    un-gap-2
    un-flex="~ col"
    un-relative
    un-z="10"
  >
    <h1 un-text="5xl green-600">ZundaArrow TTS</h1>
    <p un-wrap="~">
      <budoux-ja>
        <span un-text="green-600">ZundaArrow TTS</span> は、<a
          href="https://www.youtube.com/watch?v=5e-1ymXBFL0"
          target="_blank"
          rel="noopener"
          un-text="green-600"
          un-underline="hover:~"
          >ずんだもんSpeech</a
        >
        を使いやすくした、Windows向けのテキスト読み上げソフトです。
      </budoux-ja>
    </p>

    <hr />

    <h2 un-text="2xl green-600" un-relative un-z="10">最新リリース</h2>
    <p>
      最新バージョン：<span un-text="green-600">{{
        releaseData?.version || "取得中..."
      }}</span>
    </p>

    <div>
      <ElButton
        :disabled="!releaseData"
        :href="releaseData?.cpu"
        target="_blank"
        rel="noopener"
        tag="a"
        >CPU版をダウンロード</ElButton
      >
      <ElButton
        :disabled="!releaseData"
        :href="releaseData?.cuda"
        target="_blank"
        rel="noopener"
        tag="a"
        >GPU版をダウンロード（CUDAが必要です）</ElButton
      >
    </div>

    <div un-h="4" />

    <p un-text="xs gray-600">
      Developed by
      <a
        href="https://sevenc7c.com"
        target="_blank"
        rel="noopener"
        un-text="[#48b0d5]"
        un-underline="hover:~"
        >Nanashi.</a
      >
      | GitHub：<a
        href="https://github.com/sevenc-nanashi/zundaarrow-tts"
        target="_blank"
        rel="noopener"
        un-text="green-600"
        un-underline="hover:~"
        >sevenc-nanashi/zundaarrow-tts</a
      >
      | Released under the MIT License.
    </p>
  </main>
</template>

<style scoped></style>
