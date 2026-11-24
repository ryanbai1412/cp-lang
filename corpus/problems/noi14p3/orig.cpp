// NOI 2014 P3 (Deletion Game), written for the corpus: the accepted submission prints stored
// answers. Each move scans the grid for scoring paths of at most 18 digits (a bounded DFS from every
// start), keeps the best of the first few it finds, deletes it and lets the columns fall.
#include <bits/stdc++.h>
using namespace std;
typedef unsigned long long u64;

static u64 mulmod(u64 a, u64 b, u64 m) { return (unsigned __int128)a * b % m; }
static u64 powmod(u64 a, u64 e, u64 m) {
    u64 r = 1;
    for (a %= m; e; e >>= 1, a = mulmod(a, a, m))
        if (e & 1) r = mulmod(r, a, m);
    return r;
}
static bool is_prime(u64 n) {
    if (n < 2) return false;
    for (u64 p : {2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37})
        if (n % p == 0) return n == p;
    u64 d = n - 1; int s = 0;
    while (!(d & 1)) d >>= 1, s++;
    for (u64 a : {2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37}) {
        u64 x = powmod(a, d, n);
        if (x == 1 || x == n - 1) continue;
        bool comp = true;
        for (int r = 1; r < s && comp; r++) {
            x = mulmod(x, x, n);
            if (x == n - 1) comp = false;
        }
        if (comp) return false;
    }
    return true;
}

int n, m, K, lmin, lmax, c1, c2, F, top;
vector<vector<int>> A;
vector<vector<int>> seen;
int stamp;
vector<pair<int, int>> path, best;
long long best_score;
bool hit;
int budget;

static long long ipow(long long b, int e) { long long r = 1; while (e--) r *= b; return r; }

static long long score(const vector<pair<int, int>>& p, u64 v) {
    int l = p.size();
    bool pal = true;
    for (int i = 0, j = l - 1; i < j; i++, j--)
        if (A[p[i].first][p[i].second] != A[p[j].first][p[j].second]) pal = false;
    long long a = is_prime(v) ? ipow(l, c1) : 1, b = pal ? ipow(l, c2) : 1;
    return a == 1 && b == 1 ? 0 : a + b;
}

static void dfs(int x, int y, u64 v) {
    if (--budget < 0) return;
    int l = path.size();
    if (l >= lmin) {
        long long s = score(path, v);
        if (s > 0) hit = true;
        if (s > best_score) best_score = s, best = path;
    }
    if (l == top) return;
    static const int dx[] = {1, -1, 0, 0}, dy[] = {0, 0, 1, -1};
    for (int d = 0; d < 4; d++) {
        int a = x + dx[d], b = y + dy[d];
        if (a < 0 || a >= n || b < 0 || b >= m || A[a][b] < 0 || seen[a][b] == stamp) continue;
        seen[a][b] = stamp; path.push_back({a, b});
        dfs(a, b, v * 10 + A[a][b]);
        path.pop_back(); seen[a][b] = 0;
        if (budget < 0) return;
    }
}

int main() {
    int tc;
    scanf("%d %d %d %d %d %d %d %d %d", &tc, &n, &m, &K, &lmin, &lmax, &c1, &c2, &F);
    A.assign(n, vector<int>(m));
    seen.assign(n, vector<int>(m, 0));
    for (auto& row : A)
        for (int& x : row) scanf("%d", &x);
    top = min(lmax, 18);
    vector<vector<pair<int, int>>> moves;
    while ((int)moves.size() < K && lmin <= top) {
        best.clear(); best_score = 0;
        int found = 0;
        for (int i = n - 1; i >= 0 && found < 64; i--)
            for (int j = 0; j < m && found < 64; j++) {
                if (A[i][j] <= 0) continue;
                hit = false;
                stamp++; budget = 400;
                seen[i][j] = stamp; path = {{i, j}};
                dfs(i, j, A[i][j]);
                seen[i][j] = 0;
                if (hit) found++;
            }
        if (best.empty()) break;
        moves.push_back(best);
        set<int> cols;
        for (auto [x, y] : best) A[x][y] = -1, cols.insert(y);
        for (int y : cols) {
            int w = n - 1;
            for (int x = n - 1; x >= 0; x--)
                if (A[x][y] >= 0) { int v = A[x][y]; A[x][y] = -1; A[w--][y] = v; }
        }
    }
    printf("%d\n", (int)moves.size());
    for (auto& p : moves) {
        printf("%d", (int)p.size());
        for (auto [x, y] : p) printf(" %d %d", x + 1, y + 1);
        printf("\n");
    }
}
