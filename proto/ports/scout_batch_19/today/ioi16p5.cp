// IOI 2016 P5, Unscrambling a Messy Bug (two-phase interactive, stdio-adapted today-form)
// original: corpus/submissions/4692325.cpp
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
