#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(T&& t, U&&... u) {
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

int solve_bf(const vector<pair<int, int>>& arr, int ql, int qr) {
    if (ql > qr) {
        return -1e9;
    }
    assert(is_sorted(begin(arr), end(arr)));
    int n = sz(arr), dp[n + 1] {}, ans = 0;
    for (int i = n - 1, j = n - 1; i >= 0; i--) {
        while (arr[j].first >= arr[i].second) {
            j--;
        }
        dp[i] = dp[i + 1];
        if (arr[i].second <= qr) {
            dp[i] = max(dp[i], 1 + dp[j + 1]);
        }
        if (arr[i].first >= ql) {
            ans = max(ans, dp[i]);
        }
    }
    return ans;
}

struct DS1 {
    vector<pair<int, int>> arr;

    DS1(vector<pair<int, int>> arr) : arr(std::move(arr)) {
        assert(is_sorted(begin(arr), end(arr)));
    }

    int query(int l, int r, int ql, int qr) const {
        return solve_bf({arr.begin() + l, arr.begin() + r}, ql, qr);
    }
};

void solve() {
    int n;
    cin >> n;
    array<int, 3> arr[n];
    for (int i = 0; i < n; i++) {
        auto& [l, r, qi] = arr[i];
        cin >> l >> r;
        qi = i;
        l--;
    }
    sort(arr, arr + n);
    auto ds = DS1(({
        vector<pair<int, int>> v;
        for (auto& [l, r, _i] : arr) {
            v.emplace_back(l, r);
        }
        v;
    }));
    int iarr[n];
    for (int i = 0; i < n; i++) {
        iarr[arr[i][2]] = i;
    }
    map<array<int, 4>, int> expec;
    expec[{0, n, -1, int(1e9)}] = ds.query(0, n, -1, 1e9);
    vector<int> ans;
    for (auto& a : iarr) {
        auto it = expec.lower_bound({a + 1, -1});
        assert(it != expec.begin());
        --it;
        auto [pos, v] = *it;
        auto [cl, cr, ql, qr] = pos;
        int ml = arr[a][0], mr = arr[a][1];
        int vl = ds.query(cl, a, ql, ml), vr = ds.query(a + 1, cr, mr, qr);
        dbg(vl, vr, v);
        assert(vl + 1 + vr <= v);
        if (vl + 1 + vr == v) {
            ans.push_back(arr[a][2]);
            expec.erase(it);
            expec[{cl, a, ql, ml}] = vl;
            expec[{a + 1, cr, mr, qr}] = vr;
        }
    }
    cout << sz(ans) << endl;
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] + 1 << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}