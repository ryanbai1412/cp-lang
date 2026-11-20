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

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

template <typename T>
vector<vector<T>> transposed(const vector<vector<T>>& arr) {
    int n = sz(arr), m = sz(arr[0]);
    vector<vector<T>> ans(m, vector<T>(n));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            ans[j][i] = arr[i][j];
        }
    }
    return ans;
}

vector<int> solve1(const vector<vector<int>>& arr, int kv) {
    int n = sz(arr), m = sz(arr[0]);
    int ans = 1e9;
    vector<int> cans;
    for (int i = 0; i < n; i++) {
        int csum[m] {};
        for (int j = i; j >= 0; j--) {
            for (int k = 0; k < m; k++) {
                csum[k] += arr[j][k];
            }
            for (int l = 0, r = 0, sm = 0; l < m; l++) {
                while (r < m && sm < kv) {
                    sm += csum[r++];
                }
                if (sm == kv) {
                    ans = min(ans, 2 * ((i - j + 1) + (r - l)));
                }
                sm -= csum[l];
            }
        }
        cans.push_back(ans);
    }
    return cans;
}

int solve2(const vector<vector<int>>& arr, int kv) {
    auto va = solve1(arr, kv), vb = solve1(reversed(arr), kv);
    int ans = 1e9;
    for (int i = 0; i < sz(arr) - 1; i++) {
        ans = min(ans, va[i] + vb[sz(arr) - i - 2]);
    }
    return ans;
}

void solve() {
    int n, m, q, kv;
    cin >> n >> m >> q >> kv;
    vector<vector<int>> arr(n, vector<int>(m));
    while (q--) {
        int x, y;
        cin >> x >> y;
        x--;
        y--;
        arr[x][y]++;
    }
    int ans = min(solve2(arr, kv), solve2(transposed(arr), kv));
    if (ans >= 1e9) {
        cout << "NO" << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}