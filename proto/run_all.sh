#!/bin/bash
# Regression: every port must agree with its original; every tests/bad/*.cp must fail to compile.
cd "$(dirname "$0")"
ulimit -s unlimited 2>/dev/null   # some originals keep megabytes of tables on the stack
fail=0
for f in ports/*.cp ports/hillclimb_dp_graph/*.cp; do   # its before/ holds superseded versions, not run here
    b=${f%.cp}
    if [[ -f ${b}_judge.py ]]; then   # interactive: talk to a judge instead of comparing outputs
        r=$(timeout 600 python3 interact.py "$f" "${b}_orig.cpp" "${b}_judge.py" ${ITERS:-200} 6 2>&1 | tail -1)
    else
        r=$(python3 cp.py stress "$f" "${b}_orig.cpp" ${ITERS:-200} 6 2>&1 | tail -1)
    fi
    echo "$(basename $b): $r"; [[ $r == stress\ OK* || $r == interact\ OK* ]] || fail=1
done
for f in tests/bad/*.cp; do
    case $(basename $f) in ok_*|refine.cp) want=0;; *) want=1;; esac
    msg=$(python3 cp.py build "$f" 2>&1 >/dev/null); got=$?
    [[ $got -ne 0 ]] && got=1
    [[ $got -eq $want ]] || { echo "UNEXPECTED: $f exit $got"; fail=1; }
    # a first line `// error: text` locks the message too (decision 5.11): the build must fail with that text
    exp=$(head -1 "$f" | sed -n 's|^// error: ||p')
    [[ -z $exp || $msg == *"$exp"* ]] || { echo "WRONG MESSAGE: $f wanted \"$exp\""; fail=1; }
done
echo "bad tests checked"
# plan/19: Index, lends and lexical borrows. Each tests/lend/x.cp is built, run on x.in and compared with x.out.
# tests/ergo: the vocabulary from research/ergonomics-audit.md (ranges, steps, Counter, prefix, `f=`), run the same way.
tmp=$(mktemp -d)
for f in tests/lend/*.cp tests/ergo/*.cp; do
    b=${f%.cp}
    cp "$f" "$tmp/t.cp"     # built from a copy: the build stamps its source and writes the C++ beside it
    if python3 cp.py build "$tmp/t.cp" >/dev/null 2>&1 && g++ -std=c++17 -O1 -DLOCAL -o "$tmp/t_bin" "$tmp/t.cpp" 2>/dev/null \
        && "$tmp/t_bin" < "$b.in" 2>&1 | cmp -s - "$b.out"; then :
    else echo "RUN TEST FAILED: $f"; fail=1; fi
done
rm -rf "$tmp"
echo "lend and ergo tests checked"
# Red-team edge cases (plan/19 section 6): the ends of Int, Euclidean / and %, evaluation order, empty and
# oversized domains. Built the way `cp run` builds locally, under ASan and UBSan, so a case that must stop
# does so with a `line N: ...` message (kept in x.out) and never with undefined behaviour. ASan can't map
# its shadow memory under the unlimited stack set above, so these run with the usual 8 MB.
tmp=$(mktemp -d)
for f in tests/redteam/*.cp; do
    b=${f%.cp}
    cp "$f" "$tmp/t.cp"     # built from a copy: the build stamps its source, and the line numbers in x.out must not move
    if python3 cp.py build "$tmp/t.cp" >/dev/null 2>&1 \
        && g++ -std=c++17 -O1 -DLOCAL -fsanitize=address,undefined -fno-sanitize-recover=undefined -o "$tmp/t_bin" "$tmp/t.cpp" 2>/dev/null \
        && (ulimit -s 8192; "$tmp/t_bin" < "$b.in" 2>&1) | cmp -s - "$b.out"; then :
    else echo "REDTEAM TEST FAILED: $f"; fail=1; fi
done
rm -rf "$tmp"
echo "redteam tests checked"; exit $fail
