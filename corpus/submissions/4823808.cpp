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

constexpr long mod = 1e9 + 7;

struct mint {
    static constexpr int norm(long x) {
        if (x < 0) {
            x += mod;
            if (x < 0) {
                return int(x % mod + mod);
            }
            return int(x);
        }
        if (x >= mod) {
            x -= mod;
            if (x >= mod) {
                return int(x % mod);
            }
            return int(x);
        }
        return int(x);
    }

    int x;

    mint() : x(0) {}
    mint(int x) : x(norm(x)) {}
    mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= mod) {
            x -= mod;
        }
        return *this;
    }

    mint& operator-=(const mint& m) {
        x -= m.x;
        if (x < 0) {
            x += mod;
        }
        return *this;
    }

    mint& operator*=(const mint& m) {
        x = int((long(x) * m.x) % mod);
        return *this;
    }

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

template <typename T, size_t N>
ostream& operator<<(ostream& out, const array<T, N>& arr) {
    for (int i = 0; i < N; i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

constexpr int maxn = 1005, maxs = 8, segs[maxs][3] = {{0, 0, 0}, {0, 0, 1}, {0, 0, 2}, {0, 1, 1},
                           {0, 1, 2}, {0, 2, 2}, {1, 1, 2}, {1, 2, 2}};
int n, kv, arr[maxn][2], opts[5][maxs][maxs];
map<long, mint> memo[maxn];

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long hsh(const array<int, maxs>& csegs) {
    uint64_t ans = 0;
    for (auto& a : csegs) {
        ans ^= ans << 17;
        ans ^= ans >> 13;
        ans ^= ans >> 12;
        ans += ans * 0x14121412 + a + 0xdeadbeef;
    }
    return long(ans);
}

mint dp(int ind, const array<int, maxs> &csegs) {
    dbg(csegs);
    if (ind == n) {
        return csegs[0] <= kv;
    }
    long key = hsh(csegs);
    auto it = memo[ind].find(key);
    if (it != memo[ind].end()) {
        return it->second;
    }
    auto cnt_range = [&](int l, int r) -> int {
        auto& [cl, cr] = arr[ind];
        return max(0, min(r, cr) - max(l, cl) + 1);
    };
    mint ans = 0;
    for (int i = 0; i <= 4; i++) {
        int ccnt = cnt_range(i, i);
        if (i == 4) {
            ccnt = cnt_range(4, 2e9);
        }
        if (!ccnt) {
            continue;
        }
        array<int, maxs> nsegs;
        fill(begin(nsegs), end(nsegs), kv + 1);
        for (int j = 0; j < maxs; j++) {
            for (int k = 0; k < maxs; k++) {
                nsegs[k] = min(nsegs[k], csegs[j] + opts[i][j][k]);
            }
        }
        ans += ccnt * dp(ind + 1, nsegs);
    }
    return memo[ind][key] = ans;
}

void pcomp() {
    int ids[3][3][3];
    memset(ids, -1, sizeof(ids));
    for (int i = 0; i < maxs; i++) {
        int cur[3];
        copy(begin(segs[i]), end(segs[i]), cur);
        sort(begin(cur), end(cur));
        do {
            ids[cur[0]][cur[1]][cur[2]] = i;
        } while (next_permutation(begin(cur), end(cur)));
    }
    bool oks[5][maxs][maxs] {};
    for (int x = 0; x < 5; x++) {
        for (int i = 0; i < maxs; i++) {
            auto& [a, b, c] = segs[i];
            for (int j = 0; j <= x; j++) {
                if (j == 1) {
                    continue;
                }
                for (int k = 0; k < 8; k++) {
                    if (__builtin_popcount(k) != x - j) {
                        continue;
                    }
                    bool cans = true;
                    auto upd = [&](bool f,
                                   int x) -> vector<int> {
                        if (f) {
                            if (x == 2) {
                                return {0, 2};
                            } else {
                                return {x + 1};
                            }
                        } else {
                            cans &= !x;
                            return {0};
                        }
                    };
                    auto i1 = upd(k & 1, a);
                    auto i2 = upd(k & 2, b);
                    auto i3 = upd(k & 4, c);
                    if (x == 4 && i == 1) {
                        dbg(k, i1, i2, i3);
                    }
                    if (!cans) {
                        continue;
                    }
                    for (auto& j1 : i1) {
                        for (auto& j2 : i2) {
                            for (auto& j3 : i3) {
                                if (ids[j1][j2][j3] != -1) {
                                    oks[x][i][ids[j1][j2][j3]] = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    memset(opts, 0x3f, sizeof(opts));
    for (int i = 4; i >= 0; i--) {
        for (int j = 0; j < maxs; j++) {
            for (int k = 0; k < maxs; k++) {
                if (oks[i][j][k]) {
                    opts[i][j][k] = 0;
                } else if (i < 4) {
                    opts[i][j][k] = 1 + opts[i + 1][j][k];
                }
            }
        }
    }
}

void solve() {
    cin >> n >> kv;
    assert(kv != 1);
    for (int i = 0; i < n; i++) {
        cin >> arr[i][0] >> arr[i][1];
    }
    for (auto& a : memo) {
        a.clear();
    }
    array<int, maxs> tmp;
    fill(begin(tmp), end(tmp), kv + 1);
    tmp[0] = 0;
    cout << dp(0, tmp) << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    pcomp();
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}