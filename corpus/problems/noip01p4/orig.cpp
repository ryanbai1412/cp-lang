// corpus/submissions/5086731.cpp with each zero-initialised VLA cleared by memset: clang rejects `T a[n] {}`.
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
    int n, fc, u, v;
    cin >> n >> fc >> u >> v;
    u--;
    v--;
    int arr[4 * n][2];
    double dist[4 * n][4 * n];
    memset(dist, 0, sizeof dist);
    for (int i = 0; i < n; i++) {
        for (int j = i * 4; j < i * 4 + 3; j++) {
            cin >> arr[j][0] >> arr[j][1];
        }
        for (int j = 0; j < 3; j++) {
            auto& [x1, y1] = arr[i * 4 + j];
            auto& [x2, y2] = arr[i * 4 + ((j + 1) % 3)];
            auto& [x3, y3] = arr[i * 4 + ((j + 2) % 3)];
            if ((y1 - y2) * (y1 - y3) == -(x1 - x2) * (x1 - x3)) {
                arr[i * 4 + 3][0] = x2 + x3 - x1;
                arr[i * 4 + 3][1] = y2 + y3 - y1;
            }
        }
        dbg(arr[i * 4 + 3][0], arr[i * 4 + 3][1]);
        int w;
        cin >> w;
        for (int a = i * 4; a < i * 4 + 4; a++) {
            for (int b = 0; b < i * 4 + 4; b++) {
                auto& [x1, y1] = arr[a];
                auto& [x2, y2] = arr[b];
                int cw = w;
                if (b < 4 * i) {
                    cw = fc;
                }
                dist[a][b] = cw * hypot(x1 - x2, y1 - y2);
            }
        }
    }
    for (int i = 0; i < 4 * n; i++) {
        for (int j = i + 1; j < 4 * n; j++) {
            dist[i][j] = dist[j][i];
        }
    }
    for (int i = 0; i < 4 * n; i++) {
        for (int j = 0; j < 4 * n; j++) {
            for (int k = 0; k < 4 * n; k++) {
                dist[j][k] = min(dist[j][k], dist[j][i] + dist[i][k]);
            }
        }
    }
    double ans = 1e18;
    for (int a = 4 * u; a < 4 * u + 4; a++) {
        for (int b = 4 * v; b < 4 * v + 4; b++) {
            ans = min(ans, dist[a][b]);
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}