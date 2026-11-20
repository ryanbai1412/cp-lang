// IOI '18 P4 - Mechanical Doll (DMOJ: ioi18p4)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// A ball starts at the origin (device 0) and must visit triggers
// A[0], A[1], .., A[n-1] in this order and then return to the origin, with
// every switch back in state X at the end. A trigger has one exit; a switch
// has exits X and Y and alternates between them, X first. Build the circuit
// with at most n + log2(n) switches. Triggers are 1..=M, switches are
// numbered -1, -2, ...
//
// Send every trigger (and the origin) to one root switch, -1. Under it hang a
// complete binary tree with 2^L >= n + 1 leaves. The t-th ball to enter the
// root takes the path whose bits are t read from the low end, so it lands on
// the leaf at tree position bitrev(t). Give the unused visits to the
// leftmost tree positions and send those back to the root; a subtree that is
// entirely unused is not built at all, which keeps the count at n + L. The
// used leaves, taken in visit order, get A[0], .., A[n-1] and finally 0.
@grader
fn create_circuit(M: Int, A: List<Int>) -> (C: List<Int>, X: List<Int>, Y: List<Int>):
    let n = A.len + 1                         // targets: A, then the origin
    let L = bit_len(n - 1).max(1)
    let leaves = 1 << L
    let unused = leaves - n
    let targets = A + [0]

    // tree position p is the visit number bitrev(p); used positions in visit order
    let visit = sorted([p for p in unused..leaves], by: p => bitrev(p, L))
    let target = Map(zip(visit, targets))

    var (X, Y): (List<Int>, List<Int>) = ([], [])
    fn build(lo: Int, hi: Int) -> Int:
        if hi <= unused: return -1                        // nothing used below: back to the root
        if hi - lo == 1: return target[lo]
        let s = X.len
        X.push(0)
        Y.push(0)
        let mid = (lo + hi) / 2
        X[s] = build(lo, mid)
        Y[s] = build(mid, hi)
        -(s + 1)
    build(0, leaves)

    ([-1] * (M + 1), X, Y)

@checker(create_circuit)
fn runs(M: Int, A: List<Int>, C: List<Int>, X: List<Int>, Y: List<Int>) -> Bool:
    var (at, seen, state) = (C[0], [], [false] * X.len)
    while at != 0:
        if at > 0:
            seen.push(at)
            at = C[at]
        else:
            let s = -at - 1
            at = if state[s]: Y[s] else: X[s]
            state[s] = not state[s]
    seen == A and not any(state) and X.len <= A.len + bit_len(A.len)
