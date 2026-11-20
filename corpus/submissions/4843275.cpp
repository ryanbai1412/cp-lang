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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int n, m, mod;
    cin >> n >> m >> mod;
    int arr[n][2];
    for (auto& [len, t] : arr) {
        cin >> len >> t;
        t--;
    }
    set<vector<int>> ans;
    for (int i = 1; i < (1 << n); i++) {
        pair<int, int> mx {-1, -1};
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                mx = max(mx, pair<int, int> {arr[j][0], j});
            }
        }
        vector<int> cur(m);
        for (int j = 0; j < n; j++) {
            if (on(i, j) && j != mx.second && 2 * arr[j][0] > mx.first) {
                goto loop;
            }
        }
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                cur[arr[j][1]]++;
            }
        }
        ans.insert(cur);
    loop:;
    }
    cout << sz(ans) % mod << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}