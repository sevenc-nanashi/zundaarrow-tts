import { invoke as rawInvoke } from "@tauri-apps/api/core";

export type AppInfo = {
  version: string;
  device: string;
  buildTimestamp: string;
  commitSha: string;
  rustcVersion: string;
};

export type Ipc = {
  launch: () => Promise<number>;
  app_info: () => Promise<AppInfo>;
  open_app_folder: () => Promise<void>;
  open_log_folder: () => Promise<void>;
  poll_notification: () => Promise<Notification>;
};
export const invoke = <K extends keyof Ipc>(
  key: K,
  ...args: Parameters<Ipc[K]>
): ReturnType<Ipc[K]> => {
  console.log(`invoke: ${key}(${args.join(", ")})`);
  return rawInvoke(key, ...args).then((result) => {
    console.log(`invoke: ${key}(${args.join(", ")}) -> %o`, result);
    return result;
  }) as ReturnType<Ipc[K]>;
};

type Notification = {
  type: "serverExit";
  code: number;
};
