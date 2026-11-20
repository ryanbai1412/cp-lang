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

constexpr int maxn = 2e6;
// constexpr int maxn = 1000;

struct Indexer {
    int x = 0;

    int operator()() {
        return x++;
    }
};

struct Solve1 {
    int vcnt = 0;
    bool vis[maxn] {};
    vector<int> graph[maxn];

    void add_edge(int u, int v) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    void dfs(int u) {
        if (vis[u]) {
            return;
        }
        vcnt++;
        vis[u] = true;
        for (auto& v : graph[u]) {
            dfs(v);
        }
    }

    bool solve(int n, int m, const vector<pair<int, int>>& iarr) {
        map<int, vector<int>> arr;
        for (auto& [x, y] : iarr) {
            arr[x].push_back(y);
        }
        arr[n] = {};
        vector<array<int, 3>> segs;
        Indexer new_node;
        int prev_k = -1;
        if (arr.begin()->first) {
            segs = {{0, m - 1, new_node()}};
        }
        for (auto& [k, v] : arr) {
            if (k - prev_k > 1) {
                int cind = new_node();
                for (auto& [_l, _r, i] : segs) {
                    add_edge(i, cind);
                }
                segs = {{0, m - 1, cind}};
            }
            if (k == n) {
                continue;
            }
            v.push_back(m);
            sort(begin(v), end(v));
            int l = 0, j = 0;
            vector<array<int, 3>> nsegs;
            for (auto& a : v) {
                if (l < a) {
                    dbg(l, a);
                    int cind = new_node();
                    while (j < sz(segs) && segs[j][1] < l) {
                        j++;
                    }
                    while (j < sz(segs) && segs[j][1] < a) {
                        add_edge(cind, segs[j++][2]);
                    }
                    if (j < sz(segs) && segs[j][0] < a) {
                        add_edge(cind, segs[j][2]);
                    }
                    nsegs.push_back({l, a - 1, cind});
                }
                l = a + 1;
            }
            swap(segs, nsegs);
            prev_k = k;
        }
        dfs(0);
        dbg(vcnt, new_node.x);
        return vcnt < new_node.x;
    }

    static bool go(int n, int m, const vector<pair<int, int>>& arr) {
        return Solve1().solve(n, m, arr);
    }
};

struct Solve2 {
    bool ans = false, vis[maxn] {};
    int t = 0, tin[maxn] {}, low[maxn] {};
    vector<int> artic, graph[maxn];

    void add_edge(int u, int v) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    void dfs(int u, int p = -1) {
        vis[u] = true;
        low[u] = tin[u] = t++;
        int ch_cnt = 0;
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            } else if (vis[v]) {
                low[u] = min(low[u], tin[v]);
            } else {
                dfs(v, u);
                ch_cnt++;
                if (p != -1 && low[v] >= tin[u]) {
                    artic.push_back(u);
                }
                low[u] = min(low[u], low[v]);
            }
        }
        if (p == -1 && ch_cnt >= 2) {
            artic.push_back(u);
        }
    }

    bool solve(int n, int m, const vector<pair<int, int>>& arr) {
        if (!sz(arr)) {
            if (n == 1 || m == 1) {
                return true;
            } else {
                return false;
            }
        }
        auto ibs = [&](int x, int y) -> bool {
            return 0 <= x && x < n && 0 <= y && y < m;
        };
        vector<pair<int, int>> comp;
        int b = min(n, m) == 1 ? 3 : 2;
        for (auto& [x, y] : arr) {
            for (int i = -b; i <= b; i++) {
                for (int j = -b; j <= b; j++) {
                    int cx = x + i, cy = y + j;
                    if (ibs(cx, cy)) {
                        comp.emplace_back(x + i, y + j);
                    }
                }
            }
        }
        sort(begin(comp), end(comp));
        comp.erase(unique(begin(comp), end(comp)), comp.end());
        set<pair<int, int>> arr_s(begin(arr), end(arr));
        vector<pair<int, int>> ncomp;
        for (auto& [x, y] : comp) {
            if (!arr_s.count({x, y})) {
                ncomp.emplace_back(x, y);
            }
        }
        swap(comp, ncomp);
        for (int i = 0; i < sz(comp); i++) {
            auto& [x, y] = comp[i];
            for (int dx : {0, 1}) {
                int dy = 1 - dx;
                int cx = x + dx, cy = y + dy;
                auto it = lower_bound(begin(comp), end(comp),
                                      pair<int, int> {cx, cy});
                if (it != comp.end() && *it == pair<int, int> {cx, cy}) {
                    add_edge(i, int(it - comp.begin()));
                }
            }
        }
        for (int i = 0; i < sz(comp); i++) {
            if (!vis[i]) {
                dfs(i);
            }
        }
        for (auto& a : artic) {
            auto& [x, y] = comp[a];
            const int dx[] = {-1, -1, -1, 0, 0, 1, 1, 1},
                      dy[] = {-1, 0, 1, -1, 1, -1, 0, 1};
            dbg(x, y);
            for (int i = 0; i < 8; i++) {
                int cx = x + dx[i], cy = y + dy[i];
                if (arr_s.count({cx, cy}) || !ibs(cx, cy)) {
                    return true;
                }
            }
        }
        return false;
    }

    static bool go(int n, int m, const vector<pair<int, int>>& arr) {
        return Solve2().solve(n, m, arr);
    }
};

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    vector<pair<int, int>> arr(k);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        x--;
        y--;
    }
    if (Solve1::go(n, m, arr)) {
        cout << 0 << endl;
        return;
    }
    long white_cnt = long(n) * m - k;
    if (white_cnt <= 2) {
        cout << -1 << endl;
        return;
    }
    if (Solve2::go(n, m, arr)) {
        cout << 1 << endl;
    } else {
        cout << 2 << endl;
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