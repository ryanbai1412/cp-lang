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
vector<vector<T>> transposed(const vector<vector<T>>& arr) {
    int n = sz(arr), m = sz(arr[0]);
    vector<vector<T>> ans(m, vector<T>(n));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            ans[i][j] = arr[j][i];
        }
    }
    return ans;
}

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

struct DS1 {
    int n, m;
    vector<vector<int>> psum;

    DS1(const vector<vector<int>>& arr)
        : n(sz(arr)), m(sz(arr[0])), psum(n + 1, vector<int>(m + 1)) {
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                psum[i + 1][j + 1] =
                    arr[i][j] + psum[i + 1][j] + psum[i][j + 1] - psum[i][j];
            }
        }
    }

    int query(int xl, int xr, int yl, int yr) const {
        return psum[xr][yr] - psum[xl][yr] - psum[xr][yl] + psum[xl][yl];
    }
};

vector<int> solve_max_down(int kv, const vector<vector<int>>& arr) {
    int n = sz(arr), m = sz(arr[0]);
    DS1 ds(arr);
    vector<int> ans(n);
    for (int i = n - kv; i >= 0; i--) {
        ans[i] = ans[i + 1];
        for (int j = 0; j + kv <= m; j++) {
            ans[i] = max(ans[i], ds.query(i, i + kv, j, j + kv));
        }
    }
    return ans;
}

int solve(int kv, const vector<vector<int>>& arr) {
    int n = sz(arr), m = sz(arr[0]);
    auto max_down = solve_max_down(kv, arr);
    auto max_up = reversed(solve_max_down(kv, reversed(arr)));
    DS1 ds(arr);
    int pref[m + 1] {}, suff[m + 1] {}, cup = 0, ans = 0;
    for (int i = kv; i < n; i++) {
        int row_max = 0;
        for (int j = 0; j + kv <= m; j++) {
            int cx = ds.query(i - kv, i, j, j + kv);
            pref[j + kv] = max(pref[j + kv], cx);
            suff[j] = max(suff[j], cx);
            row_max = max(row_max, cx);
        }
        for (int j = 0; j < m; j++) {
            pref[j + 1] = max(pref[j + 1], pref[j]);
        }
        for (int j = m - 1; j >= 0; j--) {
            suff[j] = max(suff[j], suff[j + 1]);
        }
        if (i - kv) {
            cup = max(cup, row_max + max_up[i - kv - 1]);
        }
        for (int j = 0; j <= m; j++) {
            cup = max(cup, pref[j] + suff[j]);
        }
        ans = max(ans, cup + max_down[i]);
    }
    return ans;
}

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    vector<vector<int>> arr(n, vector<int> (m));
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    int ans = solve(kv, arr);
    ans = max(ans, solve(kv, reversed(arr)));
    ans = max(ans, solve(kv, transposed(arr)));
    ans = max(ans, solve(kv, reversed(transposed(arr))));
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}