// cco24p2's search: a nested fn lent its fn's list hands first_true a closure
// that indexes the list, then falls back with `??`.
let n = 3
if n > 1:
    var tops: List<Int> = []
    var best = 0
    fn search(q: Int):
        let i = (0..tops.len).first_true(i => tops[i] > q) ?? tops.len
        if i == tops.len: tops.push(q)
        else: tops[i] = q
        best += i
    for q in [5, 9, 3, 10, 4]: search(q)
    print(tops, best)
