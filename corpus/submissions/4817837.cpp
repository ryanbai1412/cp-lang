#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    int n, m;
    cin >> n >> m;
    bool graph[n][n] {};
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u][v] = graph[v][u] = true;
    }
    auto add = [&](int x, int diff) -> int {
        x += diff;
        if (x >= n) {
            return x - n;
        }
        if (x < 0) {
            return x + n;
        }
        return x;
    };
    bool dp[n][n][2] {}, trans[n][n][2];
    for (int i = 0; i < n; i++) {
        for (auto& a : dp[i][add(i, -1)]) {
            a = true;
        }
    }
    for (int i = n - 2; i >= 0; i--) {
        for (int l = 0; l < n; l++) {
            int r = add(l, i);
            for (int j = 0; j < 2; j++) {
                int cur = j ? r : l;
                {
                    int k = add(l, -1);
                    if (graph[cur][k] && dp[k][r][0]) {
                        trans[l][r][j] = false;
                        dp[l][r][j] = true;
                    }
                }
                {
                    int k = add(r, +1);
                    if (graph[cur][k] && dp[l][k][1]) {
                        trans[l][r][j] = true;
                        dp[l][r][j] = true;
                    }
                }
            }
        }
    }
    for (int i = 0; i < n; i++) {
        if (dp[i][i][0]) {
            int l = i, r = i, j = 0;
            cout << i + 1 << endl;
            while (add(l, -1) != r) {
                if (!trans[l][r][j]) {
                    cout << (l = add(l, -1)) + 1 << endl;
                    j = 0;
                } else {
                    cout << (r = add(r, +1)) + 1 << endl;
                    j = 1;
                }
            }
            return;
        }
    }
    cout << -1 << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}