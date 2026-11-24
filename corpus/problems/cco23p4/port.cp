// Source: CCO 2023 P4 (Flip it and Stick it), https://dmoj.ca/problem/cco23p4
// limits: 1000 ms, 1024 MB
// Problem: binary strings S (|S| <= 2e5) and T (|T| <= 3). One move reverses a substring of S;
//   print the fewest moves until S has no occurrence of T, or -1 if impossible.
// Solution: complement so T starts with 0, then a formula per pattern: "0" is 0 or -1; "00"
//   is #00, -1 if zeros > ones + 1; "01", "001", "011" need one move per occurrence; "010"
//   needs ceil(#010 / 2). "000" is -1 if zeros > 2 (ones + 1); otherwise greedy over the
//   zero runs between ones, moving zeros of runs > 2 into empty gaps two at a time, else
//   into gaps of size 1. O(n).
// original: corpus/submissions/5579929.cpp
let s: Str = read()
let t: Str = read()
fn plain(x: Str) -> Str = Str(x.map(c => if c == t[0]: '0' else: '1'))
let (w, p) = (plain(s), plain(t))
let zeros = w.count(c => c == '0')
let ones = w.len - zeros
let hits = w.windows(p.len).count(x => Str(x) == p)

fn spread(gaps: List<Int>) -> Int:
    var empty = gaps.count(g => g == 0)
    var single = gaps.count(g => g == 1)
    var moves = 0
    for whole in gaps:
        var g = whole
        while g > 2:
            moves += 1
            if g == 3 and single > 0:
                single -= 1
                g -= 1
            elif g == 3:
                empty -= 1
                single += 1
                g -= 1
            elif empty > 0:
                empty -= 1
                g -= 2
            else:
                single -= 1
                g -= 1
    moves

print(match p:
    "0" => if zeros > 0: -1 else: 0
    "00" => if zeros > ones + 1: -1 else: hits
    "010" => (hits + 1) / 2
    "000" => if zeros > 2 * (ones + 1): -1 else: spread([g.len for g in w.split('1')])
    _ => hits)
