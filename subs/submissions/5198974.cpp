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

void solve() {
    int n;
    cin >> n;
    long arr[n + 1];
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    arr[n] = 0;
    long req[n + 1] {};
    req[n] = 1;
    long ans = 0;
    for (int i = n; i >= 0; i--) {
        if (req[i] <= arr[i]) {
            continue;
        }
        ans += req[i] - arr[i];
        for (int j = i - 1; j >= 0; j--) {
            req[j] += req[i] - arr[i];
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}