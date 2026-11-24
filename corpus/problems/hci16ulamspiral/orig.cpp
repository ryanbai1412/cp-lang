// Walk out from the centre (row N/2, column (N-1)/2, 0-based) in runs of 1, 1, 2, 2, 3, 3, ...
// cells going right, up, left, down, numbering cells that fall inside the N x N square.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    scanf("%d", &n);
    vector<vector<int>> g(n, vector<int>(n, 0));
    int r = n / 2, c = (n - 1) / 2, k = 1;
    int dr[] = {0, -1, 0, 1}, dc[] = {1, 0, -1, 0};
    g[r][c] = k++;
    for (int run = 1, d = 0; k <= n * n; d = (d + 1) % 4) {
        for (int s = 0; s < run && k <= n * n; s++) {
            r += dr[d], c += dc[d];
            if (0 <= r && r < n && 0 <= c && c < n) g[r][c] = k++;
        }
        if (d % 2) run++;
    }
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++) printf("%3d%c", g[i][j], j + 1 < n ? ' ' : '\n');
}
