import { mkdir, stat } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import sharp from "sharp";

const inputDir = resolve(process.argv[2] ?? "/tmp/pluck-website-captures");
const publicDir = fileURLToPath(new URL("../public/", import.meta.url));
const imageDir = resolve(publicDir, "images");
const ogDir = resolve(publicDir, "og");
const names = ["workspace", "diff", "rebase"];
const widths = [1280, 1920];

await mkdir(imageDir, { recursive: true });
await mkdir(ogDir, { recursive: true });

for (const name of names) {
  const input = resolve(inputDir, `${name}.png`);
  const metadata = await sharp(input).metadata();
  if (!metadata.width || metadata.width < 1920 || !metadata.height) {
    throw new Error(`${input} must be a valid screenshot at least 1920px wide`);
  }

  for (const width of widths) {
    const base = sharp(input).rotate().resize({ width, withoutEnlargement: true });
    await Promise.all([
      base.clone().avif({ quality: 62 }).toFile(resolve(imageDir, `${name}-${width}.avif`)),
      base.clone().webp({ quality: 82 }).toFile(resolve(imageDir, `${name}-${width}.webp`)),
      base.clone().png({ compressionLevel: 9 }).toFile(resolve(imageDir, `${name}-${width}.png`)),
    ]);
  }
}

const workspace = resolve(inputDir, "workspace.png");
const icon = await sharp(resolve(publicDir, "favicon.png")).resize(72, 72).png().toBuffer();
const overlay = Buffer.from(`
  <svg width="1200" height="630" xmlns="http://www.w3.org/2000/svg">
    <rect width="1200" height="630" fill="#0d0e10" fill-opacity="0.18"/>
    <rect width="540" height="630" fill="#0d0e10" fill-opacity="0.92"/>
    <text x="82" y="270" fill="#f1f0ea" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="72" font-weight="700">Pluck</text>
    <text x="82" y="330" fill="#78d7bd" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="30">Git with developer muscle memory.</text>
    <text x="82" y="382" fill="#bbb8ae" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="22">A focused Git client for macOS.</text>
  </svg>
`);

await sharp(workspace)
  .rotate()
  .resize(1200, 630, { fit: "cover", position: "centre" })
  .composite([
    { input: overlay, top: 0, left: 0 },
    { input: icon, top: 112, left: 82 },
  ])
  .png({ compressionLevel: 9 })
  .toFile(resolve(ogDir, "pluck-cover.png"));

for (const name of names) {
  for (const width of widths) {
    for (const format of ["avif", "webp", "png"]) {
      const output = resolve(imageDir, `${name}-${width}.${format}`);
      const metadata = await sharp(output).metadata();
      const size = await stat(output);
      console.log(`${output}: ${metadata.width}x${metadata.height}, ${size.size} bytes`);
    }
  }
}
console.log(`${resolve(ogDir, "pluck-cover.png")}: 1200x630`);
