// expect: E0302
// `Dsu<V>(n)` is not the C++ `DSU(n)` of `n` vertices: `V` already gives the vertices, and the positional argument is `rollback: Bool`.
let d = Dsu<0..5>(5) // <- here
