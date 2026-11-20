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

constexpr int maxn = 25;

bool found;
int arr[3][maxn], val[30];

int get(int x, int y) {
    return val[arr[x][y]];
}

void dfs(int x, int y) {
    if (found) {
        return;
    } else if (x == -1) {
        for (auto& a : arr) {
            for (auto& b : a) {
                if (b) {
                    if (!val[b]) {
                        return;
                    }
                    break;
                }
            }
        }
        for (auto& a : arr) {
            for (auto& b : a) {
                if (b) {
                    cout << val[b];
                }
            }
            cout << endl;
        }
        found = true;
        return;
    } else if (y == 3) {
        bool carry = (x + 1 < maxn && get(0, x + 1) + get(1, x + 1) >= 10) ;
        if ((get(0, x) + get(1, x) + carry) % 10 == get(2, x)) {
            dfs(x - 1, 0);
        }
        return;
    }
    if (val[arr[y][x]] == -1) {
        bool vis[10] {};
        for (int i = 1; i < sz(val); i++) {
            if (val[i] != -1) {
                vis[val[i]] = true;
            }
        }
        for (int i = 0; i < 10; i++) {
            if (!vis[i]) {
                val[arr[y][x]] = i;
                dfs(x, y + 1);
            }
        }
        val[arr[y][x]] = -1;
    } else {
        dfs(x, y + 1);
    }
}

void solve() {
    memset(arr, 0, sizeof(arr));
    for (int i = 0; i < 3; i++) {
        string s;
        cin >> s;
        for (int j = 0; j < sz(s); j++) {
            arr[i][maxn - sz(s) + j] = s[j] - 'A' + 1;
        }
    }
    found = false;
    memset(val, -1, sizeof(val));
    val[0] = 0;
    dfs(maxn - 1, 0);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}