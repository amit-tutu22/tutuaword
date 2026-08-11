#!/usr/bin/env bash
# Report criterion benchmarks that regressed beyond a percentage threshold.
#
# Phase 1 policy (docs/performance-budgets.md): report only, never fail the build.
# Criterion on shared CI runners is too noisy to gate on; hard latency thresholds
# live in the release tests in .github/workflows/ci.yml.
set -euo pipefail
cd "$(dirname "$0")/.."

THRESHOLD="${1:-10}"
CRITERION_DIR="target/criterion"

if [[ ! -d "$CRITERION_DIR" ]]; then
  echo "No criterion output at $CRITERION_DIR — run 'cargo bench' first."
  exit 0
fi

regressions=0

# estimates.json holds the current run; change/estimates.json holds the delta
# versus the stored baseline, as a fraction (0.05 == 5% slower).
while IFS= read -r change_file; do
  bench_dir="$(dirname "$(dirname "$change_file")")"
  bench_name="${bench_dir#"$CRITERION_DIR"/}"

  pct="$(python3 -c "
import json,sys
try:
    with open('$change_file') as f:
        d = json.load(f)
    print(round(d['mean']['point_estimate'] * 100, 2))
except Exception:
    print('')
" 2>/dev/null || echo '')"

  [[ -z "$pct" ]] && continue

  if python3 -c "import sys; sys.exit(0 if $pct > $THRESHOLD else 1)" 2>/dev/null; then
    echo "REGRESSION: $bench_name is ${pct}% slower (threshold ${THRESHOLD}%)"
    regressions=$((regressions + 1))
  else
    echo "ok: $bench_name ${pct}%"
  fi
done < <(find "$CRITERION_DIR" -path '*/change/estimates.json' 2>/dev/null)

if [[ "$regressions" -gt 0 ]]; then
  echo ""
  echo "$regressions benchmark(s) regressed more than ${THRESHOLD}%."
  echo "Phase 1: reported, not blocking. Tighten in Phase 3 per docs/performance-budgets.md."
fi

exit 0
