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

using V = vector<array<int, 3>>;

struct PSA {
    vector<int> psum;

    PSA(int n) : psum(n + 1) {}

    void update(int l, int r, int x) {
        psum[l] += x;
        psum[r + 1] -= x;
    }

    vector<int> query() const {
        vector<int> ans(sz(psum));
        partial_sum(begin(psum), end(psum), ans.begin());
        ans.pop_back();
        return ans;
    }
};

struct Bit {
    int n;
    vector<int> v;

    Bit(int n) : n(n), v(n + 1) {}

    void update(int ind, int x) {
        ind++;
        while (ind <= n) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    int query(int ind) {
        int ans = 0;
        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }
        return ans;
    }

    int query(int l, int r) {
        return query(r + 1) - query(l);
    }
};

namespace brute {

int n, ans;
vector<int> mn, carr;
V arr;

void dfs(int ind) {
    if (ind == n) {
        for (auto& [l, r, x] : arr) {
            if (*min_element(carr.begin() + l, carr.begin() + r + 1) != x) {
                return;
            }
        }
        int cans = 0;
        for (int i = 0; i < n; i++) {
            for (int j = i + 1; j < n; j++) {
                cans += carr[i] > carr[j];
            }
        }
        ans = min(ans, cans);
        return;
    }
    for (int i = mn[ind]; i < 7; i++) {
        carr.push_back(i);
        dfs(ind + 1);
        carr.pop_back();
    }
}

void solve(int _n, V _arr) {
    n = _n;
    mn = vector<int>(n, 0);
    arr = _arr;
    for (auto& [l, r, x] : arr) {
        for (int i = l; i <= r; i++) {
            mn[i] = max(mn[i], x);
        }
    }
    ans = 1e9;
    dfs(0);
    if (ans == 1e9) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

}  // namespace brute

namespace st1 {

void solve(int n, V arr) {
    PSA psa(n);
    for (auto& [l, r, x] : arr) {
        assert(0 <= x && x <= 1);
        if (x) {
            psa.update(l, r, 1);
        }
    }
    auto mn = psa.query();
    for (auto& a : mn) {
        a = !!a;
    }
    int reqs[n];
    memset(reqs, -1, sizeof(reqs));
    for (auto& [l, r, x] : arr) {
        if (!x) {
            reqs[r] = max(reqs[r], l);
        }
    }
    int dp[n + 1][n + 1][n + 2];
    memset(dp, 0x3f, sizeof(dp));
    for (auto& a : dp[n]) {
        for (int lz = 0; lz <= n + 1; lz++) {
            if (lz - 1 < reqs[n - 1]) {
                a[lz] = 1e9;
            } else {
                a[lz] = 0;
            }
        }
    }
    for (int i = n - 1; i >= 0; i--) {
        for (int ones = 0; ones < n; ones++) {
            for (int lz = 0; lz <= n + 1; lz++) {
                auto& mdp = dp[i][ones][lz];
                if (i && lz - 1 < reqs[i - 1]) {
                    mdp = 1e9;
                    continue;
                }
                mdp = dp[i + 1][ones + 1][lz];
                if (!mn[i]) {
                    mdp = min(mdp, dp[i + 1][ones][i + 1] + ones);
                }
            }
        }
    }
    dbg(dp[1][1][0], dp[2][1][2]);
    if (dp[0][0][0] >= 1e9) {
        cout << -1 << endl;
    } else {
        cout << dp[0][0][0] << endl;
    }
}

}  // namespace st1

void solve() {
    int n, m;
    cin >> n >> m;
    V arr(m);
    for (auto& [l, r, x] : arr) {
        cin >> l >> r >> x;
        l--;
        r--;
    }
    vector<int> comp;
    for (auto& a : arr) {
        comp.push_back(a[2]);
    }
    sort(begin(comp), end(comp));
    comp.erase(unique(begin(comp), end(comp)), comp.end());
    for (auto& a : arr) {
        a[2] = int(lower_bound(begin(comp), end(comp), a[2]) - comp.begin());
    }
    //    if (n <= 7 && m <= 7) {
    //        brute::solve(n, arr);
    //        return;
    //    } else if (comp.back() <= 1) {
    //        st1::solve(n, arr);
    //        return;
    //    }
    int mn[n] {}, end[n];
    bool first[n] {};
    Bit bit1(m), bit2(m);
    for (auto& [l, r, x] : arr) {
        first[l] = true;
        end[l] = r;
        for (int i = l; i <= r; i++) {
            mn[i] = x;
        }
    }
    int base_inv = 0;
    {
        Bit bit(m);
        for (int i = 0; i < n; i++) {
            if (first[i]) {
                base_inv += bit.query(mn[i] + 1, m - 1);
                bit.update(mn[i], 1);
            }
        }
    }
    for (auto& a : mn) {
        bit1.update(a, 1);
    }
    int dp[n + 1][m];
    memset(dp, 0x3f, sizeof(dp));
    memset(dp[n], 0, sizeof(dp[n]));
    for (int i = n - 1; i >= 0; i--) {
        bit1.update(mn[i], -1);
        if (first[i]) {
            int last = end[i], mul = last - i;
            auto invs = [&](int x) -> int {
                return bit1.query(x + 1, m - 1) + bit2.query(x);
            };
            dbg(i, invs(2));
            int opt = 1e9;
            for (int prv = m - 1; prv >= 0; prv--) {
                dp[i][prv] = opt = min(opt, dp[last + 1][prv] + mul * invs(max(prv, mn[i])));
            }
            bit2.update(mn[i], 1);
            continue;
        }
        auto invs = [&](int x) -> int {
            return bit1.query(x + 1, m - 1) + bit2.query(x);
        };
        for (int prv = 0; prv < mn[i]; prv++) {
            dp[i][prv] = dp[i + 1][prv] + invs(prv);
        }
        int opt = 1e9;
        for (int prv = m - 1; prv >= mn[i]; prv--) {
            dp[i][prv] = opt = min(opt, dp[i + 1][prv] + invs(prv));
        }
    }
    dbg(dp[1][0], dp[1][2], dp[3][0]);
    dbg(base_inv);
    cout << dp[0][0] + base_inv << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}