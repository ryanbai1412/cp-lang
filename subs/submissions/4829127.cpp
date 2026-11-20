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
    int kv, n, m;
    cin >> kv >> n >> m;
    int arr1[n], arr2[m];
    for (auto& a : arr1) {
        cin >> a;
    }
    for (auto& a : arr2) {
        cin >> a;
    }
    int cnt1[200] {};
    for (auto& a : arr1) {
        cnt1[a]++;
    }
    bool vis[kv + 1] {};
    vis[0] = true;
    for (int i = 1; i < 200; i++) {
        for (int j = 0; j < i; j++) {
            int last = 1e9;
            for (int k = j; k <= kv; k += i) {
                if (vis[k]) {
                    last = 0;
                }
                vis[k] = last <= cnt1[i];
                last++;
            }
        }
    }
    sort(arr2, arr2 + m, greater<>());
    int ans2[kv + 1], psum = 0;
    ans2[0] = 0;
    for (int i = 1, j = 0; i <= kv; i++) {
        if (psum < i) {
            if (j < m) {
                psum += arr2[j++];
            }
            ans2[i] = ans2[i - 1];
            continue;
        }
        ans2[i] = ans2[i - 1] + 1;
    }
    int ans = 0, tot1 = accumulate(arr1, arr1 + n, 0);
    for (int i = 0; i <= kv; i++) {
        int cans = min(tot1, vis[i] ? i : i - 1);
        ans = max(ans, cans + ans2[kv - i]);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}