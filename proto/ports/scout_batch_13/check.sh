#!/bin/bash
# Check every today/ form against its original. Usage: proto/ports/scout_batch_13/check.sh [iterations]
# Builds from copies in a temporary folder, because a build stamps its source.
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
it=${1:-100}
tmp=$(mktemp -d)
fail=0
ulimit -s unlimited
orig() { echo "$root/$(sed -n 's|^// original: \(corpus/submissions/[0-9]*\.cpp\).*|\1|p' "$here/today/$1.cp")"; }
report() { echo "$1: $2"; [[ $2 == *OK* ]] || fail=1; }

# typed generator against the original: "name size", size being the generator's size argument
for job in "coci12c1p4 5" "ioi11p3 30" "ioi05p4 30" "tle17c5p6 6" "apio07p2 40"; do
    set -- $job
    cp "$here/today/$1.cp" "$tmp/$1.cp"
    n=$it; [[ $1 == coci12c1p4 ]] && n=$((it * 3))    # the original divides by zero when the answer is 1, so those inputs are rejected
    report $1 "$(cd "$root/proto" && python3 cp.py stress "$tmp/$1.cp" "$(orig $1)" $n $2 2>&1 | tail -1)"
done

# coci21c4p4: any k towns that reach the distance are accepted, and the two
# programs list the same towns in a different order, so compare them sorted
p=coci21c4p4
cp "$here/today/$p.cp" "$tmp/$p.cp"
(cd "$root/proto" && python3 cp.py stress "$tmp/$p.cp" "$(orig $p)" 0 6 > /dev/null 2>&1)    # builds the three programs, runs nothing
same=0
for seed in $(seq 1 $it); do
    "$tmp/${p}_gen" $seed 8 > "$tmp/in"
    ours=$("$tmp/${p}_bin" < "$tmp/in" | { read -r d; echo "$d"; tr ' ' '\n' | grep . | sort -n | tr '\n' ' '; })
    ref=$("$tmp/${p}_ref" < "$tmp/in" | { read -r d; echo "$d"; tr ' ' '\n' | grep . | sort -n | tr '\n' ' '; })
    [[ -n $ours && $ours == "$ref" ]] && same=$((same + 1)) || { echo "$p differs on seed $seed"; break; }
done
[[ $same == "$it" ]] && report $p "OK: $same tests agree (distance, and the towns as a set)" || report $p "FAILED"

# interactive: both sides talk to the same judge
p=cpc21c1p5
cp "$here/today/$p.cp" "$tmp/$p.cp"
report $p "$(python3 "$root/proto/interact.py" "$tmp/$p.cp" "$(orig $p)" "$here/today/${p}_judge.py" $it 4 2>&1 | tail -1)"

rm -rf "$tmp"
exit $fail
