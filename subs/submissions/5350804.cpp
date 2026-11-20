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
    bool arr[2][n];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    int ans = 0;
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < n; j++) {
            if (!arr[i][j]) {
                continue;
            }
            ans += 3;
            ans -= j && arr[i][j - 1];
            ans -= j + 1 < n && arr[i][j + 1];
            ans -= j % 2 == 0 && arr[!i][j];
            dbg(i, j, ans);
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}