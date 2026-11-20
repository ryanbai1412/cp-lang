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

template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
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

    static constexpr int inv(int x) {
        int y = MOD, vx = 1, vy = 0;
        while (x) {
            int k = y / x;
            y %= x;
            vy -= k * vx;
            swap(x, y);
            swap(vx, vy);
        }
        assert(y == 1);
        if (vy < 0) {
            vy += MOD;
        }
        return vy;
    }

    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(long x) : x(norm(x)) {}

    static constexpr Mint from_unchecked(int x) {
        Mint m;
        m.x = x;
        return m;
    }

    friend istream& operator>>(istream& in, Mint& m) {
        long x;
        in >> x;
        m = x;
        return in;
    }
    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    explicit operator int() const {
        return x;
    }
    explicit operator long() const {
        return x;
    }

    bool operator==(const Mint& m) const {
        return x == m.x;
    }
    bool operator!=(const Mint& m) const {
        return x != m.x;
    }

    Mint operator-() const {
        return !x ? 0 : Mint::from_unchecked(MOD - x);
    }
    Mint operator+() const {
        return *this;
    }

    Mint inv() const {
        return Mint::from_unchecked(Mint::inv(x));
    }

    Mint& operator++() {
        x++;
        if (x == MOD) {
            x = 0;
        }
        return *this;
    }
    Mint& operator--() {
        if (!x) {
            x = MOD - 1;
        } else {
            x--;
        }
        return *this;
    }

    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    Mint& operator-=(const Mint& m) {
        return *this += -m;
    }

    Mint& operator*=(const Mint& m) {
        x = int((long(x) * m.x) % MOD);
        return *this;
    }
    Mint& operator/=(const Mint& m) {
        return *this *= m.inv();
    }

    friend Mint operator++(Mint& a, int) {
        Mint ans = a;
        ++a;
        return ans;
    }
    friend Mint operator--(Mint& a, int) {
        Mint ans = a;
        --a;
        return ans;
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

using mint = Mint<998244353>;

constexpr int maxn = 2e3 + 5;

int n, arr[maxn], depth[maxn];
vector<int> graph[maxn];
mint memo[maxn][maxn];

void dfs(int u, int d) {
    depth[u] = d;
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        dfs(v, d + 1);
    }
}

mint dp(int u, int last) {
    mint& ans = memo[u][last];
    if (ans.x != -1) {
        return ans;
    }

    ans = 0;

    {
        mint cans = 1;
        for (auto& v : graph[u]) {
            cans *= dp(v, depth[u]);
        }
        ans += cans;
    }

    if (arr[u] < last) {
        mint cans = 1;
        for (auto& v : graph[u]) {
            cans *= dp(v, last);
        }
        ans += cans;
    }

    return ans;
}

void solve() {
    cin >> n;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    int m;
    cin >> m;
    int paths[m][2];
    for (auto& [u, v] : paths) {
        cin >> u >> v;
        u--;
        v--;
    }

    dfs(0, 0);
    memset(arr, -1, sizeof(arr));
    for (auto& [u, v] : paths) {
        assert(depth[u] < depth[v]);
        if (depth[u] > depth[v]) {
            swap(u, v);
        }
        arr[v] = max(arr[v], depth[u]);
    }

    memset(memo, -1, sizeof(memo));

    mint ans = 1;
    for (auto& v : graph[0]) {
        ans *= dp(v, 0);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}