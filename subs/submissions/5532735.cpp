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
    if (n <= 40 && m <= 40) {
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

int arr[maxn];

vector<int> solve(int l, int r) {
    if (l == r) {
        return {0};
    } else if (l + 1 == r) {
        return {0, arr[l + 1] - arr[l]};
    } else if (l + 2 == r) {
        return {0, min(arr[l + 2] - arr[l + 1], arr[l + 1] - arr[l])};
    }
    int mid = (l + r) / 2;
    return pairwise_min(min_conv(solve(l, mid), solve(mid + 1, r)),
                        min_conv(min_conv(solve(l, mid - 1), solve(mid + 2, r)),
                                 {0, arr[mid + 1] - arr[mid]}));
}

void solve() {
    int n, kv;
    cin >> n >> kv;
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    cout << solve(0, n - 1)[kv] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}