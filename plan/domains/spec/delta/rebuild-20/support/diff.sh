#!/bin/bash
# usage: diff.sh NN name count
p=$1; f=$2; N=${3:-200}; C=${CPL:-cpl}; R=${SRC:-..}; B=${BIN:-./bin}
mkdir -p fail; bad=0
for sd in $(seq 1 $N); do
  python3 gb.py gen $p $sd > in.$p
  python3 gb.py brute $p < in.$p > br.$p
  $B/$f < in.$p > na.$p 2>&1 || echo "native crash seed $sd" 
  $C run $R/$f.cp in.$p > ip.$p 2>&1 || echo "interp fail seed $sd"
  if ! cmp -s br.$p na.$p || ! cmp -s br.$p ip.$p; then bad=$((bad+1)); cp in.$p fail/$p.$sd.in; [ $bad -le 2 ] && { echo "MISMATCH $p seed $sd"; cat in.$p; echo "-- brute"; cat br.$p; echo "-- native"; cat na.$p; echo "-- interp"; head -5 ip.$p; }; fi
done
echo "$f: $N cases, $bad mismatches"
