// IOI 2016 P5, Unscrambling a Messy Bug — ideal CP Lang sketch
// Category: Two-phase IOI Grader Interactive (separate write/read budgets + compile_set phase barrier)
// Original C++: corpus/submissions/4692325.cpp (85 LOC raw, ~55 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ioi16p5.cp (stdio-adapted, verified)

// WISH B19-4: grader ABI (`extern fn` + `export fn`) unified with local `interactor:`
// On an IOI judge, `cpl build` emits the exact C++ grader entrypoints and strips `interactor:`.
// Locally, `cpl test` / `cpl stress` wires `extern fn` directly to the `interactor:` methods!
extern fn add_element(x: Str)
extern fn compile_set()
extern fn check_element(x: Str) -> Bool

export fn restore_permutation(n: 2..=128, w: Int, r: Int) -> List<Int>:
    domain Pos = 0..n

    fn mask(cur: List<Pos>, pick: Pos) -> [Pos]Char:
        var s: [Pos]Char = '1'
        for p in cur if p != pick:
            s[p] = '0'
        s

    fn write_phase(bit: Int, cur: List<Pos>):
        if cur.len <= 1: return
        let (hi, lo) = cur.partition(p => (p.label >> bit) & 1 == 1)
        for p in hi:
            add_element(mask(cur, p))
        write_phase(bit + 1, hi)
        write_phase(bit + 1, lo)

    var ans: [Pos]Int = 0

    fn read_phase(bit: Int, cur: List<Pos>):
        if cur.len <= 1: return
        let (hi, lo) = cur.partition(p => check_element(mask(cur, p)))
        for p in hi:
            ans[p] |= 1 << bit
        read_phase(bit + 1, hi)
        read_phase(bit + 1, lo)

    let all_pos = [p for p in Pos]
    write_phase(0, all_pos)
    compile_set()
    read_phase(0, all_pos)
    [ans[p] for p in Pos]

// WISH B19-4 + B19-3: stateful two-phase interactor with separate phase limits and power-of-2 generator
interactor:
    gen (n, w, r) with hidden perm:
        let k = rng.int(1..=7)                   // n = 2^k, non-typeable power-of-2 constraint
        let n = 1 << k
        let w = n * k / 2
        let r = n * k
        let perm = rng.perm(0..n)
        yield (n, w, r) with perm

    var raw: Set<Str> = {}
    var compiled = false
    var scrambled: Set<Str> = {}

    limit add_element <= w
    limit check_element <= r
    limit compile_set == 1

    on add_element(x: Str):
        require not compiled, "add_element called after compile_set"
        require x.len == n and all(ch in "01" for ch in x)
        raw.insert(x)

    on compile_set():
        require not compiled, "compile_set called twice"
        compiled = true
        for s in raw:
            scrambled.insert(Str([s[perm[i]] for i in 0..n]))

    on check_element(x: Str) -> Bool:
        require compiled, "check_element called before compile_set"
        require x.len == n and all(ch in "01" for ch in x)
        x in scrambled

    check(got: List<Int>):
        require compiled
        require got == perm
