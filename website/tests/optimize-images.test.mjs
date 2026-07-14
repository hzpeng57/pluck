import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import sharp from "sharp";

const moduleUrl = new URL("../scripts/image-pipeline.mjs", import.meta.url);
const names = ["workspace", "diff", "rebase"];
const widths = [1280, 1920];
const formats = ["avif", "webp", "png"];
const embeddedMetadataFields = ["exif", "icc", "iptc", "xmp"];

function assertMetadataStripped(metadata, file) {
  assert.deepEqual(
    embeddedMetadataFields.filter(field => metadata[field] !== undefined),
    [],
    `${file} must not contain EXIF, ICC, IPTC, or XMP metadata`,
  );
}

async function loadOptimizer() {
  try {
    const module = await import(moduleUrl.href);
    assert.equal(typeof module.optimizeProductImages, "function");
    return module.optimizeProductImages;
  } catch (error) {
    assert.fail(`optimizer must expose an importable test entry: ${error.message}`);
  }
}

async function createFixture(t) {
  const root = await mkdtemp(join(tmpdir(), "pluck-image-pipeline-"));
  const inputDir = join(root, "captures");
  const publicDir = join(root, "public");
  await mkdir(inputDir, { recursive: true });
  await mkdir(join(publicDir, "images"), { recursive: true });
  await mkdir(join(publicDir, "og"), { recursive: true });
  await sharp({
    create: { width: 128, height: 128, channels: 4, background: "#78d7bd" },
  }).png().toFile(join(publicDir, "favicon.png"));
  await writeFile(join(publicDir, "images", "existing.txt"), "keep-images");
  await writeFile(join(publicDir, "og", "existing.txt"), "keep-og");
  t.after(() => rm(root, { recursive: true, force: true }));
  return { inputDir, publicDir };
}

async function writeCapture(path, format = "png") {
  const image = sharp({
    create: { width: 1979, height: 1135, channels: 4, background: "#1a1d1b" },
  });
  await (format === "png" ? image.png() : image.jpeg()).toFile(path);
}

async function writeMarkedCapture(path) {
  const safeArea = await sharp({
    create: { width: 1924, height: 1135, channels: 4, background: "#00ff00" },
  }).png().toBuffer();
  await sharp({
    create: { width: 1979, height: 1135, channels: 4, background: "#ff0000" },
  }).composite([{ input: safeArea, left: 55, top: 0 }]).png().toFile(path);
}

async function snapshotFiles(root, relative = "") {
  const current = join(root, relative);
  const entries = await readdir(current, { withFileTypes: true });
  const snapshot = {};
  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    const child = join(relative, entry.name);
    if (entry.isDirectory()) Object.assign(snapshot, await snapshotFiles(root, child));
    else snapshot[child] = (await readFile(join(root, child))).toString("hex");
  }
  return snapshot;
}

async function assertNoTransientDirectories(publicDir) {
  const entries = await readdir(publicDir);
  assert.deepEqual(entries.filter(name => name.startsWith(".product-media-")), []);
}

test("rejects a missing third capture without changing published output", async t => {
  const optimizeProductImages = await loadOptimizer();
  const { inputDir, publicDir } = await createFixture(t);
  await writeCapture(join(inputDir, "workspace.png"));
  await writeCapture(join(inputDir, "diff.png"));
  const before = await snapshotFiles(publicDir);

  await assert.rejects(
    optimizeProductImages({ inputDir, publicDir, log: () => {} }),
    /rebase\.png.*missing/i,
  );

  assert.deepEqual(await snapshotFiles(publicDir), before);
  await assertNoTransientDirectories(publicDir);
});

test("rejects JPEG content renamed to png without changing published output", async t => {
  const optimizeProductImages = await loadOptimizer();
  const { inputDir, publicDir } = await createFixture(t);
  await Promise.all([
    writeCapture(join(inputDir, "workspace.png")),
    writeCapture(join(inputDir, "diff.png")),
    writeCapture(join(inputDir, "rebase.png"), "jpeg"),
  ]);
  const before = await snapshotFiles(publicDir);

  await assert.rejects(
    optimizeProductImages({ inputDir, publicDir, log: () => {} }),
    /rebase\.png.*PNG/i,
  );

  assert.deepEqual(await snapshotFiles(publicDir), before);
  await assertNoTransientDirectories(publicDir);
});

test("publishes exactly 18 responsive variants and one OG image", async t => {
  const optimizeProductImages = await loadOptimizer();
  const { inputDir, publicDir } = await createFixture(t);
  await Promise.all(names.map(name => writeCapture(join(inputDir, `${name}.png`))));

  await optimizeProductImages({ inputDir, publicDir, log: () => {} });

  const expectedImages = names
    .flatMap(name => widths.flatMap(width => formats.map(format => `${name}-${width}.${format}`)))
    .sort();
  assert.deepEqual((await readdir(join(publicDir, "images"))).sort(), expectedImages);
  assert.deepEqual(await readdir(join(publicDir, "og")), ["pluck-cover.png"]);

  for (const name of names) {
    for (const width of widths) {
      for (const format of formats) {
        const metadata = await sharp(join(publicDir, "images", `${name}-${width}.${format}`)).metadata();
        assert.equal(metadata.width, width);
        assert.equal(metadata.height, width === 1280 ? 755 : 1133);
        assert.equal(metadata.format, format === "avif" ? "heif" : format);
        assertMetadataStripped(metadata, `${name}-${width}.${format}`);
      }
    }
  }
  const ogMetadata = await sharp(join(publicDir, "og", "pluck-cover.png")).metadata();
  assert.deepEqual(
    { format: ogMetadata.format, width: ogMetadata.width, height: ogMetadata.height },
    { format: "png", width: 1200, height: 630 },
  );
  assertMetadataStripped(ogMetadata, "pluck-cover.png");
  await assertNoTransientDirectories(publicDir);
});

test("removes the 55px private repository rail before responsive and OG processing", async t => {
  const optimizeProductImages = await loadOptimizer();
  const { inputDir, publicDir } = await createFixture(t);
  await Promise.all(names.map(name => writeMarkedCapture(join(inputDir, `${name}.png`))));

  await optimizeProductImages({ inputDir, publicDir, log: () => {} });

  for (const name of names) {
    const { data, info } = await sharp(join(publicDir, "images", `${name}-1920.png`))
      .raw()
      .toBuffer({ resolveWithObject: true });
    assert.deepEqual({ width: info.width, height: info.height }, { width: 1920, height: 1133 });
    assert.ok(data[1] > data[0], `${name} responsive output must start in the green safe area`);
  }

  const { data: ogData } = await sharp(join(publicDir, "og", "pluck-cover.png"))
    .extract({ left: 8, top: 315, width: 1, height: 1 })
    .raw()
    .toBuffer({ resolveWithObject: true });
  assert.ok(ogData[1] > ogData[0], "OG output must start in the green safe area");
});
