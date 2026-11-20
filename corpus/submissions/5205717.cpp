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

constexpr int maxn = 505;

int n, m, arr[maxn][maxn];
bool vis[maxn][maxn];

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < m;
}

void dfs(int x, int y) {
    if (vis[x][y]) {
        return;
    }
    vis[x][y] = true;
    auto go = [&](int nx, int ny) -> void {
        if (!ibs(nx, ny)) {
            return;
        }
        if (arr[nx][ny] < arr[x][y]) {
            dfs(nx, ny);
        }
    };
    go(x, y - 1);
    go(x, y + 1);
    go(x + 1, y);
    go(x - 1, y);
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            cin >> arr[i][j];
        }
    }
    bitset<maxn> adj[m];
    for (int i = 0; i < m; i++) {
        memset(vis, 0, sizeof(vis));
        dfs(0, i);
        for (int j = 0; j < m; j++) {
            adj[i][j] = vis[n - 1][j];
        }
    }
    bool ign[m] {};
    for (int i = 0; i < m; i++) {
        for (int j = 0; j < m; j++) {
            if (i == j) {
                continue;
            }
            if (adj[i] == adj[j]) {
                ign[max(i, j)] = true;
            } else if ((adj[i] & ~adj[j]).none()) {
                ign[i] = true;
            }
        }
    }
    vector<int> tot;
    bitset<maxn> totb {};
    for (int i = 0; i < m; i++) {
        if (ign[i]) {
            continue;
        }
        tot.push_back(i);
        totb |= adj[i];
        dbg(i, adj[i]);
    }
    auto check = [&](const bitset<maxn>& b) -> bool {
        for (int i = 0; i < m; i++) {
            if (!b[i]) {
                return false;
            }
        }
        return true;
    };
    if (!check(totb)) {
        cout << 0 << endl;
        cout << totb.count() << endl;
    }
    int ans = 0;
    for (int i = 0; i < (1 << sz(tot)); i++) {
        bitset<maxn> cans {};
        for (int j = 0; j < sz(tot); j++) {
            if (on(i, j)) {
                cans |= adj[tot[j]];
            }
        }
        if (check(cans)) {
            ans = max(ans, __builtin_popcount(i));
        }
    }
    cout << 1 << endl;
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}