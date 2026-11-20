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

constexpr int maxn = 2e5 + 5;

long clim;
vector<int> ans;
vector<pair<int, int>> graph[maxn];

pair<long, long> dfs(int u, int p = -1) {
    long mxbad = 0, mngood = 1e18;
    for (auto& [v, w] : graph[u]) {
        if (v == p) {
            continue;
        }
        auto [nbad, ngood] = dfs(v, u);
        mngood = min(mngood, ngood + w);
        if (nbad == -1) {
            continue;
        }
        if (nbad + w > clim) {
            ans.push_back(v);
            mngood = min(mngood, long(w));
        } else {
            mxbad = max(mxbad, w + nbad);
        }
    }
    if (mxbad <= clim - mngood) {
        return {-1, mngood};
    }
    return {mxbad, mngood};
}

void solve() {
    int n, kv;
    cin >> n >> kv;
    for (int i = 0; i < n - 1; i++) {
        int u, v, w = 1;
        cin >> u >> v;
        u--;
        v--;
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    auto check = [&](long x) -> bool {
        ans.clear();
        clim = x;
        if (dfs(0).first != -1) {
            ans.push_back(0);
        }
        return sz(ans) <= kv;
    };
    long l = 0, r = 1e18;
    while (l < r) {
        long mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    check(l);
    cout << l << endl;
//    dbg(sz(ans));
//    bool vis[n] {};
//    for (auto& a : ans) {
//        vis[a] = true;
//    }
//    for (int i = 0; i < n && sz(ans) < kv; i++) {
//        if (!vis[i]) {
//            ans.push_back(i);
//        }
//    }
//    for (int i = 0; i < kv; i++) {
//        cout << ans[i] + 1 << " \n"[i == kv - 1];
//    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}