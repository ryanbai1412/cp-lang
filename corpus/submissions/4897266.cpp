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
    int n, m, kv;
    cin >> m >> n >> kv;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    array<int, 5> ans {-1};
    for (int i = 0; i < m; i++) {
        int mn[n], mx[n];
        for (int j = 0; j < n; j++) {
            mn[j] = mx[j] = arr[j][i];
        }
        for (int j = i; j < m && j < i + 100; j++) {
            for (int k = 0; k < n; k++) {
                mn[k] = min(mn[k], arr[k][j]);
                mx[k] = max(mx[k], arr[k][j]);
            }
            multiset<int> s;
            for (int l = 0, r = 0; l < n; l++) {
                r = max(r, l);
                while (r < n) {
                    s.insert(mn[r]);
                    s.insert(mx[r]);
                    if (*(--s.end()) - *s.begin() > kv) {
                        s.erase(s.find(mn[r]));
                        s.erase(s.find(mx[r]));
                        break;
                    }
                    r++;
                }
                ans =
                    max(ans, array<int, 5> {(r - l) * (j - i + 1), i, l, j, r - 1});
                if (l < r) {
                    s.erase(s.find(mn[l]));
                    s.erase(s.find(mx[l]));
                }
            }
        }
    }
    cout << ans[1] + 1 << " " << ans[2] + 1 << " " << ans[3] + 1 << " "
         << ans[4] + 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}