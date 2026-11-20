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

bool vis[maxn][maxn];
int dir, cx, cy;

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
    bool b;
    cin >> b;
    if (b) {
        cx += dx[dir];
        cy += dy[dir];
    }
}

void dfs() {
    if (vis[cx][cy]) {
        return;
    }
    vis[cx][cy] = true;
    for (int i = 0; i < 4; i++) {
        if (forward()) {
            dfs();
            right();
            right();
            forward();
            right();
            right();
        }
        right();
    }
}

void solve() {
    int n, m;
    char dirc;
    cin >> n >> m >> cx >> cy >> dirc;
    cx--;
    cy--;
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
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}