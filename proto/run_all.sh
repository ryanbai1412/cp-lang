#!/bin/bash
# Regression: every port must agree with its original; every tests/bad/*.cp must fail to compile.
cd "$(dirname "$0")"
ulimit -s unlimited 2>/dev/null   # some originals keep megabytes of tables on the stack
fail=0
for f in ports/*.cp; do
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
    python3 cp.py build "$f" >/dev/null 2>&1; got=$?
    [[ $got -ne 0 ]] && got=1
    [[ $got -eq $want ]] || { echo "UNEXPECTED: $f exit $got"; fail=1; }
done
echo "bad tests checked"; exit $fail
