// YAC 6 P5 — original: corpus/submissions/5149823.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)
// Rethought: the original keeps a running median in two multisets from each
// end; on the sorted line a prefix sum gives the same gather cost in O(1).

let n: 1..=200_000 = read()
let pts: [0..n](x: Int, y: Int, v: 0..=3) = read()

// best cost on one axis: gather everyone at one median, or cut the sorted line
// in two and gather each side, paying 1 for every point on the wrong side
fn axis(line: List<(at: Int, low: Bool)>) -> Int:
    let p = sorted(line)
    let pre = prefix(q.at for q in p)
    let lows = prefix(Int(q.low) for q in p)

    fn gather(l: Int, r: Int) -> Int:
        let h = (r - l) / 2
        pre.sum(r - h..r) - pre.sum(l..l + h)

    let cut = min(gather(0, i) + gather(i, n) + (i - lows.sum(..i)) + lows.sum(i..)
                  for i in 0..=n)!
    min(gather(0, n), cut)

print(axis([(q.x, q.v <= 1) for q in pts]) + axis([(q.y, q.v == 0 or q.v == 3) for q in pts]))
