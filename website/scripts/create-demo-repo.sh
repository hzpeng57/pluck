#!/usr/bin/env bash
set -euo pipefail

repo="/tmp/pluck-website-demo"
rm -rf "$repo"
mkdir -p "$repo/src/components"

git -C "$repo" init -b main
git -C "$repo" config user.name "Pluck Demo"
git -C "$repo" config user.email "demo@pluck.local"

printf '%s\n' '# Pluck Website Demo' 'Sanitized repository used for product screenshots.' > "$repo/README.md"
printf '%s\n' '<template>' '  <main>Repository workspace</main>' '</template>' > "$repo/src/App.vue"
printf '%s\n' '<template>' '  <pre class="diff">Unified diff</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add README.md src/App.vue src/components/DiffViewer.vue
git -C "$repo" commit -m "Initial workspace"

printf '%s\n' '<template>' '  <aside>Repository switcher</aside>' '</template>' > "$repo/src/components/RepoSwitcher.vue"
git -C "$repo" add src/components/RepoSwitcher.vue
git -C "$repo" commit -m "Add repository switcher"

git -C "$repo" switch -c feature/diff
printf '%s\n' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
git -C "$repo" commit -m "Add side-by-side diff"
printf '%s\n' '<script setup lang="ts">' 'const ignoreWhitespace = true;' '</script>' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
git -C "$repo" commit -m "Ignore whitespace changes"

git -C "$repo" switch main
git -C "$repo" switch -c feature/rebase
printf '%s\n' '<template>' '  <dialog open>Interactive rebase</dialog>' '</template>' > "$repo/src/components/RebaseTodoDialog.vue"
git -C "$repo" add src/components/RebaseTodoDialog.vue
git -C "$repo" commit -m "Add interactive rebase editor"

git -C "$repo" switch main
printf '%s\n' '<template>' '  <pre class="diff">Unified diff with pending review</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"

git -C "$repo" log --oneline --all --decorate --graph
git -C "$repo" status --short
