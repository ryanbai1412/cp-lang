// Minimum Manhattan tour through every '#' of a 10x10 map (2 <= nodes <= 20), one answer
// per map until end of input: Held-Karp over subsets, with node 0 fixed as the start.
#include <bits/stdc++.h>
using namespace std;
int main() {
    string row;
    while (true) {
        vector<pair<int, int>> pts;
        int got = 0;
        for (int y = 0; y < 10 && cin >> row; y++, got++)
            for (int x = 0; x < (int)row.size(); x++)
                if (row[x] == '#') pts.push_back({x, y});
        if (got < 10) break;
        int n = pts.size(), m = n - 1;
        vector<vector<int>> d(n, vector<int>(n));
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                d[i][j] = abs(pts[i].first - pts[j].first) + abs(pts[i].second - pts[j].second);
        // dp[S][j]: shortest path from node 0 through S (over nodes 1..m) ending at node j + 1
        const int INF = 1e9;
        vector<int> dp((size_t(1) << m) * m, INF);
        for (int j = 0; j < m; j++) dp[(size_t(1) << j) * m + j] = d[0][j + 1];
        for (size_t S = 1; S < (size_t(1) << m); S++)
            for (int j = 0; j < m; j++) {
                int cur = dp[S * m + j];
                if (cur == INF || !(S >> j & 1)) continue;
                for (int k = 0; k < m; k++)
                    if (!(S >> k & 1)) {
                        int& nx = dp[(S | size_t(1) << k) * m + k];
                        nx = min(nx, cur + d[j + 1][k + 1]);
                    }
            }
        int best = INF;
        size_t full = (size_t(1) << m) - 1;
        for (int j = 0; j < m; j++) best = min(best, dp[full * m + j] + d[j + 1][0]);
        printf("%d\n", best);
    }
}
