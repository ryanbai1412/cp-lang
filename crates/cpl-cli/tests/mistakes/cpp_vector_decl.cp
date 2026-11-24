// here: 5
// C++'s `vector<vector<int>> g(n, vector<int>(m));` (self-run mistakes): a bare "expected the end of the statement".
let n = 2
let m = 3
vector<vector<int>> g(n, vector<int>(m));
g[1][2] = 5
print(g[1][2])
