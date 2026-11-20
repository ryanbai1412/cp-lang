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

constexpr int maxn = 605;

int arr[maxn], dfs_vis[maxn];
bool cyc;
vector<int> graph[maxn];

void dfs(int u) {
    if (dfs_vis[u]) {
        if (dfs_vis[u] == 1) {
            cyc = true;
        }
        return;
    }
    dfs_vis[u] = 1;
    for (auto& v : graph[u]) {
        dfs(v);
    }
    dfs_vis[u] = 2;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto id = [&](int x, int y) -> int {
        return x * m + y;
    };
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (j + 1 < m) {
                graph[id(i, j)].push_back(id(i, j + 1));
            }
            int kv;
            cin >> arr[id(i, j)] >> kv;
            for (int k = 0; k < kv; k++) {
                int x, y;
                cin >> x >> y;
                graph[id(x, y)].push_back(id(i, j));
            }
        }
    }
    n *= m;
    for (int i = 0; i < n; i++) {
        for (auto& a : graph[i]) {
            dbg(i, a);
        }
    }
    bool bad[n] {}, cvis[n][n];
    for (int i = 0; i < n; i++) {
        memset(dfs_vis, 0, sizeof(dfs_vis));
        cyc = false;
        dfs(i);
        if (cyc) {
            dbg("CYC", i);
            bad[i] = true;
            continue;
        }
        copy(dfs_vis, dfs_vis + n, cvis[i]);
    }
    bool used[n] {};
    long ans = 0;
    while (true) {
        pair<int, int> opt{int(1e9), -1};
        for (int i = 0; i < n; i++) {
            if (used[i] || bad[i]) {
                continue;
            }
            long cans = 0;
            int ccnt = 0;
            for (int j = 0; j < n; j++) {
                if (cvis[i][j]) {
                    cans += arr[j];
                    ccnt += !used[j];
                }
            }
            if (cans >= 0) {
                opt = min(opt, pair<int, int> {ccnt, i});
            }
        }
        int i = opt.second;
        if (i == -1) {
            break;
        }
        for (int j = 0; j < n; j++) {
            if (cvis[i][j]) {
                ans += arr[j];
                used[j] = true;
                arr[j] = 0;
            }
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}