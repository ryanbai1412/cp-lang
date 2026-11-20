#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

#ifdef __cplusplus
extern "C" {
#endif

void init(int R, int C, int H[5000][200], int V[5000][200]);
void changeH(int P, int Q, int W);
void changeV(int P, int Q, int W);
int escape(int V1, int V2);

#ifdef __cplusplus
}
#endif

template <typename T, size_t N>
ostream& operator<<(ostream& out, const array<T, N>& arr) {
    out << "{";
    for (size_t i = 0; i < N; i++) {
        out << arr[i];
        if (i + 1 < N) {
            out << ", ";
        }
    }
    return out << "}";
}

const int maxn = 5000, maxm = 200, bsize = 40, maxt = 128;
// const int maxn = 32, maxm = 10, bsize = 2, maxt = 16;

static_assert((maxn - 1) / bsize < maxt);

template <size_t N, size_t M>
using mat = array<array<int, M>, N>;

template <size_t A, size_t B, size_t C>
mat<A, C> mul(const mat<A, B>& x, const mat<B, C>& y) {
    static_assert(C % 8 == 0);
    mat<A, C> ans;
    for (auto& a : ans) {
        a.fill(1e9);
    }
    for (size_t i = 0; i < A; i++) {
        for (size_t j = 0; j < B; j++) {
            for (size_t k = 0; k < C; k++) {
                ans[i][k] = min(ans[i][k], x[i][j] + y[j][k]);
            }
        }
    }
    return ans;
}

int n, m, harr[5000][200], varr[5000][200];
mat<maxm, maxm> v[maxt * 2];

mat<maxm, maxm> solve(int l, int r) {
    mat<maxm, maxm> ans {};
    for (int i = 0; i < maxm; i++) {
        ans[i].fill(1e9);
        ans[i][i] = 0;
    }
    for (int i = 0; i < m; i++) {
        auto& dp = ans[i];
        for (int j = l; j < r; j++) {
            array<int, maxm> ndp;
            ndp.fill(1e9);
            int psum[m + 1];
            psum[0] = 0;
            partial_sum(harr[j], harr[j] + m, psum + 1);
            int ans = 1e9;
            for (int k = 0; k < m; k++) {
                ans = min(ans, dp[k] - psum[k]);
                ndp[k] = min(ndp[k], ans + varr[j][k] + psum[k]);
            }
            ans = 1e9;
            for (int k = m - 1; k >= 0; k--) {
                ans = min(ans, dp[k] + psum[k]);
                ndp[k] = min(ndp[k], ans + varr[j][k] - psum[k]);
            }
            swap(dp, ndp);
        }
    }
    return ans;
}

void update_leaf(int ind) {
    int x = ind - maxt;
    v[ind] = solve(x * bsize, min(n, (x + 1) * bsize));
}

void maintain(int o) {
    v[o] = mul(v[o * 2], v[o * 2 + 1]);
}

void init(int _n, int _m, int _harr[5000][200], int _varr[5000][200]) {
    n = _n;
    m = _m;
    for (int i = 0; i < n; i++) {
        copy(_harr[i], _harr[i] + m, harr[i]);
        copy(_varr[i], _varr[i] + m, varr[i]);
    }
    for (int i = maxt; i < 2 * maxt; i++) {
        update_leaf(i);
    }
    for (int i = maxt - 1; i; i--) {
        maintain(i);
    }
}

void update(int ind) {
    static int cnt = 0;
    assert(++cnt <= 500);
    ind /= bsize;
    ind += maxt;
    update_leaf(ind);
    while (true) {
        ind >>= 1;
        if (!ind) {
            break;
        }
        maintain(ind);
    }
}

void changeH(int x, int y, int w) {
    harr[x][y] = w;
    update(x);
}

void changeV(int x, int y, int w) {
    varr[x][y] = w;
    update(x);
}

int escape(int a, int b) {
    return v[1][a][b];
}