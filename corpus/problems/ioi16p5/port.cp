// Source: IOI 2016 P5 (Unscrambling a Messy Bug), https://dmoj.ca/problem/ioi16p5
// Problem: interactive. A hidden permutation p of n bits (n = 2^b <= 128). Add at most w
//   n-bit strings to a set, compile once (every string's bits get reordered by p), then ask
//   at most r membership queries; recover p. Hardest subtask: n = 128, w = r = 896.
// Solution: divide and conquer on the bits of the index. At depth d the positions of a group
//   share their lower d bits; write, for each position a in the group with bit d set, the
//   string of 1s outside the group, 0s inside except at a. After compiling, query the same
//   shape for every image position: the query hits exactly when it lands on a written
//   string, which gives bit d of its preimage and splits the group. (n/2) log n writes,
//   n log n reads.
// Stdio-adapted: "A s", "C" and "Q s" are the three library calls, "!" prints the answer.
// original: corpus/submissions/4692325.cpp
// Bounds: n: 2..=128; w_lim, r_lim: 1..=10000
let n: 2..=128 = read()
let w_lim: 1..=10000 = read()
let r_lim: 1..=10000 = read()
domain Pos = 0..n

fn build_mask(cur: List<Int>, pick: Int) -> [Pos]Char:
    var in_cur: [Pos]Bool = false
    for idx in 0..cur.len:
        in_cur[Pos(cur[idx])] = true
    var s: [Pos]Char = '1'
    for p in Pos:
        if in_cur[p] and p.label != pick:
            s[p] = '0'
    s

fn add_elem(s: [Pos]Char):
    let ack: Int = ask("A", s)
    assert ack == 1

fn compile_all():
    let ack: Int = ask("C")
    assert ack == 1

fn check_elem(s: [Pos]Char) -> Bool:
    let rep: 0..=1 = ask("Q", s)
    rep == 1

fn write_phase(bit: Int, cur: List<Int>):
    if cur.len <= 1:
        return
    var lo: List<Int> = []
    var hi: List<Int> = []
    for idx in 0..cur.len:
        let a = cur[idx]
        if ((a / (1 << bit)) % 2) == 1:
            add_elem(build_mask(cur, a))
            hi.push(a)
        else:
            lo.push(a)
    write_phase(bit + 1, hi)
    write_phase(bit + 1, lo)

var ans: [Pos]Int = 0

fn read_phase(bit: Int, cur: List<Int>, mut out: [Pos]Int):
    if cur.len <= 1:
        return
    var lo: List<Int> = []
    var hi: List<Int> = []
    for idx in 0..cur.len:
        let a = cur[idx]
        if check_elem(build_mask(cur, a)):
            out[Pos(a)] += 1 << bit
            hi.push(a)
        else:
            lo.push(a)
    read_phase(bit + 1, hi, mut out)
    read_phase(bit + 1, lo, mut out)

var all_pos: List<Int> = []
for i in 0..n:
    all_pos.push(i)

write_phase(0, all_pos)
compile_all()
read_phase(0, all_pos, mut ans)
print("!", ans)
