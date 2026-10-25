#!/usr/bin/env bash
# Docs consistency gate: every changelog carries the workspace version, every
# translated README pins it in the install snippet, and the per-crate test
# counts in README.md / docs/README.en.md match the real `#[test]` counts.
# Usage: bash scripts/check-docs.sh   (exit 1 with one FAIL line per mismatch)
set -u
cd "$(dirname "$0")/.."

fail=0
fail() {
    echo "FAIL: $*"
    fail=1
}

version=$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version *= *"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)
if [ -z "$version" ]; then
    echo "FAIL: no workspace.package.version in Cargo.toml"
    exit 1
fi

for f in docs/CHANGELOG.md docs/CHANGELOG.*.md; do
    grep -qF "## [$version]" "$f" || fail "$f has no '## [$version]' heading"
done

for f in README.md docs/README.*.md; do
    grep -qF "bee_rust = \"$version\"" "$f" ||
        grep -qE "bee_rust *= *\{[^}]*version *= *\"$version\"" "$f" ||
        fail "$f install snippet does not pin bee_rust = \"$version\""
done

# Crate order matches the tables; keep the two in sync if either changes.
crates="bee_config bee_cache bee_template bee_logs bee_kv bee_search bee_graph bee_tsdb
bee_orm bee_orm_macro bee_session bee_router bee_cli"

check_test_table() {
    local file=$1 sum=0 crate want actual declared
    for crate in $crates; do
        want=$(grep -oE "^\| $crate \| [0-9]+ \|" "$file" | grep -oE '[0-9]+')
        if [ -z "$want" ]; then
            fail "$file has no test-count row for $crate"
            continue
        fi
        actual=$(grep -rE '^\s*#\[(tokio::)?test\]' "crates/$crate" --include='*.rs' | wc -l)
        if [ "$want" -ne "$actual" ]; then
            fail "$file: $crate says $want tests, found $actual"
        fi
        sum=$((sum + want))
    done

    declared=$(grep -oE '[0-9]+ (个测试|tests passing)' "$file" | grep -oE '^[0-9]+' | head -n 1)
    if [ -z "$declared" ]; then
        fail "$file has no '<N> 个测试' / '<N> tests passing' total"
    elif [ "$declared" -ne "$sum" ]; then
        fail "$file total says $declared tests, table sums to $sum"
    fi
}

check_test_table README.md
check_test_table docs/README.en.md

exit "$fail"
