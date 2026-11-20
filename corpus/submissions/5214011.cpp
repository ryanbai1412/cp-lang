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

template <int MOD>
struct Mint {
    static constexpr int norm(int x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return x;
    }

    static constexpr Mint pow(Mint base, long exp) {
        Mint ans = 1;
        while (exp) {
            if (exp & 1) {
                ans *= base;
            }
            base *= base;
            exp >>= 1;
        }
        return ans;
    }

    static constexpr Mint inv(Mint x) {
        return pow(x, MOD - 2);
    }

    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(int x) : x(norm(x)) {}
    constexpr Mint(int x, bool) : x(x) {}

    friend istream& operator>>(istream& in, Mint& m) {
        int x;
        cin >> x;
        m = x;
        return in;
    }
    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    friend Mint operator-(const Mint& m) {
        if (!m.x) {
            return m;
        } else {
            return Mint(MOD - m.x, true);
        }
    }

    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    Mint& operator-=(const Mint& m) {
        x -= m.x;
        if (x < 0) {
            x += MOD;
        }
        return *this;
    }
    Mint& operator*=(const Mint& m) {
        x = (long(x) * m.x) % MOD;
        return *this;
    }
    Mint& operator/=(const Mint& m) {
        return *this *= Mint::inv(m);
    }

    friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }
    friend Mint operator/(Mint a, const Mint& b) {
        return a /= b;
    }
};

using mint = Mint<int(1e9 + 7)>;

struct M {
    static constexpr int maxn = 2e6 + 5;

    mint pow2[maxn + 1];

    M() {
        pow2[0] = 1;
        for (int i = 0; i < maxn; i++) {
            pow2[i + 1] = pow2[i] * 2;
        }
    }
} M;

constexpr int maxn = 5e5 + 5;

struct DS1 : public set<pair<int, int>> {
    using set<pair<int, int>>::set;

    void insert(int u, int v) {
        if (u > v) {
            std::swap(u, v);
        }
        emplace(u, v);
    }

    bool contains(int u, int v) {
        if (u > v) {
            std::swap(u, v);
        }
        return count({u, v});
    }
};

bool vis[maxn];
int comp[maxn], depth[maxn], tlow[maxn];
vector<int> graph[maxn];
DS1 bridges;

void dfs(int u, int p, int d) {
    tlow[u] = depth[u] = d;
    vis[u] = true;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        } else if (vis[v]) {
            tlow[u] = min(tlow[u], depth[v]);
            continue;
        }
        dfs(v, u, d + 1);
        if (tlow[v] > d) {
            dbg("BRIDGE", u, v);
            bridges.insert(u, v);
        }
        tlow[u] = min(tlow[u], tlow[v]);
    }
}

void dfs2(int u, int cc) {
    if (comp[u] != -1) {
        return;
    }
    comp[u] = cc;
    for (auto& v : graph[u]) {
        if (bridges.contains(u, v)) {
            continue;
        }
        dfs2(v, cc);
    }
}

struct Solver {
    static constexpr mint NO_MEMO = {-1, true};

    int n;
    vector<int> arr;
    vector<mint> memo;
    vector<vector<int>> graph;

    Solver(const vector<int>& arr)
        : n(sz(arr)), arr(arr), memo(n, NO_MEMO), graph(n) {}

    void add_edge(int u, int v) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    void pdfs(int u) {
        for (auto& v : graph[u]) {
            graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
            pdfs(v);
        }
    }

    mint dp(int u) {
        mint& ans = memo[u];
        if (ans.x != -1) {
            return ans;
        }
        ans = M.pow2[arr[u]];
        for (auto& v : graph[u]) {
            ans *= 1 + dp(v) / 2;
        }
        ans -= 1;
        return ans;
    }

    mint dp1(int u) {
        mint ans = 0;
        {
            mint cans = M.pow2[arr[u]] - 1;
            for (auto& v : graph[u]) {
                cans *= 1 + dp(v) / 2;
            }
            ans += cans;
        }
        {
            mint cans = 1;
            for (auto& v : graph[u]) {
                cans *= 1 + dp(v) / 2;
            }
            cans -= 1;
            for (auto& v : graph[u]) {
                cans -= dp(v) / 2;
            }
            ans += cans;
        }
        return ans;
    }

    mint solve() {
        pdfs(0);
        mint ans = 0;
        for (int i = 0; i < n; i++) {
            ans += dp1(i);
        }
        return ans;
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    dfs(0, -1, 0);
    memset(comp, -1, sizeof(comp));
    int cind = 0;
    for (int i = 0; i < n; i++) {
        if (comp[i] != -1) {
            continue;
        }
        dfs2(i, cind++);
    }
    vector<int> csz(cind);
    for (int i = 0; i < n; i++) {
        csz[comp[i]]++;
    }
    for (auto& a : csz) {
        dbg(a);
    }
    Solver solver(csz);
    for (auto& [u, v] : bridges) {
        solver.add_edge(comp[u], comp[v]);
    }
    cout << solver.solve() * M.pow2[m] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}