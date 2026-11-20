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

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            return int(x + MOD);
        }
        return int(x);
    }

    int x;

    constexpr Mint(long x = 0) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
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
        x = int(long(x) * m.x % MOD);
        return *this;
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

    bool operator==(const Mint& m) const {
        return x == m.x;
    }
    bool operator!=(const Mint& m) const {
        return x != m.x;
    }
};

template <int base>
struct H {
    using mint = Mint<int(1e9 + 7)>;

    static constexpr mint BASE = base;

    vector<mint> psum, pow;

    H(const string& s) : psum {0}, pow {1} {
        for (auto& c : s) {
            psum.push_back(psum.back() * BASE + (c - 'a' + 1));
            pow.push_back(pow.back() * BASE);
        }
    }

    mint query(int l, int r) const {
        return psum[r] - psum[l] * pow[r - l];
    }
};

struct DS {
    int ans = 0, cnt[26] {};

    void update(char cc) {
        int c = cc - 'a';
        ans -= cnt[c] & 1;
        cnt[c]++;
        ans += cnt[c] & 1;
    }
};

struct Bit {
    static constexpr int N = 30;

    int v[N] {};

    void update(int ind, int x) {
        ind++;
        while (ind <= N) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    int psum(int ind) const {
        int ans = 0;
        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }
        return ans;
    }
};

void solve() {
    string s;
    cin >> s;
    int n = sz(s);
    H<31> h(s);
    vector<int> arrab[n];
    for (int i = 1; i <= n; i++) {
        for (int j = i; j < n; j += i) {
            if (j >= 2 * i) {
                if (h.query(j - i, j) != h.query(j - i - i, j - i)) {
                    break;
                }
            }
            arrab[j].push_back(i);
        }
    }
    dbg(h.query(0, 2), h.query(2, 4));
    vector<int> queries[n + 1];
    {
        DS ds;
        for (int i = n - 1; i >= 0; i--) {
            ds.update(s[i]);
            for (auto& a : arrab[i]) {
                queries[a].push_back(ds.ans);
            }
        }
    }
    long ans = 0;
    DS ds;
    Bit cq;
    for (int i = 1; i <= n; i++) {
        for (auto& a : queries[i]) {
            ans += cq.psum(a + 1);
        }
        ds.update(s[i - 1]);
        cq.update(ds.ans, +1);
        dbg(ans);
    }
    cout << ans << endl;
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