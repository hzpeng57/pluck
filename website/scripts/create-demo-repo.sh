#!/usr/bin/env bash
set -euo pipefail

repo="/tmp/pluck-website-demo"
export GIT_CONFIG_GLOBAL=/dev/null
export GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME="Pluck Demo"
export GIT_AUTHOR_EMAIL="demo@pluck.local"
export GIT_COMMITTER_NAME="Pluck Demo"
export GIT_COMMITTER_EMAIL="demo@pluck.local"

commit_demo() {
  local date="$1"
  local message="$2"
  GIT_AUTHOR_DATE="$date" GIT_COMMITTER_DATE="$date" \
    git -C "$repo" -c commit.gpgSign=false -c core.hooksPath=/dev/null commit -m "$message"
}

rm -rf "$repo"
mkdir -p "$repo/src/components"

git -C "$repo" init -b main
git -C "$repo" config user.name "Pluck Demo"
git -C "$repo" config user.email "demo@pluck.local"

printf '%s\n' '# Pluck Website Demo' 'Sanitized repository used for product screenshots.' > "$repo/README.md"
printf '%s\n' '<template>' '  <main>Repository workspace</main>' '</template>' > "$repo/src/App.vue"
printf '%s\n' '<template>' '  <pre class="diff">Unified diff</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add README.md src/App.vue src/components/DiffViewer.vue
commit_demo "2026-01-01T12:00:00+00:00" "Initial workspace"

printf '%s\n' '<template>' '  <aside>Repository switcher</aside>' '</template>' > "$repo/src/components/RepoSwitcher.vue"
git -C "$repo" add src/components/RepoSwitcher.vue
commit_demo "2026-01-01T12:01:00+00:00" "Add repository switcher"

git -C "$repo" switch -c feature/diff
printf '%s\n' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
commit_demo "2026-01-01T12:02:00+00:00" "Add side-by-side diff"
printf '%s\n' '<script setup lang="ts">' 'const ignoreWhitespace = true;' '</script>' '<template>' '  <div class="split-diff">Side-by-side diff</div>' '</template>' > "$repo/src/components/DiffViewer.vue"
git -C "$repo" add src/components/DiffViewer.vue
commit_demo "2026-01-01T12:03:00+00:00" "Ignore whitespace changes"

git -C "$repo" switch main
git -C "$repo" switch -c feature/rebase
printf '%s\n' '<template>' '  <dialog open>Interactive rebase</dialog>' '</template>' > "$repo/src/components/RebaseTodoDialog.vue"
git -C "$repo" add src/components/RebaseTodoDialog.vue
commit_demo "2026-01-01T12:04:00+00:00" "Add interactive rebase editor"

git -C "$repo" switch main
printf '%s\n' '<template>' '  <pre class="diff">Unified diff with pending review</pre>' '</template>' > "$repo/src/components/DiffViewer.vue"

git -C "$repo" log --oneline --all --decorate --graph
git -C "$repo" status --short
