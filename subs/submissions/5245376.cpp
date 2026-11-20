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

constexpr int maxn = 55, dirs[8][2] {{-1, -1}, {-1, 0}, {-1, 1}, {0, -1},
                                     {0, 1},   {1, -1}, {1, 0},  {1, 1}};

int n, arr[maxn][maxn], ql, qr;
string grid[maxn];
bool vis[maxn][maxn];

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < n;
}

bool valid(int x, int y) {
    return ibs(x, y) && ql <= arr[x][y] && arr[x][y] <= qr;
}

void dfs(int x, int y) {
    if (!valid(x, y) || vis[x][y]) {
        return;
    }
    vis[x][y] = true;
    for (auto& [dx, dy] : dirs) {
        int cx = x + dx, cy = y + dy;
        dfs(cx, cy);
    }
}

bool check(int al, int ar) {
    ql = al;
    qr = ar;
    memset(vis, 0, sizeof(vis));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            if (grid[i][j] == 'P') {
                dfs(i, j);
            }
        }
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            if (grid[i][j] == 'K' && !vis[i][j]) {
                return false;
            }
        }
    }
    return true;
}

void solve() {
    cin >> n;
    for (int i = 0; i < n; i++) {
        cin >> grid[i];
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            cin >> arr[i][j];
        }
    }
    int ans = 1e9;
    vector<int> vals;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            vals.push_back(arr[i][j]);
        }
    }
    sort(begin(vals), end(vals));
    for (int i = 0; i < n * n; i++) {
        for (int j = i; j < n * n; j++) {
            if (check(vals[i], vals[j])) {
                ans = min(ans, vals[j] - vals[i]);
                break;
            }
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}