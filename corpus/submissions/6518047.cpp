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

struct mint {
    static constexpr int MOD = 998244353;

    static constexpr int norm(long x) {
        if (0 <= x && x < MOD) {
            return int(x);
        } else if (-MOD <= x && x < 0) {
            return int(x + MOD);
        }
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    static constexpr mint pow(mint base, long exp) {
        mint ans = 1;

        while (exp) {
            if (exp & 1) {
                ans *= base;
            }
            base *= base;
            exp >>= 1;
        }

        return ans;
    }

    static constexpr mint inv(mint x) {
        return mint::pow(x, MOD - 2);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    static constexpr mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    constexpr mint operator-() const {
        if (!x) {
            return {};
        }
        return mint::from_unchecked(MOD - x);
    }

    constexpr mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr mint& operator-=(const mint& m) {
        return *this += -m;
    }

    constexpr mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }
    constexpr mint& operator/=(const mint& m) {
        return *this *= mint::inv(m);
    }

    friend constexpr mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend constexpr mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend constexpr mint operator*(mint a, const mint& b) {
        return a *= b;
    }
    friend constexpr mint operator/(mint a, const mint& b) {
        return a /= b;
    }
};

namespace s1 {

int count_hamil(const vector<vector<bool>>& graph) {
    int n = sz(graph);

    int dp[1 << n][n] {};
    for (int i = 0; i < n; i++) {
        dp[(1 << n) - 1][i] = graph[i][0];
    }

    for (int mask = (1 << n) - 2; mask >= 0; mask--) {
        for (int u = 0; u < n; u++) {
            for (int v = 0; v < n; v++) {
                if (on(mask, u) && !on(mask, v) && graph[u][v]) {
                    dp[mask][u] += dp[mask | (1 << v)][v];
                }
            }
        }
    }

    return dp[1][0] / 2;
}

bool solve(const vector<pair<int, int>>& edges, int kv) {
    int n = sz(edges) + 1;

    vector<vector<int>> graph(n);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
    }

    vector<int> leaves;
    {
        vector<int> q;
        q.push_back(0);

        while (sz(q)) {
            int u = q.back();
            q.pop_back();

            if (!sz(graph[u])) {
                leaves.push_back(u);
            } else {
                for (auto& v : graph[u]) {
                    q.push_back(v);
                }
            }
        }
    }

    vector ngraph(n, vector(n, false));
    for (auto& [u, v] : edges) {
        ngraph[u][v] = ngraph[v][u] = true;
    }
    int m = sz(leaves);
    for (int i = 0; i < m; i++) {
        for (int j = 1; j <= kv; j++) {
            int u = leaves[i], v = leaves[(i + j) % m];
            ngraph[u][v] = ngraph[v][u] = true;
        }
    }

    cout << count_hamil(ngraph) << endl;
    return true;
}

}  // namespace s1

namespace sk1 {

constexpr int MAXN = 1e3 + 100;

vector<int> graph[MAXN];
bool is_chain[MAXN];
mint memo[MAXN][3];

void dfs(int u) {
    for (auto& v : graph[u]) {
        dfs(v);
    }

    if (!sz(graph[u])) {
        is_chain[u] = true;
    } else if (sz(graph[u]) == 1) {
        is_chain[u] = is_chain[graph[u][0]];
    }
}

mint dp(int u, int ty) {
    if (!sz(graph[u])) {
        return 1;
    }

    mint& ans = memo[u][ty + 1];
    if (ans.x != -1) {
        return ans;
    }

    int m = sz(graph[u]);

    if (ty == -1) {
        ans = dp(graph[u][0], -1);
        for (int i = 1; i < m; i++) {
            ans *= dp(graph[u][i], 0);
        }
    } else if (ty == +1) {
        ans = dp(graph[u].back(), +1);
        for (int i = 0; i < m - 1; i++) {
            ans *= dp(graph[u][i], 0);
        }
    } else {
        ans = 0;

        mint pref[m + 1], suff[m + 1];
        pref[0] = suff[m] = 1;
        for (int i = 0; i < m; i++) {
            pref[i + 1] = pref[i] * dp(graph[u][i], 0);
        }
        for (int i = m - 1; i >= 0; i--) {
            suff[i] = suff[i + 1] * dp(graph[u][i], 0);
        }

        for (int i = 0; i + 1 < m; i++) {
            ans += pref[i] * suff[i + 2] * dp(graph[u][i], +1) *
                   dp(graph[u][i + 1], -1);
        }
    }

    return ans;
}

bool solve(const vector<pair<int, int>>& edges, int kv) {
    if (kv != 1) {
        return false;
    }

    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
    }

    dfs(0);

    int m = sz(graph[0]);
    if (m == 1) {
        cout << 0 << endl;
        return true;
    } else if (m == 2 && is_chain[graph[0][0]] && is_chain[graph[0][1]]) {
        cout << 1 << endl;
        return true;
    }

    memset(memo, -1, sizeof(memo));
    mint ans = dp(0, 0);

    dbg(dp(1, +1));
    {
        mint cmul = 1;
        for (int i = 1; i + 1 < m; i++) {
            cmul *= dp(graph[0][i], 0);
        }
        ans += cmul * dp(graph[0].back(), +1) * dp(graph[0][0], -1);
    }

    cout << ans << endl;

    return true;
}

}  // namespace sk1

void solve() {
    int n, kv;
    cin >> n >> kv;

    vector<pair<int, int>> edges;
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
        p--;
        edges.emplace_back(p, i);
    }

    sk1::solve(edges, kv) || s1::solve(edges, kv);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}