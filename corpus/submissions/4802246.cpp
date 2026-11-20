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
    
    PSA(int n): psum(n + 1) {}
    
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

}

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
    if (n <= 7 && m <= 7) {
        brute::solve(n, arr);
        return;
    } else if (comp.back() <= 1) {
        st1::solve(n, arr);
        return;
    }
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