import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { optimizeProductImages } from "./image-pipeline.mjs";

const inputDir = resolve(process.argv[2] ?? "/tmp/pluck-website-captures");
const publicDir = fileURLToPath(new URL("../public/", import.meta.url));

await optimizeProductImages({ inputDir, publicDir });
