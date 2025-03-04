import { defineStore } from "pinia";
import { up } from "up-fetch";
import { invoke } from "../invoke";

export const useServerStore = defineStore("server", {
  state: () => ({
    _port: 0,
  }),

  actions: {
    async launch() {
      const port = await invoke("launch");

      this._port = port;
    },

    async wait(args: { timeout: number }) {
      for (let i = 0; i < args.timeout; i++) {
        await new Promise((resolve) => setTimeout(resolve, 1000));

        try {
          await this.upfetch("/");

          return;
        } catch (e) {
          // ignore
        }
      }
    },
  },
  getters: {
    port: (state) => {
      if (state._port === 0) {
        throw new Error("Port not set");
      }

      return state._port;
    },
    upfetch: (state) =>
      up(fetch, () => ({
        baseUrl: `http://localhost:${state._port}`,
      })),
  },
});
