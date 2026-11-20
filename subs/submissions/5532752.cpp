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

vector<int> pairwise_min(vector<int> a, vector<int> b) {
    if (sz(a) > sz(b)) {
        swap(a, b);
    }
    for (int i = 0; i < sz(a); i++) {
        b[i] = min(b[i], a[i]);
    }
    return b;
}

vector<int> min_conv_slow(const vector<int>& a, const vector<int>& b) {
    int n = sz(a), m = sz(b);
    vector<int> ans(n + m - 1, 1e9);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            ans[i + j] = min(ans[i + j], a[i] + b[j]);
        }
    }
    return ans;
}

vector<int> min_conv(const vector<int>& a, const vector<int>& b) {
    int n = sz(a), m = sz(b);
    if (n <= 10 && m <= 10) {
        return min_conv_slow(a, b);
    }
    vector<int> vs;
    for (int i = 0; i < n - 1; i++) {
        vs.push_back(a[i + 1] - a[i]);
    }
    for (int i = 0; i < m - 1; i++) {
        vs.push_back(b[i + 1] - b[i]);
    }
    sort(begin(vs), end(vs));
    vector<int> ans {0};
    for (auto& x : vs) {
        ans.push_back(ans.back() + x);
    }
    return ans;
}

constexpr int maxn = 1e5 + 5;

using A = array<array<vector<int>, 2>, 2>;

vector<int> solve_slow(const vector<int>& arr) {
    int n = sz(arr), kv = n / 2, dp[n + 1][kv + 1];
    memset(dp, 0x3f, sizeof(dp));
    dp[n][0] = 0;
    for (int i = n - 1; i >= 0; i--) {
        memcpy(dp[i], dp[i + 1], sizeof(dp[i]));
        if (i == n - 1) {
            continue;
        }
        for (int j = 1; j <= kv; j++) {
            dp[i][j] = min(dp[i][j], dp[i + 2][j - 1] + arr[i + 1] - arr[i]);
        }
    }
    return vector<int>(dp[0], dp[0] + kv + 1);
}

int arr[maxn];

A solve(int l, int r) {
    if (r - l <= 10) {
        A ans;
        for (int il = 0; il < 2; il++) {
            for (int ir = 0; ir < 2; ir++) {
                ans[il][ir] =
                    solve_slow(vector<int>(arr + l + il, arr + r + 1 - ir));
            }
        }
        return ans;
    }
    int mid = (l + r) / 2;
    A cl = solve(l, mid), cr = solve(mid + 1, r), ans;
    for (int il = 0; il < 2; il++) {
        for (int ir = 0; ir < 2; ir++) {
            ans[il][ir] = pairwise_min(min_conv(cl[il][0], cr[0][ir]),
                                       min_conv(min_conv(cl[il][1], cr[1][ir]),
                                                {0, arr[mid + 1] - arr[mid]}));
        }
    }
    return ans;
}

void solve() {
    int n, kv;
    cin >> n >> kv;
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    cout << solve(0, n - 1)[0][0][kv] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}