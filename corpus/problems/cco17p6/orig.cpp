// CCO 2017 P6 (Shifty Grid): fill rows 0..N-2 one tile at a time with a column shift, a row
// shift below the solved part and the column shift undone; then fix the last row's parity with
// one shift of it (an M-cycle is odd for even M) and finish it with 3-cycle commutators.
#include <bits/stdc++.h>
using namespace std;

int n, m;
vector<vector<int>> g;
vector<array<int, 3>> ops;

void row(int i, int r) {
    r = ((r % m) + m) % m;
    if (!r) return;
    rotate(g[i].begin(), g[i].end() - r, g[i].end());
    ops.push_back({1, i + 1, r});
}

void col(int j, int d) {
    d = ((d % n) + n) % n;
    if (!d) return;
    vector<int> c(n);
    for (int i = 0; i < n; i++) c[(i + d) % n] = g[i][j];
    for (int i = 0; i < n; i++) g[i][j] = c[i];
    ops.push_back({2, j + 1, d});
}

pair<int, int> find(int t) {
    for (int i = 0; i < n; i++)
        for (int j = 0; j < m; j++)
            if (g[i][j] == t) return {i, j};
    return {-1, -1};
}

// Moves P_s -> P_s', P_s' -> (n-1, j), (n-1, j) -> P_s along the last row, where
// P_x = (n-1, j - x); everything else stays put.
void cycle3(int j, int s, int s2) {
    for (auto [x, sign] : {pair{s, 1}, pair{s2, -1}}) {
        if (sign > 0) col(j, 1), row(n - 1, x), col(j, -1), row(n - 1, -x);
        else row(n - 1, x), col(j, 1), row(n - 1, -x), col(j, -1);
    }
}

int main() {
    scanf("%d %d", &n, &m);
    g.assign(n, vector<int>(m));
    for (auto& r : g)
        for (int& x : r) scanf("%d", &x);
    for (int r = 0; r + 1 < n; r++)
        for (int c = 0; c < m; c++) {
            int t = r * m + c;
            while (true) {
                auto [a, b] = find(t);
                if (a == r && b == c) break;
                if (a == r) {
                    col(b, 1), row(r + 1, 1), col(b, -1);
                } else if (b == c) {
                    row(a, 1);
                } else {
                    int d = a - r;
                    col(c, d), row(a, c - b), col(c, -d);
                }
            }
        }
    vector<int>& last = g[n - 1];
    int base = (n - 1) * m, parity = 0;
    {
        vector<int> p(m);
        for (int j = 0; j < m; j++) p[j] = last[j] - base;
        vector<bool> seen(m);
        for (int j = 0; j < m; j++)
            if (!seen[j]) {
                int len = 0;
                for (int x = j; !seen[x]; x = p[x]) seen[x] = true, len++;
                parity ^= (len - 1) & 1;
            }
    }
    if (parity) row(n - 1, 1);
    for (int c = 0; c < m; c++) {
        int t = base + c;
        if (last[c] == t) continue;
        int b = find(t).second, e = -1;
        for (int x = c + 1; x < m; x++)
            if (x != b) { e = x; break; }
        cycle3(c, ((c - e) % m + m) % m, ((c - b) % m + m) % m);
    }
    printf("%zu\n", ops.size());
    for (auto [k, i, r] : ops) printf("%d %d %d\n", k, i, r);
}
