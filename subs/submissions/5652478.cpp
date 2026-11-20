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

constexpr int maxn = 5e5 + 5;

struct Lazy {
    mint mult, add;

    Lazy& operator+=(const Lazy& l) {
        add = add * l.mult + l.add;
        mult *= l.mult;
        return *this;
    }

    friend Lazy operator+(Lazy a, const Lazy& b) {
        return a += b;
    }

    void mult_i(mint x) {
        mult *= x;
        add *= x;
    }

    mint operator()(mint x) const {
        return x * mult + add;
    }

    bool is_def() const {
        return mult.x == 1 && add.x == 0;
    }

    static Lazy c_def() {
        return {1, 0};
    }
};

struct Node {
    Lazy lazy;
    int lc, rc;

    bool leaf() const {
        return !lc && !rc;
    }
} heap[20 << 20];

int n, hind = 1;

int alloc() {
    heap[hind].lazy = Lazy::c_def();
    return hind++;
}

mint get(int o, int l, int r, int ind, Lazy clazy) {
    if (!o) {
        return clazy(0);
    }
    clazy = heap[o].lazy + clazy;
    int mid = (l + r) / 2;
    if (ind <= mid) {
        return get(heap[o].lc, l, mid, ind, clazy);
    } else {
        return get(heap[o].rc, mid + 1, r, ind, clazy);
    }
}

mint get(int o, int ind) {
    return get(o, 0, n, ind, Lazy::c_def());
}

void pushdown(int o) {
    if (heap[o].lazy.is_def()) {
        // return;
    }

    auto go_c = [&](int& c) -> void {
        if (!c) {
            c = alloc();
        }
        heap[c].lazy += heap[o].lazy;
    };

    go_c(heap[o].lc);
    go_c(heap[o].rc);

    heap[o].lazy = Lazy::c_def();
}

int mult(int o1, int o2) {
    if (heap[o2].leaf()) {
        swap(o1, o2);
    }
    if (heap[o1].leaf()) {
        heap[o2].lazy.mult_i(heap[o1].lazy(0));
        return o2;
    }

    pushdown(o1);
    pushdown(o2);

    heap[o1].lc = mult(heap[o1].lc, heap[o2].lc);
    heap[o1].rc = mult(heap[o1].rc, heap[o2].rc);
    return o1;
}

void clear(int& o, int l, int r, int ind) {
    if (!o) {
        return;
    } else if (r <= ind) {
        o = 0;
        return;
    }
    pushdown(o);
    int mid = (l + r) / 2;
    clear(heap[o].lc, l, mid, ind);
    if (mid < ind) {
        clear(heap[o].rc, mid + 1, r, ind);
    }
}

int clear(int o, int ind) {
    if (ind < 0) {
        return o;
    }
    clear(o, 0, n, ind);
    return o;
}

void add(int o, mint x) {
    heap[o].lazy.add += x;
}

int arr[maxn], depth[maxn];
vector<int> graph[maxn];

void dfs(int u, int d) {
    depth[u] = d;
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        dfs(v, d + 1);
    }
}

int dp(int u) {
    mint cadd = 1;
    int ans = alloc();
    add(ans, 1);

    for (auto& v : graph[u]) {
        auto cans = dp(v);
        cadd *= get(cans, depth[u]);
        ans = mult(ans, cans);
    }

    clear(ans, arr[u]);
    add(ans, cadd);

    return ans;
}

// vector<mint> mult(vector<mint> a, const vector<mint>& b) {
//     for (int i = 0; i < sz(a); i++) {
//         a[i] *= b[i];
//     }
//     return a;
// }
//
// vector<mint> add(vector<mint> a, mint x) {
//     for (auto& b : a) {
//         b += x;
//     }
//     return a;
// }
//
// vector<mint> cleared(vector<mint> a, int ind) {
//     fill(a.begin(), a.begin() + ind + 1, 0);
//     return a;
// }
//
// vector<mint> dp(int u) {
//     mint cadd = 1;
//     vector<mint> ans(n, 1);
//
//     for (auto& v : graph[u]) {
//         auto cans = dp(v);
//         cadd *= cans[depth[u]];
//         ans = mult(ans, cans);
//     }
//
//     ans = cleared(ans, arr[u]);
//
//     ans = add(ans, cadd);
//
//     return ans;
// }

void solve() {
    dbg(sizeof(heap) / 1e6);
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

    mint ans = 1;
    for (auto& v : graph[0]) {
        ans *= get(dp(v), 0);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}