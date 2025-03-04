<script setup lang="ts">
import { ref } from "vue";

const text = ref("");
</script>
<template>
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
    />
    <button
      un-bg="green-600 hover:green-700"
      un-text="white"
      un-p="x-4 y-2"
      un-rounded="2"
      un-flex="~"
      un-cursor="pointer"
      un-items="center"
    >
      <div
        un-i-material-symbols-volume-up
        un-inline-block
        un-w="4"
        un-h="4"
        un-m="r-1"
      />
      合成
    </button>
  </main>
</template>
