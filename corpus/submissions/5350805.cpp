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
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int ans[n];
    memset(ans, 0x3f, sizeof(ans));
    for (int i = 0; i < n; i++) {
        int cans = 0;
        for (int j = 0; i - j >= 0 && i + j < n; j++) {
            cans += abs(arr[i - j] - arr[i + j]);
            ans[2 * j] = min(ans[2 * j], cans);
        }
    }
    for (int i = 0; i < n; i++) {
        int cans = 0;
        for (int j = 0; i - j >= 0 && i + j + 1 < n; j++) {
            cans += abs(arr[i - j] - arr[i + j + 1]);
            ans[2 * j + 1] = min(ans[2 * j + 1], cans);
        }
    }
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}