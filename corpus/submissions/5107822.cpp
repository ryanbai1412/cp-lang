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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

constexpr int maxn = 1e6 + 5;

bool vis[maxn], icyc[maxn];
vector<int> st, cyc;
vector<array<int, 3>> graph[maxn];

void dfs(int u, int p = -1) {
    dbg(u);
    if (vis[u]) {
        dbg(u, st);
        cyc = vector<int>(find(begin(st), end(st), u), st.end());
        dbg(sz(cyc));
        for (auto& a : cyc) {
            icyc[a] = true;
        }
        return;
    }
    st.push_back(u);
    vis[u] = true;
    for (auto& [v, _w, i] : graph[u]) {
        if (i == p || icyc[v]) {
            continue;
        }
        dbg(u, v);
        dfs(v, i);
    }
    st.pop_back();
}

template <int N>
long dp(int u, int p = -1) {
    vector<long> ans;
    for (auto& [v, w, i] : graph[u]) {
        if (p != i && !icyc[v]) {
            ans.push_back(w + dp<1>(v, i));
        }
    }
    if (sz(ans) < N) {
        return accumulate(begin(ans), end(ans), long(0));
    }
    partial_sort(begin(ans), ans.begin() + N, end(ans), greater<>());
    return accumulate(ans.begin(), ans.begin() + N, long(0));
}

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n; i++) {
        int u, w;
        cin >> u >> w;
        u--;
        graph[u].push_back({i, w, i});
        graph[i].push_back({u, w, i});
    }
    long ans = 0;
    bool vis_edge[n] {};
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        dbg(i);
        cyc.clear();
        dfs(i);
        vector<long> cycw;
        for (int j = 0; j < sz(cyc); j++) {
            int u = cyc[j], v = cyc[(j + 1) % sz(cyc)];
            auto [_v, w, ei] = *find_if(begin(graph[u]), end(graph[u]),
                                        [&](const array<int, 3>& a) -> bool {
                                            return a[0] == v && !vis_edge[a[2]];
                                        });
            vis_edge[ei] = true;
            cycw.push_back(w);
        }
        long cyc_sum = accumulate(begin(cycw), end(cycw), long(0)), cans = 0,
             psum = 0, w1 = -1e18, w2 = -1e18;
        for (int j = 0; j < sz(cyc); j++) {
            long cw = dp<1>(cyc[j]);
            cans = max({cans, cw + psum + w1, cw + cyc_sum - psum + w2,
                        dp<2>(cyc[j])});
            w1 = max(w1, cw - psum);
            w2 = max(w2, cw + psum);
            psum += cycw[j];
        }
        dbg(cans);
        ans += cans;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}