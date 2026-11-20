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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

#ifdef __cplusplus
extern "C" {
#endif

void init(int T);
void orderCoins();
void answer(int W[]);

int getMedian(int A, int B, int C);
int getHeaviest(int A, int B, int C);
int getLightest(int A, int B, int C);
int getNextLightest(int A, int B, int C, int D);

#ifdef __cplusplus
}
#endif

struct G {
    int q_median(int a, int b, int c) {
        return getMedian(a + 1, b + 1, c + 1) - 1;
    }
    int q_heaviest(int a, int b, int c) {
        return getHeaviest(a + 1, b + 1, c + 1) - 1;
    }
    int q_lightest(int a, int b, int c) {
        return getLightest(a + 1, b + 1, c + 1) - 1;
    }
    int q_next_lightest(int a, int b, int c, int d) {
        return getNextLightest(a + 1, b + 1, c + 1, d + 1) - 1;
    }
} G;

template <typename Cb>
struct Cmp {
    Cb cb;

    Cmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

vector<vector<int>> perms;

array<int, 3> sorted(int o, int a, int b, int c) {
    array<int, 3> ans {a, b, c};
    sort(begin(ans), end(ans), Cmp([&](int u) -> int { return perms[o][u]; }));
    return ans;
}

int q_median(int o, int a, int b, int c) {
    return sorted(o, a, b, c)[1];
}

int q_lightest(int o, int a, int b, int c) {
    return sorted(o, a, b, c)[0];
}

int q_heaviest(int o, int a, int b, int c) {
    return sorted(o, a, b, c)[2];
}

int q_next_lightest(int o, int a, int b, int c, int d) {
    Cmp cmp([&](int u) -> int { return perms[o][u]; });

    vector<int> gt;
    for (auto& i : {a, b, c}) {
        if (perms[o][i] > perms[o][d]) {
            gt.push_back(i);
        }
    }
    if (sz(gt)) {
        return *min_element(begin(gt), end(gt), cmp);
    }

    return q_lightest(o, a, b, c);
}

u64 r_vals[720];
int p_mul[720][720];
map<u64, int> memo;

int dp(const vector<int>& arr) {
    if (sz(arr) <= 1) {
        return 0;
    }
    int p3 = 1, log3 = 0;
    while (p3 < sz(arr)) {
        p3 *= 3;
        log3++;
    }

    u64 k_hash = 0;
    for (auto& a : arr) {
        k_hash += r_vals[a];
    }

    auto [it, inserted] = memo.emplace(k_hash, 0);
    int& ans = it->second;
    if (!inserted) {
        return ans;
    }

    ans = 1e9;

    for (int i = 0; i < 6; i++) {
        for (int j = i + 1; j < 6; j++) {
            for (int k = j + 1; k < 6; k++) {
                auto go = [&](auto cb) -> void {
                    if (ans == log3) {
                        return;
                    }

                    vector<int> out[3];

                    for (auto& o : arr) {
                        int res = cb(o);
                        if (res == i) {
                            out[0].push_back(o);
                        } else if (res == j) {
                            out[1].push_back(o);
                        } else {
                            assert(res == k);
                            out[2].push_back(o);
                        }
                    }

                    if (max({sz(out[0]), sz(out[1]), sz(out[2])}) > p3 / 3) {
                        return;
                    }

                    assert(sz(out[0]) + sz(out[1]) + sz(out[2]) == sz(arr));
                    ans =
                        min(ans, 1 + max({dp(out[0]), dp(out[1]), dp(out[2])}));
                };

                go([&](int o) -> int { return q_lightest(o, i, j, k); });
                go([&](int o) -> int { return q_heaviest(o, i, j, k); });
                go([&](int o) -> int { return q_median(o, i, j, k); });
                for (int d = 0; d < 6; d++) {
                    if (d == i || d == j || d == k) {
                        continue;
                    }
                    go([&](int o) -> int {
                        return q_next_lightest(o, i, j, k, d);
                    });
                }
            }
        }
    }

    return ans;
}

void init(int) {
    {
        mt19937_64 cowng(1412);
        for (auto& a : r_vals) {
            a = cowng();
        }
    }

    vector<int> perm(6);
    iota(begin(perm), end(perm), 0);
    do {
        perms.push_back(perm);
    } while (next_permutation(begin(perm), end(perm)));

    for (int i = 0; i < 720; i++) {
        for (int j = 0; j < 720; j++) {
            auto tmp = perms[i];
            for (auto& a : tmp) {
                a = perms[j][a];
            }
            p_mul[i][j] =
                int(lower_bound(begin(perms), end(perms), tmp) - perms.begin());
        }
    }

    vector<int> arr(720);
    iota(begin(arr), end(arr), 0);

    arr.erase(remove_if(begin(arr), end(arr),
                        [&](int o) -> bool {
                            return q_next_lightest(o, 0, 1, 2, 3) != 0;
                        }),
              arr.end());

    dbg(dp({}));
    dbg(sz(arr));
    dbg(dp(arr));
    dbg(sz(memo));
}

void print(vector<int> arr) {
    if (sz(arr) <= 1) {
        vector<int> ans(6);
        auto c_arr = perms[arr[0]];
        for (int i = 0; i < 6; i++) {
            ans[c_arr[i]] = i + 1;
        }
        answer(ans.data());
        return;
    }
    int p3 = 1;
    while (p3 < sz(arr)) {
        p3 *= 3;
    }

    int ans = dp(arr);
    dbg(ans);

    for (int i = 0; i < 6; i++) {
        for (int j = i + 1; j < 6; j++) {
            for (int k = j + 1; k < 6; k++) {
                auto go = [&](auto cb) -> bool {
                    vector<int> out[3];

                    for (auto& o : arr) {
                        int res = cb(o);
                        if (res == i) {
                            out[0].push_back(o);
                        } else if (res == j) {
                            out[1].push_back(o);
                        } else {
                            assert(res == k);
                            out[2].push_back(o);
                        }
                    }

                    if (max({sz(out[0]), sz(out[1]), sz(out[2])}) > p3 / 3) {
                        return false;
                    }

                    assert(sz(out[0]) + sz(out[1]) + sz(out[2]) == sz(arr));
                    if (1 + max({dp(out[0]), dp(out[1]), dp(out[2])}) == ans) {
                        return true;
                    }
                    return false;
                };

                if (go([&](int o) -> int { return q_lightest(o, i, j, k); })) {
                    int res = G.q_lightest(i, j, k);
                    arr.erase(remove_if(begin(arr), end(arr),
                                        [&](int o) -> bool {
                                            return q_lightest(o, i, j, k) !=
                                                   res;
                                        }),
                              arr.end());
                    print(arr);
                    return;
                }
                if (go([&](int o) -> int { return q_heaviest(o, i, j, k); })) {
                    int res = G.q_heaviest(i, j, k);
                    arr.erase(remove_if(begin(arr), end(arr),
                                        [&](int o) -> bool {
                                            return q_heaviest(o, i, j, k) !=
                                                   res;
                                        }),
                              arr.end());
                    print(arr);
                    return;
                }
                if (go([&](int o) -> int { return q_median(o, i, j, k); })) {
                    int res = G.q_median(i, j, k);
                    arr.erase(remove_if(begin(arr), end(arr),
                                        [&](int o) -> bool {
                                            return q_median(o, i, j, k) != res;
                                        }),
                              arr.end());
                    print(arr);
                    return;
                }
                for (int d = 0; d < 6; d++) {
                    if (d == i || d == j || d == k) {
                        continue;
                    }
                    if (go([&](int o) -> int {
                            return q_next_lightest(o, i, j, k, d);
                        })) {
                        int res = G.q_next_lightest(i, j, k, d);
                        arr.erase(remove_if(begin(arr), end(arr),
                                            [&](int o) -> bool {
                                                return q_next_lightest(o, i, j,
                                                                       k, d) !=
                                                       res;
                                            }),
                                  arr.end());
                        print(arr);
                        return;
                    }
                }
            }
        }
    }

    assert(false);
}

void orderCoins() {
    vector<int> arr(720);
    iota(begin(arr), end(arr), 0);
    print(arr);
}