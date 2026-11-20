#pragma GCC optimize("Ofast,unroll-loops")

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

constexpr int n = 9, scores[5] = {10, 9, 8, 7, 6};

int arr[n][n], ans = -1;
bool visr[n][n], visc[n][n], visg[3][3][n];

void dfs(int x, int y) {
    if (x == n) {
        int cans = 0;
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                cans += (arr[i][j] + 1) * scores[max(abs(i - 4), abs(j - 4))];
            }
        }
        ans = max(ans, cans);
        return;
    } else if (y == -1) {
        dfs(x + 1, n - 1);
        return;
    } else if (arr[x][y] != -1) {
        dfs(x, y - 1);
        return;
    }
    auto& cvg = visg[x / 3][y / 3];
    for (int i = 0; i < 9; i++) {
        if (!visr[x][i] && !visc[y][i] && !cvg[i]) {
            visr[x][i] = visc[y][i] = cvg[i] = true;
            arr[x][y] = i;
            dfs(x, y - 1);
            visr[x][i] = visc[y][i] = cvg[i] = false;
            arr[x][y] = -1;
        }
    }
}

void solve() {
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
            b--;
        }
    }
    bool bad = false;
    auto upd = [&](bool& b) -> void {
        bad |= b;
        b = true;
    };
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            if (arr[i][j] != -1) {
                upd(visr[i][arr[i][j]]);
                upd(visc[j][arr[i][j]]);
                upd(visg[i / 3][j / 3][arr[i][j]]);
            }
        }
    }
    if (bad) {
        cout << -1 << endl;
        return;
    }
    dfs(0, n - 1);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}