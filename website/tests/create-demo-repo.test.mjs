import assert from "node:assert/strict";
import { access, chmod, mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import test from "node:test";

const repo = "/tmp/pluck-website-demo";
const script = resolve(dirname(fileURLToPath(import.meta.url)), "../scripts/create-demo-repo.sh");

function run(command, args, env) {
  return spawnSync(command, args, { encoding: "utf8", env });
}

function git(args, env) {
  const result = run("git", ["-C", repo, ...args], env);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.replace(/\n$/, "");
}

function runDemo(env) {
  const result = run("bash", [script], env);
  assert.equal(result.status, 0, `stdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
}

function commitHashes(env) {
  return git(["rev-list", "--all"], env).split("\n").sort();
}

test("creates deterministic demo history isolated from hostile Git configuration", async t => {
  const root = await mkdtemp(join(tmpdir(), "pluck-hostile-git-"));
  const hooks = join(root, "hooks");
  const hookMarker = join(root, "hook-ran");
  const config = join(root, "hostile.gitconfig");
  const preCommit = join(hooks, "pre-commit");
  await mkdir(hooks, { recursive: true });
  await writeFile(preCommit, `#!/usr/bin/env bash\ntouch '${hookMarker}'\nexit 1\n`);
  await chmod(preCommit, 0o755);
  await writeFile(config, [
    "[user]",
    "  name = Private User",
    "  email = private@example.invalid",
    "[commit]",
    "  gpgSign = true",
    "[core]",
    `  hooksPath = ${hooks}`,
    "",
  ].join("\n"));
  t.after(() => rm(root, { recursive: true, force: true }));
  t.after(() => rm(repo, { recursive: true, force: true }));

  const env = {
    ...process.env,
    HOME: root,
    GIT_CONFIG_GLOBAL: config,
    GIT_CONFIG_SYSTEM: config,
    GIT_AUTHOR_NAME: "Hostile Author",
    GIT_AUTHOR_EMAIL: "hostile-author@example.invalid",
    GIT_COMMITTER_NAME: "Hostile Committer",
    GIT_COMMITTER_EMAIL: "hostile-committer@example.invalid",
  };

  runDemo(env);
  const firstHashes = commitHashes(env);
  assert.equal(firstHashes.length, 5);
  const identities = git(["log", "--all", "--format=%an|%ae|%cn|%ce"], env).split("\n");
  assert.equal(identities.length, 5);
  assert.ok(identities.every(line => line === "Pluck Demo|demo@pluck.local|Pluck Demo|demo@pluck.local"));
  for (const hash of firstHashes) {
    assert.doesNotMatch(git(["cat-file", "-p", hash], env), /^gpgsig /m);
  }
  assert.equal(git(["remote"], env), "");
  assert.equal(git(["branch", "--show-current"], env), "main");
  assert.equal(git(["status", "--short"], env), " M src/components/DiffViewer.vue");
  assert.equal(git(["diff", "--name-only"], env), "src/components/DiffViewer.vue");
  assert.equal(git(["diff", "--cached", "--name-only"], env), "");
  assert.match(git(["diff", "--", "src/components/DiffViewer.vue"], env), /Unified diff with pending review/);
  await assert.rejects(access(hookMarker));

  runDemo(env);
  assert.deepEqual(commitHashes(env), firstHashes);
  await assert.rejects(access(hookMarker));
});
