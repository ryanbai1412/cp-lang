#!/bin/bash
# run.sh FILE ID [N]: N random inputs; compare cpl run (checked), native C++ and the brute
F=$1; ID=$2; N=${3:-200}
HERE=$(cd "$(dirname "$0")" && pwd); CPL=${CPL:-cpl}; BIN=${BIN:?native binary}
W=$(mktemp -d); bad=0
one() {
  s=$1; python3 $HERE/stress.py gen $ID $s > $W/$s.in
  $CPL run $HERE/../$F $W/$s.in > $W/$s.interp 2>$W/$s.ierr; ie=$?
  $BIN < $W/$s.in > $W/$s.native 2>/dev/null; ne=$?
  if [ $ID = 1991D ]; then
    python3 $HERE/stress.py valid $ID $W/$s.in $W/$s.interp && python3 $HERE/stress.py valid $ID $W/$s.in $W/$s.native && cmp -s $W/$s.interp $W/$s.native && [ $ie = 0 ] && [ $ne = 0 ] || echo "MISMATCH seed $s"
  else
    python3 $HERE/stress.py brute $ID < $W/$s.in > $W/$s.brute
    cmp -s <(tr -s ' \n' '\n' < $W/$s.brute) <(tr -s ' \n' '\n' < $W/$s.interp) && cmp -s <(tr -s ' \n' '\n' < $W/$s.brute) <(tr -s ' \n' '\n' < $W/$s.native) && [ $ie = 0 ] && [ $ne = 0 ] || echo "MISMATCH seed $s (interp=$ie native=$ne)"
  fi
}
export -f one; export W HERE CPL BIN F ID
seq 1 $N | xargs -P 8 -I{} bash -c 'one {}' | sort -V > $W/report
echo "$F $ID: $(grep -c MISMATCH $W/report) mismatches / $N  (work dir $W)"; head -5 $W/report
