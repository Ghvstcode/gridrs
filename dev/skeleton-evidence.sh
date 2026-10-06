#!/bin/sh
# Run inside the rust image at the repo root (on a throwaway copy). For every task: add its
# marker, run its verify command, record the outcome, remove the marker.
set -u
export GRIDRS_CLIPPY=off
printf '%-28s %-6s %s\n' TASK VERIFY "ACCEPTANCE RESULT ON THE UNTOUCHED SKELETON"
for f in tasks/*.yaml; do
  id=$(basename "$f" .yaml)
  : > "ci/done/$id"
  out=$(sh ci/check.sh "$id" 2>&1); code=$?
  rm -f "ci/done/$id"
  res=$(printf '%s\n' "$out" | grep -E '^test result:' | tail -1 | sed 's/; 0 ignored.*//')
  [ $code -eq 0 ] && v=PASS || v=FAIL
  printf '%-28s %-6s %s\n' "$id" "$v(exit $code)" "$res"
done
