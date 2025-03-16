import fs from "node:fs/promises";

if (process.argv.length !== 3) {
  console.error("Usage: node setVersion.js <version>");
  process.exit(1);
}

const version = process.argv[2];

const appVue = await fs.readFile("landing/src/App.vue", "utf-8");

await fs.writeFile(
  "landing/src/App.vue",
  appVue.replace(/const version = ".*";/, `const version = "${version}";`),
);

await fs.writeFile("landing/public/version.json", JSON.stringify({ version }));
