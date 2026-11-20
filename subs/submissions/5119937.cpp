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

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

ll sum_subtriangles(int n, int k, vector<vector<int>> arr) {
    int pn = 1;
    while (pn < k) {
        int cn = min(k, (pn * 3 + 1) / 2);
        for (int i = 0; i + cn <= n; i++) {
            for (int j = 0; j <= i; j++) {
                arr[i][j] = max({arr[i][j], arr[i + cn - pn][j],
                                 arr[i + cn - pn][j + cn - pn]});
            }
        }
        pn = cn;
    }
    ll ans = 0;
    for (int i = 0; i + k <= n; i++) {
        for (int j = 0; j <= i; j++) {
            ans += arr[i][j];
        }
    }
    return ans;
}

void solve() {
    int n, k;
    cin >> n >> k;
    vector<vector<int>> arr(n);
    for (int i = 0; i < n; i++) {
        arr[i].resize(i + 1);
        for (auto& a : arr[i]) {
            cin >> a;
        }
    }
    cout << sum_subtriangles(n, k, arr) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}