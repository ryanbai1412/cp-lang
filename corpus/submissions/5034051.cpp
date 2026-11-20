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

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

const int maxn = 3005;
int tri[2][maxn][maxn], sq[2][maxn][maxn], arr[maxn][maxn];

ll sum_subtriangles(int n, int k, vector<vector<int>> _arr) {
    for (int i = 0; i < n; i++) {
        copy(begin(_arr[i]), end(_arr[i]), arr[n - i - 1]);
    }
    int cind = 0, pind = 1;
    memcpy(sq[cind], arr, sizeof(arr));
    memcpy(tri[cind], arr, sizeof(arr));
    int trix = __lg(k), sqx = __lg((k + 1) / 2);
    for (int i = 1; i <= trix; i++) {
        swap(cind, pind);
        int cx = 1 << (i - 1), cy = 1 << i;
        for (int x = 0; x + cy <= n; x++) {
            for (int y = 0; y + cy <= n; y++) {
                sq[cind][x][y] = max({sq[pind][x][y], sq[pind][x + cx][y], sq[pind][x][y + cx],
                                      sq[pind][x + cx][y + cx]});
                tri[cind][x][y] = max({
                    tri[pind][x + cx][y],
                    tri[pind][x][y + cx],
                    sq[pind][x][y],
                });
            }
        }
    }
    int tind = cind;
    memcpy(sq[cind], arr, sizeof(arr));
    for (int i = 1; i <= sqx; i++) {
        swap(cind, pind);
        int cx = 1 << (i - 1), cy = 1 << i;
        for (int x = 0; x + cy <= n; x++) {
            for (int y = 0; y + cy <= n; y++) {
                sq[cind][x][y] = max({sq[pind][x][y], sq[pind][x + cx][y], sq[pind][x][y + cx],
                                      sq[pind][x + cx][y + cx]});
            }
        }
    }
    long ans = 0;
    dbg(trix, sqx, tri[tind][0][0]);
    int triadd = k - (1 << trix);
    int sqadd = (k + 1) / 2 - (1 << sqx);
    for (int i = 0; i + k <= n; i++) {
        for (int j = 0; j + k <= n - i; j++) {
            dbg(i, j, tri[tind][i][j + triadd], triadd);
            ans += max({sq[cind][i][j], sq[cind][i + sqadd][j],
                        sq[cind][i][j + sqadd], sq[cind][i + sqadd][j + sqadd],
                        tri[tind][i + triadd][j], tri[tind][i][j + triadd]});
        }
    }
    return ans;
}

void solve() {
    int n, k;
    cin >> n >> k;
    vector<vector<int>> arr(n);
    for (int i = 0; i < n; i++) {
        arr[i].resize(i + 1);
        for (auto& a : arr[i]) {
            cin >> a;
        }
    }
    cout << sum_subtriangles(n, k, arr) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}