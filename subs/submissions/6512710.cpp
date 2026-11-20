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

int count_inv(const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    for (int i = 0; i < n; i++) {
        for (int j = i + 1; j < n; j++) {
            ans += arr[i] > arr[j];
        }
    }

    return ans;
}

int count2(const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    for (int i = 0; i < n; i++) {
        ans += abs(arr[i] - i);
    }

    return ans / 2;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    int ans = 0;
    while (next_permutation(begin(arr), end(arr))) {
        ans += count_inv(arr) == count2(arr);
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