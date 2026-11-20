#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

namespace s1 {

void solve(int, const vector<array<int, 4>>& edges) {
    int m = sz(edges);

    vector<pair<int, int>> segs;
    for (auto& [_u, _v, ql, qr] : edges) {
        segs.emplace_back(ql, qr);
    }

    set<pair<int, int>> cs;
    vector<int> evts[m];
    for (int i = 0; i < m; i++) {
        evts[segs[i].first].push_back(i);
    }

    int arr[m];
    for (int i = 0; i < m; i++) {
        for (auto& a : evts[i]) {
            cs.emplace(segs[a].second, a);
        }

        if (!sz(cs) || cs.begin()->first < i) {
            cout << "NO" << endl;
            return;
        }

        arr[cs.begin()->second] = i;
        cs.erase(cs.begin());
    }

    cout << "YES" << endl;
    for (int i = 0; i < m; i++) {
        cout << arr[i] + 1 << " \n"[i == m - 1];
    }
}

}  // namespace s1

namespace brute {

struct Tree {
    int n;
    vector<vector<pair<int, int>>> graph;

    Tree(const vector<array<int, 4>>& edges) : n(sz(edges) + 1), graph(n) {
        for (int i = 0; i < n - 1; i++) {
            auto& [u, v, _l, _r] = edges[i];
            graph[u].emplace_back(v, i);
            graph[v].emplace_back(u, i);
        }
    }

    vector<int> st, ans;
    void dfs(int u, int p, int targ) {
        if (u == targ) {
            ans = st;
        }

        for (auto& [v, ei] : graph[u]) {
            if (v == p) {
                continue;
            }
            st.push_back(ei);
            dfs(v, u, targ);
            st.pop_back();
        }
    }

    vector<int> path(int u, int v) {
        dfs(u, -1, v);
        return ans;
    }
};

void solve(int n, const vector<array<int, 4>>& edges) {
    int m = sz(edges);
    assert(m <= 20);

    Tree tree(vector(begin(edges), begin(edges) + n - 1));

    int dep[m] {};
    for (int i = n - 1; i < m; i++) {
        auto& [u, v, _l, _r] = edges[i];
        auto cp = tree.path(u, v);

        for (auto& a : cp) {
            dep[i] |= 1 << a;
        }
    }

    bool dp[1 << m] {};
    dp[(1 << m) - 1] = true;
    for (int mask = (1 << m) - 2; mask >= 0; mask--) {
        int cw = __builtin_popcount(mask);
        for (int i = 0; i < m; i++) {
            if (!on(mask, i) && edges[i][2] <= cw && cw <= edges[i][3] &&
                (mask & dep[i]) == dep[i]) {
                dp[mask] |= dp[mask | (1 << i)];
            }
        }
    }

    if (!dp[0]) {
        cout << "NO" << endl;
        return;
    }

    cout << "YES" << endl;
    int ans[m];
    {
        int mask = 0;
        while (mask != (1 << m) - 1) {
            int cw = __builtin_popcount(mask);
            for (int i = 0; i < m; i++) {
                if (!on(mask, i) && edges[i][2] <= cw && cw <= edges[i][3] &&
                    (mask & dep[i]) == dep[i] && dp[mask | (1 << i)]) {
                    ans[i] = cw;
                    mask |= 1 << i;
                    break;
                }
            }
        }
    }
    for (int i = 0; i < m; i++) {
        cout << ans[i] + 1 << " \n"[i == m - 1];
    }
}

}  // namespace brute

void solve() {
    int n, m;
    cin >> n >> m;

    vector<array<int, 4>> edges(m);
    for (auto& [u, v, ql, qr] : edges) {
        cin >> u >> v >> ql >> qr;
        u--;
        v--;
        ql--;
        qr--;
    }

    if (m == n - 1) {
        s1::solve(n, edges);
        return;
    }
    brute::solve(n, edges);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}