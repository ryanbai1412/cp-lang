// corpus/submissions/5156196.cpp with each zero-initialised VLA cleared by memset: clang rejects `T a[n] {}`.
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

constexpr int bsize = 2520;

void solve() {
    int _dim, mnk, mxk, n;
    cin >> _dim >> mnk >> mxk >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    sort(arr, arr + n);
    for (int prv = 0, pcomp = 0, i = 0; i < n; i++) {
        int cv = arr[i];
        arr[i] = pcomp + ((arr[i] - prv) % bsize) + (min(1, (arr[i] - prv) / bsize) * bsize);
        prv = cv;
        pcomp = arr[i];
    }
    int m = arr[n - 1], dp[m + 20];
    memset(dp, 0, sizeof dp);
    bool vis[m + 1];
    memset(vis, 0, sizeof vis);
    for (auto& a : arr) {
        vis[a] = true;
    }
    for (int i = m; i >= 0; i--) {
        dp[i] = 1e9;
        for (int j = mnk; j <= mxk; j++) {
            dp[i] = min(dp[i], dp[i + j]);
        }
        dp[i] += vis[i];
    }
    cout << dp[0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}