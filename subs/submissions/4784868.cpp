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
#define sz(x) int(size(x))

const int maxn = 1e6, dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

int t, artic, vis[maxn], tin[maxn], low[maxn];
vector<int> graph[maxn];

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void dfs(int u, int p = -1) {
    low[u] = tin[u] = t++;
    vis[u] = 1;
    int ch_cnt = 0;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        if (vis[v]) {
            low[u] = min(low[u], tin[v]);
        } else {
            dfs(v, u);
            low[u] = min(low[u], low[v]);
            ch_cnt++;
        }
    }
    if ((p == -1 && ch_cnt >= 2) || (p != -1 && low[u] >= tin[u])) {
        artic = u;
    }
}

void solve(int n, int m, vector<pair<int, int>> black) {
    vector<vector<char>> arr(n, vector<char>(m));
    for (auto& [x, y] : black) {
        arr[x][y] = true;
    }
    vector<pair<int, int>> white;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (!arr[i][j]) {
                white.emplace_back(i, j);
            }
        }
    }
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x < n && 0 <= y && y < m;
    };
    auto bfs = [&](int x, int y) -> int {
        int ans = 0;
        vector<vector<char>> vis(n, vector<char>(m));
        vector<pair<int, int>> q;
        auto push = [&](int x, int y) -> void {
            if (!vis[x][y]) {
                ans++;
                vis[x][y] = true;
                q.emplace_back(x, y);
            }
        };
        push(x, y);
        while (sz(q)) {
            auto [x, y] = q.back();
            q.pop_back();
            for (int i = 0; i < 4; i++) {
                int cx = x + dx[i], cy = y + dy[i];
                if (ibs(cx, cy) && !arr[cx][cy]) {
                    push(cx, cy);
                }
            }
        }
        return ans;
    };
//    if (sz(white) <= 5) {
//        int ans = 1e9;
//        for (int i = 0; i < (1 << sz(white)); i++) {
//            int picked_cnt = __builtin_popcount(i),
//                left_cnt = sz(white) - picked_cnt;
//            if (left_cnt <= 1) {
//                continue;
//            }
//            auto toggle = [&]() -> void {
//                for (int j = 0; j < sz(white); j++) {
//                    if (on(i, j)) {
//                        auto& [x, y] = white[j];
//                        arr[x][y] ^= true;
//                    }
//                }
//            };
//            toggle();
//            int x = -1, y = -1;
//            for (int j = 0; j < sz(white); j++) {
//                if (!on(i, j)) {
//                    auto& [cx, cy] = white[j];
//                    x = cx;
//                    y = cy;
//                }
//            }
//            assert(x != -1);
//            if (bfs(x, y) < left_cnt) {
//                ans = min(ans, picked_cnt);
//            }
//            toggle();
//        }
//        if (ans == 1e9) {
//            cout << -1 << endl;
//        } else {
//            cout << ans << endl;
//        }
//        return;
//    }
    if (sz(white) <= 1) {
        cout << -1 << endl;
        return;
    }
    auto [wx, wy] = white[0];
    if (bfs(wx, wy) < sz(white)) {
        cout << 0 << endl;
        return;
    }
    if (sz(white) == 2) {
        cout << -1 << endl;
        return;
    }
    for (int i = 0; i < n * m; i++) {
        graph[i].clear();
    }
    auto id = [&](int x, int y) -> int {
        return x * m + y;
    };
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (!arr[i][j]) {
                for (int k = 0; k < 4; k++) {
                    int cx = i + dx[k], cy = j + dy[k];
                    if (ibs(cx, cy) && !arr[cx][cy]) {
                        graph[id(i, j)].push_back(id(cx, cy));
                    }
                }
            }
        }
    }
    artic = -1;
    fill(vis, vis + n * m, 0);
    dfs(id(wx, wy));
    if (artic != -1) {
        cout << 1 << endl;
    } else {
        cout << 2 << endl;
    }
}

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    vector<pair<int, int>> arr(k);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        x--;
        y--;
    }
    auto comp = [&](vector<int> arr) -> map<int, int> {
        sort(begin(arr), end(arr));
        map<int, int> ans;
        int cur_orig = arr[0], cur_comp = arr[0];
        for (auto& a : arr) {
            cur_comp += min(3, a - cur_orig);
            cur_orig = a;
            ans[a] = cur_comp;
        }
        return ans;
    };
    vector<int> xs {-1, n}, ys {-1, m};
    for (auto& [x, y] : arr) {
        xs.push_back(x);
        ys.push_back(y);
    }
    auto xcomp = comp(xs), ycomp = comp(ys);
    for (auto& [x, y] : arr) {
        x = xcomp[x];
        y = ycomp[y];
    }
    n = xcomp[n];
    m = ycomp[m];
    dbg(n, m);
    for (auto& [x, y] : arr) {
        dbg(x, y);
    }
    if (long(n) * m <= 1e6) {
        solve(n, m, arr);
    } else {
        cout << -2 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}