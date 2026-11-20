#!/bin/bash
# Check every today/ form against its original. Usage: proto/ports/scout_batch_5/check.sh [iterations]
# Builds from copies in a temporary folder, because a build stamps its source.
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
it=${1:-100}
tmp=$(mktemp -d)
fail=0
orig() { echo "$root/$(sed -n 's|^// original: \(corpus/submissions/[0-9]*\.cpp\).*|\1|p' "$here/today/$1.cp")"; }
report() { echo "$1: $2"; [[ $2 == *OK* || $2 == *agree ]] || fail=1; }

# typed generator against the original
for p in noip10p3 cco22p1 cco23p6 ioi06p2 ioi13p5; do
    cp "$here/today/$p.cp" "$tmp/$p.cp"
    ref=$(orig $p)
    if [[ $p == ioi13p5 ]]; then    # the original is a grader function: give it a main
        { cat "$ref"; cat <<'EOF'
int main() {
    int a, b, t; scanf("%d %d %d", &a, &b, &t);
    vector<int> x(a + 1), y(b + 1), w(t), s(t);
    for (int i = 0; i < a; i++) scanf("%d", &x[i]);
    for (int i = 0; i < b; i++) scanf("%d", &y[i]);
    for (int i = 0; i < t; i++) scanf("%d %d", &w[i], &s[i]);
    printf("%d\n", putaway(a, b, t, x.data(), y.data(), w.data(), s.data()));
}
EOF
        } > "$tmp/robots.cpp"; ref="$tmp/robots.cpp"
    fi
    n=$it; [[ $p == cco22p1 ]] && n=$((it * 8))    # most of its inputs are rejected (K19)
    report $p "$(python3 "$root/proto/cp.py" stress "$tmp/$p.cp" "$ref" $n 6 2>&1 | tail -1)"
done

# interactive: both sides talk to the same judge
for p in cco18p4 yac6p3; do
    cp "$here/today/$p.cp" "$tmp/$p.cp"
    report $p "$(python3 "$root/proto/interact.py" "$tmp/$p.cp" "$(orig $p)" "$here/today/${p}_judge.py" $it 6 2>&1 | tail -1)"
done

# inputs the typed generator cannot make (K4) or a port that only runs on the interpreter (F1)
report "btoi17p1, noi06p4" "$(python3 "$here/feed.py" "$tmp" "$it" 2>&1 | tail -1)"
rm -rf "$tmp"
exit $fail
