#include "bits/extc++.h"
#include "count.h"

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

ostream& operator<<(ostream& out, const info &a) {
    return out << a.vertex[0] << " " << a.vertex[1] << endl;
}

info merge_r(info a, info b) {
    if (isempty(a)) {
        return b;
    } else if (isempty(b)) {
        return a;
    }
    return MR(a, b);
}

info merge_c(info a, info b) {
    if (isempty(a)) {
        return b;
    } else if (isempty(b)) {
        return a;
    }
//    dbg(a.vertex[0], a.vertex[1], b.vertex[0], b.vertex[1]);
    return MC(a, b);
}

constexpr int maxn = 1e5 + 5;

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

int n;
info lift[maxn][5][16];

void init(int T, int _n, int q, vector<int> p, vector<info> arr, int M) {
    n = _n;
    info blift[n][21];
    for (int i = 0; i < n - 1; i++) {
        blift[i][0] = arr[i];
    }
    blift[n - 1][0] = emptyinfo;
    for (int i = 1; i < 21; i++) {
        for (int j = 0; j < n; j++) {
            blift[j][i] =
                merge_c(blift[j][i - 1], blift[min(n - 1, j + (1 << (i - 1)))][i - 1]);
        }
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < 5; j++) {
            for (int k = 0; k < 16; k++) {
                lift[i][j][k] = emptyinfo;
                int u = i;
                for (int bit = 0; bit < 4 && u < n; bit++) {
                    if (on(k, bit)) {
                        lift[i][j][k] = merge_c(lift[i][j][k], blift[u][j * 4 + bit]);
                        u += 1 << (j * 4 + bit);
                    }
                }
            }
        }
    }
}

info ask(int u, int d) {
    u--;
    int l = max(0, u - d), r = min(n, u + d), len = r - l;
    info ans = emptyinfo;
    for (int i = 0; i < 4; i++) {
        ans = merge_c(ans, lift[min(n - 1, l)][i][len & 15]);
        l += (len & 15) << (i * 4);
        len >>= 4;
    }
    return ans;
}