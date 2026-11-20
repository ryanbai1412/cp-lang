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

struct Psum2 {
    vector<vector<int>> psum;

    Psum2(const vector<vector<int>>& arr)
        : psum(sz(arr) + 1, vector<int>(sz(arr[0]) + 1)) {
        int n = sz(arr), m = sz(arr[0]);
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                psum[i + 1][j + 1] =
                    psum[i + 1][j] + psum[i][j + 1] - psum[i][j] + arr[i][j];
            }
        }
    }

    int query(int x1, int y1, int x2, int y2) const {
        return psum[x2][y2] - psum[x1][y2] - psum[x2][y1] + psum[x1][y1];
    }
};

void solve() {
    int n, m, ax, ay, bx, by;
    cin >> m >> n >> ay >> ax >> by >> bx;
    vector<vector<int>> arr(n, vector<int>(m));
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    Psum2 psum(arr);
    array<int, 5> ans {};
    for (int i = 0; i + ax <= n; i++) {
        for (int j = 0; j + ay <= m; j++) {
            for (int x1 = i + 1; x1 + bx < i + ax; x1++) {
                for (int y1 = j + 1; y1 + by < j + ay; y1++) {
                    ans = max(ans, array<int, 5> {
                                       psum.query(i, j, i + ax, j + ay) -
                                           psum.query(x1, y1, x1 + bx, y1 + by),
                                       i, j, x1, y1});
                }
            }
        }
    }
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < 2; j++) {
            cout << ans[1 + i * 2 + (j ^ 1)] + 1 << " \n"[j == 1];
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}