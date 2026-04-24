#!/usr/bin/env bash
# Remove the example `Note` domain from the template.
#
# Strategy: find files tagged `@EXAMPLE-FILE` and delete them; find files
# containing `@EXAMPLE-BLOCK-START` / `@EXAMPLE-BLOCK-END` pairs and strip
# those regions. Same mechanism used by `scripts/bulk-rename.sh` (git ls-files
# + grep + sed), which means new file types introduced later are covered
# automatically — no hardcoded file list to drift.
#
# Post-strip we run `cargo fmt` to normalize whitespace artifacts that sed
# deletion leaves behind (trailing blank lines, single-field struct literals
# that shrink after a field is removed, etc.). That's purely mechanical
# cleanup of the strip itself, not a code rewrite. Anything beyond that
# (e.g. dead dependencies in Cargo.toml, unused imports) is left for you to
# notice and decide about.
set -euo pipefail

# 1. Delete whole files tagged @EXAMPLE-FILE.
mapfile -t FILES_TO_DELETE < <(
    git ls-files \
        | grep -v '^scripts/clean-examples\.sh$' \
        | xargs grep -l "@EXAMPLE-FILE" 2>/dev/null || true
)

for f in "${FILES_TO_DELETE[@]}"; do
    rm -v "$f"
done

# 2. Strip @EXAMPLE-BLOCK-START ... @EXAMPLE-BLOCK-END regions from remaining
#    files. The -i.bak / rm .bak dance keeps this portable across GNU and BSD
#    sed.
mapfile -t FILES_WITH_BLOCKS < <(
    git ls-files \
        | grep -v '^scripts/clean-examples\.sh$' \
        | xargs grep -l "@EXAMPLE-BLOCK-START" 2>/dev/null || true
)

for f in "${FILES_WITH_BLOCKS[@]}"; do
    sed -i.bak '/@EXAMPLE-BLOCK-START/,/@EXAMPLE-BLOCK-END/d' "$f"
    rm -f "$f.bak"
done

# 3. Normalize whitespace artifacts sed leaves behind (trailing blank lines,
#    single-field struct literals with dangling commas, etc.). Only runs if
#    `cargo` is on PATH — skips silently otherwise so the strip itself is
#    still useful in minimal environments.
if command -v cargo >/dev/null 2>&1; then
    cargo fmt >/dev/null 2>&1 || true
fi

echo ""
echo "Examples stripped:"
echo "  ${#FILES_TO_DELETE[@]} whole file(s) deleted"
echo "  ${#FILES_WITH_BLOCKS[@]} file(s) had example blocks removed"
if command -v cargo >/dev/null 2>&1; then
    echo "  cargo fmt run to normalize post-strip whitespace"
fi
echo ""
echo "Next steps:"
echo "  1. Run 'just check' to confirm the remaining scaffold still compiles."
echo "  2. Review 'jj diff' before committing."
echo "  3. Optional cleanup: 'cargo machete' to find dependencies that only"
echo "     the removed Note domain was using."
