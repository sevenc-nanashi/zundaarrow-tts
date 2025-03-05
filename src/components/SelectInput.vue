<script setup lang="ts">
import {
  SelectContent,
  SelectItem,
  SelectItemText,
  SelectItemIndicator,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  SelectIcon,
  SelectScrollUpButton,
  SelectScrollDownButton,
} from "reka-ui";
import { computed } from "vue";

const props = defineProps<{
  placeholder?: string;
  items: { label: string; value: string }[];
}>();

const value = defineModel<string>();

const currentLabel = computed(() => {
  const item = props.items.find((item) => item.value === value.value);
  return item?.label;
});
</script>

<template>
  <SelectRoot v-model="value">
    <SelectTrigger
      v-bind="$attrs"
      un-rounded="2"
      un-border="1 slate-500"
      un-flex
      un-items="center"
    >
      <slot>
        <SelectValue :placeholder="props.placeholder">
          {{ currentLabel }}
        </SelectValue>
        <SelectIcon>
          <div
            un-i-material-symbols-arrow-drop-down
            un-m="l-1"
            un-w="4"
            un-h="4"
            un-relative
            un-top="[1px]"
          />
        </SelectIcon>
      </slot>
    </SelectTrigger>

    <SelectPortal>
      <SelectContent
        :sideOffset="4"
        un-min-w="40"
        un-bg="white"
        un-rounded="2"
        un-shadow="md slate/50"
        un-relative
        un-p="2"
        un-z="50"
      >
        <SelectScrollUpButton />
        <SelectViewport>
          <SelectItem
            v-for="item in props.items"
            :key="item.value"
            :value="item.value"
            un-text="hover:green-600"
            un-cursor="pointer"
            un-flex
            un-items="center"
          >
            <slot name="item" v-bind="{ item }">
              <SelectItemText>{{ item.label }}</SelectItemText>
              <SelectItemIndicator>
                <div
                  v-if="item.value === value"
                  un-i-material-symbols-check
                  un-text="green-600"
                  un-m="r-1"
                  un-relative
                  un-top="[1px]"
                />
              </SelectItemIndicator>
            </slot>
          </SelectItem>
        </SelectViewport>
        <SelectScrollDownButton />
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
