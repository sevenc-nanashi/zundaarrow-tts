<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";

const open = defineModel<boolean>("open");

const props = defineProps<{
  closeOnOverlayClick?: boolean;
}>();
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay
        un-bg="black/10"
        un-fixed
        un-top="0"
        un-left="0"
        un-w="screen"
        un-h="screen"
      />
      <DialogContent
        un-bg="white"
        un-fixed
        un-flex="~ col"
        un-top="[50%]"
        un-left="[50%]"
        un-translate-x="[-50%]"
        un-translate-y="[-50%]"
        un-p="4"
        un-rounded="2"
        un-shadow="md"
        un-z="50"
      >
        <DialogTitle un-text="lg">
          <slot name="title" />
        </DialogTitle>
        <DialogDescription>
          <slot name="description" />
        </DialogDescription>
        <div un-flex un-justify="end">
          <DialogClose
            un-mt="2"
            un-items="center"
            un-flex
            un-text="sm slate-500"
            un-border="1 slate-500"
            un-p="x-2 y-1"
            un-rounded="2"
            un-cursor="pointer"
          >
            <slot name="close">
              <div
                un-i-material-symbols-close
                un-inline-block
                un-w="4"
                un-h="4"
                un-m="r-1"
              />
              閉じる</slot
            >
          </DialogClose>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
