#!/bin/sh
# Maintainer tool used to build the (unpublished) reference branch.
#
#   sh dev/integrate-dry-run.sh <worktrees-dir> <task-id>...
#
# For each task: commit the dry-run worktree <worktrees-dir>/<task-id> (branch
# ref/<task-id>), audit the diff against the task's scope.write, and cherry-pick
# it onto `reference`, which must be checked out at <worktrees-dir>/_reference.
set -eu
WT_DIR="${1:?usage: integrate-dry-run.sh <worktrees-dir> <task-id>...}"; shift
REPO="$(cd "$(dirname "$0")/.." && pwd)"
for id in "$@"; do
    WT="$WT_DIR/$id"
    git -C "$WT" add -A
    git -C "$WT" commit -q -m "feat($id): reference implementation" || true
    base="$(git -C "$WT" merge-base reference "ref/$id")"
    (cd "$REPO" && sh ci/scope-audit.sh diff "$id" "$base" "ref/$id")
    git -C "$WT_DIR/_reference" cherry-pick "ref/$id" >/dev/null
    echo "[$id] on reference as $(git -C "$WT_DIR/_reference" rev-parse --short HEAD)"
done
