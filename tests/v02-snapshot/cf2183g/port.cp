// Source: Codeforces 2183 G (Snake Instructions), https://codeforces.com/contest/2183/problem/G
// Problem: interactive, multitest. n <= 1e5 snakes at distinct positions a_1 < ... <= 4n with
//   unknown speeds in 0..=2. A query is a string of L/R (length <= 4n); all snakes move with
//   it, the faster of two that meet is removed, and the survivors' final positions are
//   returned. With at most 3 queries print every speed, or -1 if no 3 queries can tell.
// Solution: ask "R", "RL", "L". "RL" retraces "R", so it has the same survivors back at
//   their start, which gives their speeds r - v. Every removed snake has speed >= 1;
//   resolve them right to left: next to a speed-0 snake at a[i] + 1, it is ambiguous (-1)
//   when a speed-0 snake also sits at a[i] - 1, else speed 1 iff "L" left a snake at
//   a[i] - 1; otherwise its speed is 2. O(n) per test.

@!interactive

fn ask(s: Str) -> List<Int>:
    print("?", s)
    let k: Int = read()
    let got: [0..k]Int = read()
    got.collect()

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=4 * n
let a: [0..n]1..=4 * n = read()
let vr = ask("R")
let vrl = ask("RL")
let vl = ask("L")

var ans: [0..n]?(0..=2) = None
var idx: [Pos]?Int = None
for (i, x) in a.items: idx[Pos(x)] = i
for (v, r) in zip(vrl, vr): ans[idx[Pos(v)]!] = r - v

var vl_has: [Pos]Bool = false
for v in vl if v >= 1: vl_has[Pos(v)] = true

for i in (0..n).rev if ans[i] == None:
    if i + 1 < n and ans[i + 1] == 0 and a[i + 1] == a[i] + 1:
        if i > 0 and ans[i - 1] == 0 and a[i - 1] == a[i] - 1:
            print("!", -1)
            return
        ans[i] = if a[i] > 1 and vl_has[Pos(a[i] - 1)]: 1 else: 2
    else:
        ans[i] = 2

print("!", [x! for x in ans])
