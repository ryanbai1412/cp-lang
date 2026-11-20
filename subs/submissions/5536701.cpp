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

template <typename T>
struct ST {
    int n;
    vector<vector<T>> v;

    ST(const vector<T>& arr) : n(sz(arr)), v(__lg(n) + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= __lg(n); i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = min(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    T query(int l, int r) const {
        int ind = __lg(r - l);
        return min(v[ind][l], v[ind][r - (1 << ind)]);
    }
};

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
    int n;
    vector<pair<int, int>> arr;
    ST<pair<int, int>> st;
    vector<vector<int>> lift;

    pair<int, int> query_st(int l, int r) const {
        if (l >= r) {
            return {int(2e9), -1};
        }
        return st.query(l, r);
    }

    int first_after(int x) const {
        return int(lower_bound(begin(arr), end(arr), pair<int, int> {x, -1}) -
                   arr.begin());
    }

    DS1(const vector<pair<int, int>>& arr)
        : n(sz(arr)),
          arr(arr),
          st(({
              vector<pair<int, int>> v;
              for (int i = 0; i < n; i++) {
                  v.emplace_back(arr[i].second, i);
              }
              v;
          })),
          lift(__lg(n) + 1, vector<int>(n)) {
        assert(is_sorted(begin(arr), end(arr)));
        for (int i = 0; i < n; i++) {
            lift[0][i] = query_st(first_after(arr[i].second), n).second;
        }
        for (int i = 1; i <= __lg(n); i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }

    int query(int l, int r, int ql, int qr) const {
        if (ql > qr) {
            return -1e9;
        }
        int cql = max(first_after(ql), l), u = query_st(cql, r).second;
        if (u == -1 || arr[u].second > qr) {
            return 0;
        }
        int ans = 1;
        for (int i = __lg(n); i >= 0; i--) {
            int nu = lift[i][u];
            if (nu != -1 && nu < r && arr[nu].second <= qr) {
                u = nu;
                ans += 1 << i;
            }
        }
        ans += query_st(first_after(arr[u].second), r).first <= qr;
        return ans;
    }
};

void solve() {
    {
        DS1 ds({{3, 9}, {8, 11}, {9, 17}, {12, 19}});
        dbg(ds.query(1, 3, 9, 12));
//        return;
    }
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