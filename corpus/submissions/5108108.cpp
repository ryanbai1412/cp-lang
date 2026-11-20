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

    static constexpr Mint from_unsafe(int x) {
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
        return !x ? 0 : Mint::from_unsafe(MOD - x);
    }
    Mint operator+() const {
        return *this;
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
        x -= m.x;
        if (x < 0) {
            x += MOD;
        }
        return *this;
    }

    Mint& operator*=(const Mint& m) {
        x = int((long(x) * m.x) % MOD);
        return *this;
    }
    Mint& operator/=(const Mint& m) {
        return *this *= m.inv();
    }
    Mint inv() const {
        return Mint::from_unsafe(Mint::inv(x));
    }

    friend Mint operator++(Mint& a, int) {
        return ++a;
    }
    friend Mint operator--(Mint& a, int) {
        return --a;
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

using mint = Mint<1024523>;

struct M {
    static constexpr int maxn = 1e6 + 5;

    mint fact[maxn], ifact[maxn];

    M() {
        fact[0] = ifact[0] = 1;
        for (int i = 1; i < maxn; i++) {
            fact[i] = fact[i - 1] * i;
            ifact[i] = 1 / fact[i];
        }
    }

    mint choose(int n, int k) const {
        return fact[n] * ifact[k] * ifact[n - k];
    }
} M;

constexpr int maxn = 505;

int n, m;
string s1, s2;
mint memo[maxn][maxn][maxn];

mint dp(int i1, int j1, int i2, int j2) {
    if (i1 == n && j1 == m) {
        return 0;
    }
    mint& ans = memo[i1][j1][i2];
    if (ans.x != -1) {
        return ans;
    }
    ans = 0;
    auto ways = [&](int i, int j) -> mint {
        i = n - i;
        j = m - j;
        return M.choose(i + j, i);
    };
    auto go = [&](char c1, char c2, int i1, int j1, int i2, int j2) -> void {
        if (c1 == c2) {
            ans += dp(i1, j1, i2, j2);
        } else {
            ans += ways(i1, j1) * ways(i2, j2);
        }
    };
    if (i1 < n) {
        if (i2 < n) {
            go(s1[i1], s1[i2], i1 + 1, j1, i2 + 1, j2);
        }
        if (j2 < m) {
            go(s1[i1], s2[j2], i1 + 1, j1, i2, j2 + 1);
        }
    }
    if (j1 < m) {
        if (i2 < n) {
            go(s2[j1], s1[i2], i1, j1 + 1, i2 + 1, j2);
        }
        if (j2 < m) {
            go(s2[j1], s2[j2], i1, j1 + 1, i2, j2 + 1);
        }
    }
    return ans;
}

void solve() {
    cin >> n >> m >> s1 >> s2;
    memset(memo, -1, sizeof(memo));
    cout << M.choose(n + m, n) * M.choose(n + m, n) - dp(0, 0, 0, 0)
         << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}