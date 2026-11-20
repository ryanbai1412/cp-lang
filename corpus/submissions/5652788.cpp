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

struct M {
    static constexpr int maxn = 1e7 + 5;

    mint fact[maxn];

    M() {
        fact[0] = 1;
        for (int i = 1; i < maxn; i++) {
            fact[i] = i * fact[i - 1];
        }
    }

    mint choose(int n, int k) const {
        return fact[n] / fact[k] / fact[n - k];
    }

    mint choose2(int n, int k) const {
        if (n < 0 || k < 0 || k > n) {
            return 0;
        }
        return choose(n, k);
    }
} M;

struct Quadratic {
    mint a, b, c;

    mint operator()(mint x) const {
        return a * x * x + b * x + c;
    }
};

Quadratic interpolate(mint y0, mint y1, mint y2) {
    mint c = y0;
    mint a = (y2 - c - 2 * (y1 - c)) / 2;
    mint b = y1 - c - a;
    return {a, b, c};
}

void solve() {
    int n, m, type;
    cin >> n >> m >> type;

    Quadratic f;
    if (type == 1) {
        f = {0, 1, 0};
    } else {
        f = {1, 0, 0};
    }

    for (int iter = 0; iter < m; iter++) {
        int kv;
        cin >> kv;
        array<mint, 3> out {};

        for (int i = 1; i <= 3; i++) {
            for (int mask = 0; mask < (1 << i); mask++) {
                int cnt[2] {kv, n - kv};
                mint prob = 1;
                for (int j = 0; j < i; j++) {
                    prob *= cnt[on(mask, j)]-- / mint(n - j);
                }

                int pcnt = __builtin_popcount(mask);
                if (on(mask, i - 1)) {
                    out[i - 1] += prob * f(kv + pcnt - 1);
                } else {
                    out[i - 1] += prob * f(i - pcnt - 1);
                }
            }
        }

        f = interpolate(out[0], out[1], out[2]);
    }

    int q;
    cin >> q;
    while (q--) {
        int ind;
        cin >> ind;
        ind--;
        cout << f(ind) + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}