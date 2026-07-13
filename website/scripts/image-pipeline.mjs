import { mkdir, mkdtemp, readdir, rename, rm, stat } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import sharp from "sharp";

const names = ["workspace", "diff", "rebase"];
const widths = [1280, 1920];
const formats = ["avif", "webp", "png"];

async function inspectCapture(input) {
  let metadata;
  try {
    metadata = await sharp(input).metadata();
  } catch (error) {
    if (error.code === "ENOENT" || error.message.includes("Input file is missing")) {
      throw new Error(`${input} is missing`, { cause: error });
    }
    throw new Error(`${input} could not be decoded as a PNG`, { cause: error });
  }
  if (metadata.format !== "png") {
    throw new Error(`${input} must contain PNG data; detected ${metadata.format ?? "unknown"}`);
  }
  if (!Number.isInteger(metadata.width) || metadata.width < 1920 ||
      !Number.isInteger(metadata.height) || metadata.height <= 0) {
    throw new Error(`${input} must be a valid PNG screenshot at least 1920px wide`);
  }
  return input;
}

async function preflightCaptures(inputDir) {
  return Promise.all(names.map(name => inspectCapture(resolve(inputDir, `${name}.png`))));
}

function overlaySvg() {
  return Buffer.from(`
    <svg width="1200" height="630" xmlns="http://www.w3.org/2000/svg">
      <rect width="1200" height="630" fill="#0d0e10" fill-opacity="0.18"/>
      <rect width="540" height="630" fill="#0d0e10" fill-opacity="0.92"/>
      <text x="82" y="270" fill="#f1f0ea" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="72" font-weight="700">Pluck</text>
      <text x="82" y="330" fill="#78d7bd" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="30">Git with developer muscle memory.</text>
      <text x="82" y="382" fill="#bbb8ae" font-family="-apple-system, BlinkMacSystemFont, sans-serif" font-size="22">A focused Git client for macOS.</text>
    </svg>
  `);
}

async function generateStagedOutputs({ captures, publicDir, imageDir, ogDir }) {
  await Promise.all(captures.map(async (input, index) => {
    const name = names[index];
    for (const width of widths) {
      const base = sharp(input).rotate().resize({ width, withoutEnlargement: true });
      await Promise.all([
        base.clone().avif({ quality: 62 }).toFile(resolve(imageDir, `${name}-${width}.avif`)),
        base.clone().webp({ quality: 82 }).toFile(resolve(imageDir, `${name}-${width}.webp`)),
        base.clone().png({ compressionLevel: 9 }).toFile(resolve(imageDir, `${name}-${width}.png`)),
      ]);
    }
  }));

  const icon = await sharp(resolve(publicDir, "favicon.png")).resize(72, 72).png().toBuffer();
  await sharp(captures[0])
    .rotate()
    .resize(1200, 630, { fit: "cover", position: "centre" })
    .composite([
      { input: overlaySvg(), top: 0, left: 0 },
      { input: icon, top: 112, left: 82 },
    ])
    .png({ compressionLevel: 9 })
    .toFile(resolve(ogDir, "pluck-cover.png"));
}

async function validateStagedOutputs(imageDir, ogDir) {
  const expectedImages = names
    .flatMap(name => widths.flatMap(width => formats.map(format => `${name}-${width}.${format}`)))
    .sort();
  assertFileList(await readdir(imageDir), expectedImages, "responsive image staging");
  assertFileList(await readdir(ogDir), ["pluck-cover.png"], "OG image staging");
  await Promise.all([
    ...expectedImages.map(file => sharp(resolve(imageDir, file)).metadata()),
    sharp(resolve(ogDir, "pluck-cover.png")).metadata(),
  ]);
}

function assertFileList(actual, expected, label) {
  const sorted = [...actual].sort();
  if (sorted.length !== expected.length || sorted.some((file, index) => file !== expected[index])) {
    throw new Error(`${label} contains unexpected files: ${sorted.join(", ")}`);
  }
}

async function pathExists(path) {
  try {
    await stat(path);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function replacePublishedOutputs(publicDir, stagedImages, stagedOg, transactionId) {
  const publishedImages = resolve(publicDir, "images");
  const publishedOg = resolve(publicDir, "og");
  const backupImages = resolve(publicDir, `.product-media-backup-${transactionId}-images`);
  const backupOg = resolve(publicDir, `.product-media-backup-${transactionId}-og`);
  const hadImages = await pathExists(publishedImages);
  const hadOg = await pathExists(publishedOg);
  let movedImages = false;
  let movedOg = false;

  try {
    if (hadImages) await rename(publishedImages, backupImages);
    if (hadOg) await rename(publishedOg, backupOg);
    await rename(stagedImages, publishedImages);
    movedImages = true;
    await rename(stagedOg, publishedOg);
    movedOg = true;
  } catch (error) {
    if (movedImages) await rm(publishedImages, { recursive: true, force: true });
    if (movedOg) await rm(publishedOg, { recursive: true, force: true });
    if (hadImages && await pathExists(backupImages)) await rename(backupImages, publishedImages);
    if (hadOg && await pathExists(backupOg)) await rename(backupOg, publishedOg);
    throw error;
  }

  await Promise.all([
    rm(backupImages, { recursive: true, force: true }),
    rm(backupOg, { recursive: true, force: true }),
  ]);
}

export async function optimizeProductImages({ inputDir, publicDir, log = console.log }) {
  const resolvedInputDir = resolve(inputDir);
  const resolvedPublicDir = resolve(publicDir);
  const captures = await preflightCaptures(resolvedInputDir);

  await mkdir(resolvedPublicDir, { recursive: true });
  const stagingDir = await mkdtemp(resolve(resolvedPublicDir, ".product-media-staging-"));
  const transactionId = basename(stagingDir).replace(".product-media-staging-", "");
  const stagedImages = join(stagingDir, "images");
  const stagedOg = join(stagingDir, "og");

  try {
    await Promise.all([
      mkdir(stagedImages, { recursive: true }),
      mkdir(stagedOg, { recursive: true }),
    ]);
    await generateStagedOutputs({ captures, publicDir: resolvedPublicDir, imageDir: stagedImages, ogDir: stagedOg });
    await validateStagedOutputs(stagedImages, stagedOg);
    await replacePublishedOutputs(resolvedPublicDir, stagedImages, stagedOg, transactionId);

    for (const name of names) {
      for (const width of widths) {
        for (const format of formats) {
          const output = resolve(resolvedPublicDir, "images", `${name}-${width}.${format}`);
          const metadata = await sharp(output).metadata();
          const size = await stat(output);
          log(`${output}: ${metadata.width}x${metadata.height}, ${size.size} bytes`);
        }
      }
    }
    log(`${resolve(resolvedPublicDir, "og", "pluck-cover.png")}: 1200x630`);
  } finally {
    await rm(stagingDir, { recursive: true, force: true });
  }
}
