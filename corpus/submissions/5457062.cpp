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

struct mint {
    static constexpr int MOD = 20170408;

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
    constexpr mint(int x, bool) : x(x) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint(MOD - x, true);
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

vector<vector<mint>> mul(const vector<vector<mint>>& arra,
                         const vector<vector<mint>>& arrb) {
    int a = sz(arra), b = sz(arra[0]), c = sz(arrb[0]);
    assert(b == sz(arrb));
    vector<vector<mint>> ans(a, vector<mint>(c));
    for (int i = 0; i < a; i++) {
        for (int j = 0; j < b; j++) {
            for (int k = 0; k < c; k++) {
                ans[i][k] += arra[i][j] * arrb[j][k];
            }
        }
    }
    return ans;
}

vector<vector<mint>> bpow(vector<vector<mint>> base, long exp) {
    int n = sz(base);
    vector<vector<mint>> ans(n, vector<mint>(n));
    for (int i = 0; i < n; i++) {
        ans[i][i] = 1;
    }
    while (exp) {
        if (exp & 1) {
            ans = mul(ans, base);
        }
        base = mul(base, base);
        exp >>= 1;
    }
    return ans;
}

mint solve(int n, const vector<int>& arr) {
    int m = sz(arr);
    vector<vector<mint>> mat(m, vector<mint>(m));
    for (int i = 0; i < m; i++) {
        for (int j = 0; j < m; j++) {
            mat[i][(i + j) % m] = arr[j];
        }
    }
    return bpow(mat, n)[0][0];
}

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    bool prime[m + 1];
    memset(prime, 1, sizeof(prime));
    prime[0] = prime[1] = false;
    for (int i = 2; i <= m; i++) {
        if (!prime[i] || long(i) * i > m) {
            continue;
        }
        for (int j = i * i; j <= m; j += i) {
            prime[j] = false;
        }
    }
    vector<int> occa(kv), occp(kv);
    for (int i = 1; i <= m; i++) {
        occa[i % kv]++;
        if (!prime[i]) {
            occp[i % kv]++;
        }
    }
    cout << solve(n, occa) - solve(n, occp) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}