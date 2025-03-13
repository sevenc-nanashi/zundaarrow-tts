import { parse } from "@std/toml";
import { $, cd } from "zx";
import fs from "node:fs/promises";
import path from "node:path";
import deepmerge from "deepmerge";
import { setOutput } from "@actions/core";

if (process.argv.length < 4) {
  console.error("Usage: build.ts <version> <device>");
  process.exit(1);
}

const dirname = import.meta.dirname.replaceAll("\\", "/");
const version = process.argv[2];
const device = process.argv[3];
$.verbose = true;
cd(`${dirname}/../`);

await $`pnpm run tauri build --conf ./src-tauri/tauri.conf.prod.json`;

const base = await fs.readFile("./src-tauri/Tauri.toml", "utf-8").then(parse);
const patched = await fs
  .readFile("./src-tauri/tauri.conf.prod.json", "utf-8")
  .then(JSON.parse);

const merged = deepmerge(base, patched) as any;

const files = [
  process.platform === "win32" ? "zundaarrow_tts.exe" : "zundaarrow_tts",
  ...Object.values(merged.bundle.resources),
];

const archivePath = `${dirname}/../zundaarrow_tts-${version}-${device}.7z`;
const metaPath = `${dirname}/../zundaarrow_tts-${version}-${device}.meta.json`;
await $({
  cwd: `${dirname}/../target/release`,
})`7z a -mx=9 -mhc=false -mfb=258 -mpass=15 -v1999m -r ${path.resolve(archivePath)} ${files}`;

const list = await $`7z l ${archivePath}.001`.text();
// 2025-03-09 07:30:34         6518015337   3876121952  60021 files, 6769 folders
const size = list.match(/([0-9]+) +[0-9]+ +[0-9]+ files, [0-9]+ folders/);
if (!size) {
  console.error("Failed to get archive size");
  process.exit(1);
}

const archiveSize = Number.parseInt(size[1]);
console.log(`Archive size: ${archiveSize} bytes`);

const meta = {
  version,
  device,
  archiveSize,
};

await fs.writeFile(metaPath, JSON.stringify(meta, null, 2));

const archivePaths = await fs
  .readdir(path.dirname(archivePath))
  .then((files) =>
    files
      .filter((file) => file.startsWith(path.basename(archivePath)))
      .map((file) => path.join(path.dirname(archivePath), file)),
  );

setOutput("archivePaths", archivePaths.join("\n"));
setOutput("metaPath", metaPath);
