// expect: ok
// rule: 10.1.4
// tool: build {case} -o -
// has: "std::vector<std::vector<int>> create_map(int n, int m, std::vector<int> a, std::vector<int> b);"
// has: "cpl::Judge<cpl::Grid<ll, 2>, std::vector<std::vector<int>>>::to("
// A `[m]T` sized by a value crosses as a `std::vector`, and a `[r, c]T` grid as a vector of rows.
@grader
fn create_map(n: 1..=40, m: Int, a: [m]1..=n, b: [m]1..=n) -> [2 * n, 2 * n]1..=n:
    var s = 0
    for e in a.keys: s += a[e] * b[e]
    return [2 * n, 2 * n] (x, y) => 1 + (x + y + s) % n
