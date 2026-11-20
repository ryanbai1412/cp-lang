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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(size(x))

void k_eq_1(int n, ld* arr, int k, int p) {
    sort(arr + 1, arr + n, greater<>());
    ld ans = 0, sum = 0;
    for (int i = 1; i <= n; i++) {
        sum += arr[i - 1];
        ans = max(ans, sum / i);
    }
    cout << setprecision(2 * p) << ans << endl;
}

void solve() {
    int n, k, p;
    cin >> n >> k >> p;
    ld arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    if (k == 1) {
        k_eq_1(n, arr, k, p);
        return;
    }
    sort(arr + 1, arr + n);
    for (int i = 1; i < n; i++) {
        arr[0] = max(arr[0], (arr[0] + arr[i]) / 2);
    }
    cout << setprecision(2 * p) << arr[0] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}