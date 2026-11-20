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

int mex(int kv, const vector<int>& arr) {
    int m = *max_element(begin(arr), end(arr)) * kv;
    bool dp[m + 1] {};
    dp[0] = true;
    for (int i = 0; i < kv; i++) {
        bool ndp[m + 1] {};
        memcpy(ndp, dp, sizeof(dp));
        for (auto& a : arr) {
            for (int j = a; j <= m; j++) {
                ndp[j] |= dp[j - a];
            }
        }
        memcpy(dp, ndp, sizeof(dp));
    }
    for (int i = 1; i <= m; i++) {
        if (!dp[i]) {
            return i;
        }
    }
    return m + 1;
}

int n, kv, ans;
vector<int> arr, mans;

void dfs() {
    int cmx = mex(kv, arr);
    if (sz(arr) == n) {
        if (cmx > ans) {
            ans = cmx;
            mans = arr;
        }
        return;
    }
    for (int i = arr.back() + 1; i <= cmx; i++) {
        arr.push_back(i);
        dfs();
        arr.pop_back();
    }
}

void solve() {
    cin >> kv >> n;
    assert(kv + n == 13);
    assert(kv == 10 && n == 3);
    arr = {1};
    dfs();
    assert(is_sorted(begin(mans), end(mans)));
    for (int i = 0; i < n; i++) {
        cout << mans[i] << " \n"[i == n - 1];
    }
    cout << "MAX=" << ans - 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}