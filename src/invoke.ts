import { invoke as rawInvoke } from "@tauri-apps/api/core";

export type Ipc = {
  launch: () => Promise<number>;
  open_folder: () => Promise<void>;
  poll_notification: () => Promise<Notification>;
};
export const invoke = <K extends keyof Ipc>(
  key: K,
  ...args: Parameters<Ipc[K]>
): ReturnType<Ipc[K]> => rawInvoke(key, ...args) as ReturnType<Ipc[K]>;

type Notification = {
  type: "serverExit";
  code: number;
};
