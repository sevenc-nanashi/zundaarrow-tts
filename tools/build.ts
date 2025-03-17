import { parse } from "@std/toml";
import { $, cd } from "zx";
import fs from "node:fs/promises";
import fsSync from "node:fs";
import path from "node:path";
import deepmerge from "deepmerge";
import { setOutput } from "@actions/core";
import createXxhash from "xxhash-wasm";
import lzmajs from "lzma-native";
import { pipeline } from "node:stream/promises";
import { Semaphore } from "@core/asyncutil";
import cliProgress from "cli-progress";

if (process.argv.length < 4) {
  console.error("Usage: build.ts <version> <device> [...options]");
  process.exit(1);
}

type HashInfo = {
  position: number;
  compressedSize: number;
  decompressedSize: number;
};

const xxhash = await createXxhash();

async function main() {
  const dirname = import.meta.dirname.replaceAll("\\", "/");
  const version = process.argv[2];
  const device = process.argv[3];
  const skipTauri = process.argv.includes("--skipTauri");
  $.verbose = true;

  cd(`${dirname}/../`);
  const platform =
    process.platform === "win32"
      ? "windows"
      : process.platform === "darwin"
        ? "macos"
        : "linux";

  const baseName = `zundaarrow_tts-${platform}-${version}-${device}`;
  const internalName = `__internal_${baseName}`;
  const destRoot = `${dirname}/../dist-packed`;
  const archivePath = `${destRoot}/${internalName}.bin`;
  const metaPath = `${destRoot}/${internalName}.json`;
  const filesRoot = `${dirname}/../target/release`;

  if (!skipTauri) {
    console.log("Building Tauri");
    await buildTauri(version, device);
  }
  console.log("Compressing files");
  const { hashInfo, fileToHash } = await compressFiles(destRoot, filesRoot);
  console.log("Creating archive");
  const archivePaths = await createArchive(
    archivePath,
    `${destRoot}/repository`,
    hashInfo,
  );
  console.log("Writing meta");
  await writeMeta(
    baseName,
    destRoot,
    metaPath,
    archivePaths,
    version,
    device,
    fileToHash,
    hashInfo,
  );

  if (!skipTauri) {
    console.log("Cleaning up");
    await fs.rm(`${filesRoot}/zundamon-speech`, { recursive: true });
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

async function buildTauri(version: string, device: string) {
  await $({
    env: {
      ...process.env,
      ZTS_VERSION: version,
      ZTS_DEVICE: device,
    },
  })`pnpm run tauri build --config ./src-tauri/tauri.conf.prod.json`;
}

async function compressFiles(destRoot: string, filesRoot: string) {
  const base = await fs.readFile("./src-tauri/Tauri.toml", "utf-8").then(parse);
  const patched = await fs
    .readFile("./src-tauri/tauri.conf.prod.json", "utf-8")
    .then(JSON.parse);

  const merged = deepmerge(base, patched) as any;

  const roots = [
    process.platform === "win32" ? "zundaarrow_tts.exe" : "zundaarrow_tts",
    ...Object.values(merged.bundle.resources),
  ] as string[];

  const repositoryPath = `${destRoot}/repository/`;
  await fs.mkdir(repositoryPath, { recursive: true });

  const getHash = async (path: string) => {
    const file = fsSync.createReadStream(path);
    const hash = xxhash.create64();
    for await (const chunk of file) {
      hash.update(chunk);
    }
    const hashString = hash.digest().toString(16).padStart(16, "0");

    return hashString;
  };

  const fileToHash: Map<string, string> = new Map();
  const hashInfo: Map<string, HashInfo> = new Map();
  const hashes = new Set<string>();
  const throttle = 10;
  const semaphore = new Semaphore(throttle);

  const shouldIgnore = (path: string) =>
    [".h", ".hpp", ".c", ".cpp"].some((ext) => path.endsWith(ext)) ||
    path.includes("__pycache__");

  const pack = (path: string, relativePath: string) =>
    semaphore.lock(async () => {
      const hashString = await getHash(path);
      fileToHash.set(relativePath, hashString);
      if (hashes.has(hashString)) {
        return;
      }
      hashes.add(hashString);
      const finalPath = `${repositoryPath}/${hashString}`;
      const finalPathStat = await fs.stat(finalPath).catch(() => null);
      if (finalPathStat) {
        const info = await fs.stat(path);
        hashInfo.set(hashString, {
          position: -1,
          compressedSize: finalPathStat.size,
          decompressedSize: info.size,
        });
        return;
      }

      console.log(`Compressing ${relativePath}`);
      const compressed = fsSync.createWriteStream(`${finalPath}.tmp`);
      const compressor = lzmajs.createCompressor({
        preset: 9,
      });

      const file = fsSync.createReadStream(path);
      await pipeline(file, compressor, compressed, {
        end: true,
      });

      hashInfo.set(hashString, {
        position: -1,
        compressedSize: compressed.bytesWritten,
        decompressedSize: file.bytesRead,
      });
      await fs.rename(`${finalPath}.tmp`, finalPath);
    });
  const promises: Promise<void>[] = [];

  const filePaths: string[] = [];
  for (const root of roots) {
    const stat = await fs.stat(`${filesRoot}/${root}`).catch(() => null);
    if (!stat) {
      console.error(`Failed to stat ${root}`);
      process.exit(1);
    }
    if (stat.isFile()) {
      filePaths.push(`${filesRoot}/${root}`);
    } else {
      console.log(`Traversing ${root}`);
      for await (const file of fs.glob(`${filesRoot}/${root}/**/*`, {
        withFileTypes: true,
      })) {
        if (!file.isFile()) {
          continue;
        }
        const filePath = `${file.parentPath}/${file.name}`;
        const relativePath = path
          .relative(filesRoot, filePath)
          .replaceAll("\\", "/");
        if (shouldIgnore(relativePath)) {
          continue;
        }
        filePaths.push(filePath);
      }
    }
  }

  const progress = new cliProgress.SingleBar(
    {},
    cliProgress.Presets.shades_classic,
  );
  console.log(`Compressing ${filePaths.length} files`);
  progress.start(filePaths.length, 0);
  for (const filePath of filePaths) {
    const relativePath = path
      .relative(filesRoot, filePath)
      .replaceAll("\\", "/");
    promises.push(
      pack(filePath, relativePath).then(() => progress.increment()),
    );
  }

  await Promise.all(promises);
  progress.stop();

  return {
    hashInfo,
    hashes,
    fileToHash,
  };
}

async function createArchive(
  baseArchivePath: string,
  repositoryPath: string,
  hashInfo: Map<string, HashInfo>,
) {
  const archives = [baseArchivePath + ".001"];
  const maxArchiveSize = 1024 * 1024 * 1024 * 2 - 1;
  let archive = fsSync.createWriteStream(baseArchivePath + ".001");
  const bar = new cliProgress.SingleBar({}, cliProgress.Presets.shades_classic);
  bar.start(hashInfo.size, 0);
  let bytesWritten = 0;
  let allPosition = 0;

  const appendFile = async (
    archive: fsSync.WriteStream,
    path: string,
    info: HashInfo,
  ) => {
    info.position = allPosition;
    const file = fsSync.createReadStream(path);
    for await (const chunk of file) {
      archive.write(chunk);
      bytesWritten += chunk.length;
      allPosition += chunk.length;
    }
    bar.increment();
  };
  const sortedHashInfo = [...hashInfo.entries()].toSorted(
    ([_a, a], [_b, b]) => b.compressedSize - a.compressedSize,
  );
  while (sortedHashInfo.length > 0) {
    const [hash, info] = sortedHashInfo[0];
    const path = `${repositoryPath}/${hash}`;

    if (bytesWritten + info.compressedSize > maxArchiveSize) {
      const maximumFileIndex = sortedHashInfo.findIndex(
        ([_hash, info]) => bytesWritten + info.compressedSize <= maxArchiveSize,
      );
      if (maximumFileIndex !== -1) {
        const [maximumFileHash, maximumFileInfo] =
          sortedHashInfo[maximumFileIndex];
        await appendFile(
          archive,
          `${repositoryPath}/${maximumFileHash}`,
          maximumFileInfo,
        );
        sortedHashInfo.splice(maximumFileIndex, 1);
      } else {
        archive.end();
        const newFileName = `${baseArchivePath}.${(archives.length + 1).toString().padStart(3, "0")}`;
        archive = fsSync.createWriteStream(newFileName);

        archives.push(newFileName);
        bytesWritten = 0;
      }
    } else {
      await appendFile(archive, path, info);
      sortedHashInfo.shift();
    }
  }
  bar.stop();
  archive.end();

  return archives;
}

async function writeMeta(
  baseName: string,
  destRoot: string,
  metaPath: string,
  archivePaths: string[],
  version: string,
  device: string,
  fileToHash: Map<string, string>,
  hashInfo: Map<string, HashInfo>,
) {
  const meta = {
    version,
    device,
    hashes: Object.fromEntries(fileToHash),
    hashInfo: Object.fromEntries(hashInfo),
  };

  await fs.writeFile(metaPath, JSON.stringify(meta));

  await $({
    cwd: `${import.meta.dirname}/../installer`,
    env: {
      ...process.env,
      ZTS_VERSION: version,
      ZTS_DEVICE: device,
    },
  })`cargo build --release`;

  const suffix = process.platform === "win32" ? ".exe" : "";
  const installerPath = `${destRoot}/${baseName}-installer${suffix}`;
  await fs.copyFile(
    `${import.meta.dirname}/../target/release/installer${suffix}`,
    installerPath,
  );

  const dummyFileName = `_internal______________________________________`;
  const dummyFilePath = `${destRoot}/${dummyFileName}`;
  await fs.writeFile(dummyFilePath, " ");

  setOutput(
    "assets",
    [...archivePaths, installerPath, metaPath, dummyFilePath].join("\n"),
  );
}
