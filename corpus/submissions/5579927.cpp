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
    dbgh(__VA_ARGS__)
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
    static constexpr int MOD = 1e6 + 3;

    static constexpr int norm(long x) {
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
        assert(x.x);
        return pow(x, MOD - 2);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

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
    mint& operator/=(const mint& m) {
        return *this *= mint::inv(m);
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

optional<vector<int>> gen_vals(const vector<int>& arr) {
    int n = sz(arr);
    vector<int> ans(n + 1, -1);
    bool found = false;
    for (int i = 0; i < n; i++) {
        if (!arr[i]) {
            continue;
        }
        found = true;
        assert(arr[i] == -1 || arr[i] == 1);
        if (arr[i] == -1) {
            ans[i] = 1;
        } else {
            ans[i] = 0;
        }
        break;
    }
    if (!found) {
        return {};
    }
    for (int i = 0; i < n; i++) {
        if (ans[i] != -1) {
            ans[i + 1] = ans[i] + arr[i];
        }
    }
    for (int i = n - 1; i >= 0; i--) {
        if (ans[i + 1] != -1) {
            ans[i] = ans[i + 1] - arr[i];
        }
    }
    return ans;
}

struct M {
    static constexpr int maxn = 1e6 + 3;

    mint fact[maxn], ifact[maxn];

    M() {
        fact[0] = ifact[0] = 1;
        for (int i = 1; i < maxn; i++) {
            fact[i] = fact[i - 1] * i;
            ifact[i] = mint::inv(fact[i]);
        }
    }

    mint choose(int n, int k) {
        if (n < 0 || k < 0 || k > n) {
            return 0;
        }
        return fact[n] * ifact[k] * ifact[n - k];
    }
} M;

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n - m + 1];
    for (auto& a : arr) {
        cin >> a;
    }
    vector<int> cvals;
    for (int i = 0; i < m; i++) {
        vector<int> cdiff;
        for (int j = i; j + m < n; j += m) {
            cdiff.push_back(arr[j + 1] - arr[j]);
        }
        auto cv = gen_vals(cdiff);
        if (!cv) {
            cvals.push_back(-1);
        } else {
            cvals.push_back(cv.value()[0]);
        }
    }
    int csum = 0, open = 0;
    for (auto& a : cvals) {
        if (a == -1) {
            open++;
        } else {
            csum += a;
        }
    }
    cout << M.choose(open, arr[0] - csum) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}