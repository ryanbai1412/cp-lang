#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
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

int solve(int xa, int xb, const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    vector<int> targ(n);
    for (int i = 0; i < n; i++) {
        if (i % 2) {
            targ[i] = xb;
        } else {
            targ[i] = xa;
        }
    }

    for (int i = 0; i < n; i++) {
        ans += arr[i] != targ[i];
    }

    for (int i = 0; i + 1 < n; i++) {
        if (arr[i] == targ[i + 1] && arr[i + 1] == targ[i] &&
            (!i || arr[i - 1] != targ[i])) {
            ans++;
        }
    }

    return ans;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
        assert(0 <= a && a < n);
    }

    int ans = n;
    for (int a = 0; a <= n; a++) {
        for (int b = 0; b <= n; b++) {
            if (a == b) {
                continue;
            }
            ans = min(ans, solve(a, b, arr));
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