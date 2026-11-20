#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct mint {
    static constexpr int MOD = 998244353;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    static mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint::from_unchecked(MOD - x);
    }

    mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    mint& operator-=(const mint& m) {
        return *this += -m;
    }

    mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

namespace s1 {

constexpr int maxn = 20;

int mask, dfs_cnt;
bool vis[maxn];
vector<int> graph[maxn];

void dfs(int u) {
    if (vis[u]) {
        return;
    }
    vis[u] = true;
    dfs_cnt++;
    for (auto& v : graph[u]) {
        if (on(mask, u) && on(mask, v)) {
            continue;
        }
        dfs(v);
    }
}

bool solve(int n, int kv, const vector<pair<int, int>>& edges) {
    if (n > 20) {
        return false;
    }
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    int ans = 0;
    for (mask = 1; mask < (1 << n); mask++) {
        memset(vis, 0, sizeof(vis));
        bool b = false;
        for (auto& [u, v] : edges) {
            b |= on(mask, u) && on(mask, v);
        }
        if (!b) {
            continue;
        }
        vector<int> cnts;
        for (int i = 0; i < n; i++) {
            if (!on(mask, i)) {
                continue;
            }
            b &= !vis[i];
            dfs_cnt = 0;
            dfs(i);
            cnts.push_back(dfs_cnt);
        }
        if (!b) {
            continue;
        }
        sort(begin(cnts), end(cnts));
        cnts.erase(unique(begin(cnts), end(cnts)), cnts.end());
        if (sz(cnts) > kv + 1 ||
            (kv == 1 && sz(cnts) == 2 && cnts[1] - cnts[0] > 1)) {
            continue;
        }
        ans++;
    }
    cout << ans << endl;
    return true;
}
}  // namespace s1

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    vector<pair<int, int>> edges(m);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }
    s1::solve(n, kv, edges);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}