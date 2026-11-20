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

template <bool mn>
struct ST {
    static int op(int a, int b) {
        return mn ? min(a, b) : max(a, b);
    }

    int n;
    vector<vector<int>> v;

    ST(const vector<int>& arr) : n(sz(arr)), v(__lg(n) + 1, vector<int>(n)) {
        v[0] = arr;
        for (int i = 1; i <= __lg(n); i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = op(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    int query(int l, int r) const {
        int k = __lg(r - l + 1);
        return op(v[k][l], v[k][r - (1 << k) + 1]);
    }
};

void solve() {
    int n, m, kv;
    cin >> m >> n >> kv;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    int ans = -1;
    vector<array<int, 4>> poss;
    for (int i = 0; i < m; i++) {
        vector<int> mn(n), mx(n);
        for (int j = 0; j < n; j++) {
            mn[j] = mx[j] = arr[j][i];
        }
        for (int j = i; j < m && j < i + 100; j++) {
            for (int k = 0; k < n; k++) {
                mn[k] = min(mn[k], arr[k][j]);
                mx[k] = max(mx[k], arr[k][j]);
            }
            ST<true> stmn(mn);
            ST<false> stmx(mx);
            for (int l = 0, r = 0; l < n; l++) {
                r = max(r, l);
                while (r < n) {
                    if (stmx.query(l, r) - stmn.query(l, r) > kv) {
                        break;
                    }
                    r++;
                }
                int cans = (r - l) * (j - i + 1);
                if (cans > ans) {
                    ans = cans;
                    poss.clear();
                }
                if (cans == ans) {
                    poss.push_back({i, l, j, r - 1});
                }
            }
        }
    }
//    assert(sz(poss) == 1);
    auto [x1, y1, x2, y2] = poss[0];
    int mn = 1e9, mx = -1e9;
    for (int i = y1; i <= y2; i++) {
        for (int j = x1; j <= x2; j++) {
            mn = min(mn, arr[i][j]);
            mx = max(mx, arr[i][j]);
        }
    }
    assert(mx - mn <= kv);
    cout << x1 + 1 << " " << y1 + 1 << " " << x2 + 1 << " " << y2 + 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}