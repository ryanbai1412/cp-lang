#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

using P = pair<int, int>;

P& operator+=(P& a, const P& b) {
    a.first += b.first;
    a.second += b.second;
    return a;
}

P operator+(P a, const P& b) {
    return a += b;
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename T>
auto& get(T& arr, P p) {
    return arr[p.first][p.second];
}

constexpr P DIRS[4] {{0, 1}, {0, -1}, {1, 0}, {-1, 0}};

int n, m;

bool ibs(P p) {
    auto [x, y] = p;
    return 0 <= x && x < n && 0 <= y && y < m;
}

int ans, deg1;
bool vis[4][100];
string arr[4], vis_ord;

bool blocked(P p) {
    return !ibs(p) || get(vis, p);
}

int degree(P p) {
    int ans = 0;
    for (auto& dp : DIRS) {
        ans += !blocked(p + dp);
    }
    return ans;
}

bool prune(P p) {
    for (int i : {0, 2}) {
        bool ok = true;
        for (int j : {0, 1}) {
            ok &= blocked(p + DIRS[i | j]) && !blocked(p + DIRS[(i ^ 2) | j]);
        }
        if (ok) {
            return true;
        }
    }

    int deg1 = 0;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            P cp {i, j};
            if (p != cp && degree(cp) == 1 && !blocked(cp)) {
                deg1++;
            }
        }
    }

    if (deg1 > 1) {
        // dbg(p, deg1);
        return true;
    }

    return false;
}

void dfs(P p, int depth) {
    if (blocked(p) || get(arr, p) != vis_ord[depth]) {
        return;
    }
    if (depth == n * m - 1) {
        ans++;
        return;
    }
    if (prune(p)) {
        // return;
    }

    get(vis, p) = true;

    for (auto& dp : DIRS) {
        dfs(p + dp, depth + 1);
    }

    get(vis, p) = false;
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        arr[i].resize(m);
        for (auto& a : arr[i]) {
            cin >> a;
        }
    }
    vis_ord.resize(n * m);
    for (auto& a : vis_ord) {
        cin >> a;
    }

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (i == 0 || i == n - 1 || j == 0 || j == m - 1) {
                dfs({i, j}, 0);
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