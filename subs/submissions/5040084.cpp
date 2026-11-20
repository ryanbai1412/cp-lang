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

constexpr int maxn = 80, dx[] = {-1, 0, 1, 0}, dy[] = {0, 1, 0, -1};

#ifdef LOCAL
bool arr[maxn][maxn];
#endif

bool vis[maxn][maxn], bad[maxn][maxn];
int n, m, dir, cx, cy;

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < m;
}

void left() {
    if (!dir) {
        dir = 3;
    } else {
        dir--;
    }
    cout << "L" << endl;
}

void right() {
    if (dir == 3) {
        dir = 0;
    } else {
        dir++;
    }
    cout << "R" << endl;
}

bool forward() {
    cout << "F" << endl;
#ifdef LOCAL
    int nx = cx + dx[dir], ny = cy + dy[dir];
    bool b = ibs(nx, ny) && !arr[nx][ny];
#else
    bool b;
    cin >> b;
#endif
    if (b) {
        cx += dx[dir];
        cy += dy[dir];
    }
    return b;
}

void dfs() {
    if (vis[cx][cy]) {
        return;
    }
    vis[cx][cy] = true;
    for (int i = 0; i < 4; i++) {
        int nx = cx + dx[dir], ny = cy + dy[dir];
        if (ibs(nx, ny) && !vis[nx][ny] && !bad[nx][ny]) {
            if (forward()) {
                dfs();
                right();
                right();
                forward();
                left();
                continue;
            } else {
                bad[nx][ny] = true;
            }
        }
        right();
    }
}

void solve() {
    char dirc;
    cin >> n >> m >> cx >> cy >> dirc;
    cx--;
    cy--;
#ifdef LOCAL
    for (int i = 0; i < n; i++) {
        string s;
        cin >> s;
        for (int j = 0; j < m; j++) {
            arr[i][j] = s[j] == 'X';
        }
    }
#endif
    dir = int(string("NESW").find(dirc));
    dfs();
    cout << "A" << endl;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            cout << "X."[vis[i][j]];
        }
        cout << endl;
    }
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}