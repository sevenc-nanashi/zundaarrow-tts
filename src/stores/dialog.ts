import { defineStore } from "pinia";

export type Dialog = {
  title: string;
  message: string;
};

type InternalDialog = Dialog & {
  nonce: number;
};

export const useDialogStore = defineStore("dialog", {
  state: () => ({
    dialogs: [] as InternalDialog[],
  }),

  actions: {
    alert(title: string, message: string) {
      this.dialogs.push({ title, message, nonce: Math.random() });
    },
    close(nonce: number) {
      this.dialogs = this.dialogs.filter((dialog) => dialog.nonce !== nonce);
    }
  },
});
