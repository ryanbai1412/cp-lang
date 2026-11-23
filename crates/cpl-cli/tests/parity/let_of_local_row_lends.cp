// R05.3.5: `let prev = ends[j % 2]` lends the row of a local list.
fn f(n: Int) -> Int:
    var ends: List<List<Int>> = [[1, 2], [3]]
    var t = 0
    for j in 0..n:
        let prev = ends[j % 2]
        t += prev.len + (prev.find(x => x > j) ?? 0)
        ends[(j + 1) % 2].push(j)
    t
print(f(6))
