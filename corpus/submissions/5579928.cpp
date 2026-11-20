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

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int narr[n];
    {
        int pref[n], suff[n];
        for (int i = 0, x = 0; i < n; i++) {
            x = max(x, arr[i]);
            pref[i] = x;
        }
        for (int i = n - 1, x = 0; i >= 0; i--) {
            x = max(x, arr[i]);
            suff[i] = x;
        }
        for (int i = 0; i < n; i++) {
            narr[i] = min(pref[i], suff[i]);
        }
    }
    auto opt = [&](int l, int r, int x) -> mint {
        int ans = 1e9;
        for (int i = l; i <= r; i++) {
            if (arr[i] > x) {
                ans = min(ans, arr[i]);
            }
        }
        return ans;
    };
    map<int, vector<int>> mp;
    for (int i = 0; i < n; i++) {
        mp[arr[i]].push_back(i);
    }
    mint ans = 0;
    set<int> csi;
    int pk = -1;
    // sum [1, n]
    auto psum_xs = [&](long n) -> mint { return n * (n + 1) / 2; };
    auto sum_xs = [&](long l, long r) -> mint {
        return psum_xs(r) - psum_xs(l - 1);
    };
    vector<pair<int, int>> prefq[n], suffq[n];
    auto process = [&](int ql, int qr) -> void {
        if (!sz(csi)) {
            return;
        }
        int cfirst = *csi.begin(), clast = *(--csi.end());
        if (sz(csi) == 1) {
            int cx = cfirst;
            ans += sum_xs(ql, qr - 1);
            prefq[cx].emplace_back(ql, qr - ql);
            suffq[cx].emplace_back(ql, qr - ql);
            return;
        }
        ans += sz(csi) * sum_xs(ql, qr - 1) +
               (sz(csi) * 2 - 3) * sum_xs(ql + 1, qr);
        prefq[cfirst].emplace_back(ql, qr - ql);
        suffq[clast].emplace_back(ql, qr - ql);
        prefq[n - 1].emplace_back(ql, qr - ql);
    };
    for (auto& [k, v] : mp) {
        if (pk != -1) {
            process(pk, k);
        }
        for (auto& a : v) {
            csi.insert(a);
        }
        auto still_in = [&](int i) -> bool { return k < narr[i]; };
        while (sz(csi) && !still_in(*csi.begin())) {
            csi.erase(csi.begin());
        }
        while (sz(csi) && !still_in(*(--csi.end()))) {
            csi.erase(--csi.end());
        }
        pk = k;
    }
    {
        set<int> cs;
        for (int i = 0; i < n; i++) {
            cs.insert(arr[i]);
            for (auto& [x, mul] : prefq[i]) {
                ans += mul * mint(*cs.upper_bound(x));
            }
        }
    }
    {
        set<int> cs;
        for (int i = n - 1; i >= 0; i--) {
            cs.insert(arr[i]);
            for (auto& [x, mul] : suffq[i]) {
                ans += mul * mint(*cs.upper_bound(x));
            }
        }
    }
    // for (int i = 0; i < n; i++) {
    //     for (auto& [x, mul] : prefq[i]) {
    //         ans += mul * opt(0, i, x);
    //     }
    //     for (auto& [x, mul] : suffq[i]) {
    //         ans += mul * opt(i, n - 1, x);
    //     }
    // }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}