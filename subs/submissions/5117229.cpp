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

int n, arr[26], arrv[3][26];
bool vis[26];

int mod(int x) {
    if (x >= n) {
        x -= n;
    }
    return x;
}

void dfs(int ind, int carry) {
    if (ind == n) {
        if (!carry) {
            for (int i = 0; i < n; i++) {
                cout << arr[i] << " \n"[i == n - 1];
            }
            exit(0);
        }
        return;
    }
    for (int i = ind + 1; i < n; i++) {
        int a = arr[arrv[0][ind]], b = arr[arrv[1][ind]], c = arr[arrv[2][ind]];
        if (a != -1 && b != -1 && c != -1 && mod(a + b) != c &&
            mod(a + b + 1) % n != c) {
            return;
        }
    }
    for (int i = 0; i < 2; i++) {
        if (arr[arrv[i][ind]] == -1) {
            for (int j = n - 1; j >= 0; j--) {
                if (vis[j]) {
                    continue;
                }
                vis[j] = true;
                arr[arrv[i][ind]] = j;
                dfs(ind, carry);
                vis[j] = false;
                arr[arrv[i][ind]] = -1;
            }
            return;
        }
    }
    int a = arr[arrv[0][ind]], b = arr[arrv[1][ind]], &c = arr[arrv[2][ind]];
    dbg(arr[0], arr[1], arr[2], arr[3], arr[4]);
    int cx = a + b + carry, cv = mod(cx);
    if (c == -1 && !vis[cv]) {
        vis[cv] = true;
        c = cv;
        dfs(ind + 1, cx >= n);
        vis[cv] = false;
        c = -1;
    } else if (cv == c) {
        dfs(ind + 1, cx >= n);
    }
}

void solve() {
    cin >> n;
    for (int i = 0; i < 3; i++) {
        string s;
        cin >> s;
        reverse(begin(s), end(s));
        for (int j = 0; j < n; j++) {
            arrv[i][j] = s[j] - 'A';
        }
    }
    memset(arr, -1, sizeof(arr));
    dfs(0, 0);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}