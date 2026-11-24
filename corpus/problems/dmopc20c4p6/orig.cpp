// The answer is always N (official editorial): with x_i = 1 for opinion 1, child i pairs up
// iff sum over friends of x_j + deg_i * x_i = deg_i (mod 2), a symmetric GF(2) system whose
// right side is its diagonal, so it is consistent. Gauss-Jordan with bitsets.
#include <bits/stdc++.h>
using namespace std;
const int MX = 2000;
bitset<MX + 1> row[MX];
int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<int> deg(n);
    for (int i = 0; i < m; i++) {
        int a, b;
        scanf("%d %d", &a, &b);
        a--, b--;
        row[a][b] = row[b][a] = 1;
        deg[a]++, deg[b]++;
    }
    for (int i = 0; i < n; i++)
        if (deg[i] % 2) row[i][i] = 1, row[i][n] = 1;
    vector<int> pivot_col(n, -1);
    int r = 0;
    for (int c = 0; c < n && r < n; c++) {
        int p = -1;
        for (int i = r; i < n; i++)
            if (row[i][c]) { p = i; break; }
        if (p < 0) continue;
        swap(row[p], row[r]);
        for (int i = 0; i < n; i++)
            if (i != r && row[i][c]) row[i] ^= row[r];
        pivot_col[r++] = c;
    }
    vector<int> x(n, 0);
    for (int i = 0; i < r; i++) x[pivot_col[i]] = row[i][n];
    printf("%d\n", n);
    for (int i = 0; i < n; i++) printf("%d%c", x[i] ? 1 : 2, i + 1 < n ? ' ' : '\n');
}
