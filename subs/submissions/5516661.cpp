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
    int n, mod;
    cin >> n >> mod;
    int arr[n + 1];
    arr[0] = 1;
    for (int i = 1; i <= n; i++) {
        cin >> arr[i];
    }
    vector<int> ans;
    for (int i = 1; i <= n; i++) {
        assert(arr[i] == 0 || arr[i] == 1);
        if (!arr[i]) {
            continue;
        }
        ans.push_back(i);
        for (int j = n; j >= i; j--) {
            arr[j] = (arr[j] - arr[j - i] + mod) % mod;
        }
    }
    cout << sz(ans) << endl;
    for (int i = 0; i < sz(ans); i++) {
        cout << ans[i] << " \n"[i == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}