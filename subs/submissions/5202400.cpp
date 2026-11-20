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

template <int exp>
int bpow(int base) {
    int ans = 1;
    for (int i = 0; i < exp; i++) {
        ans *= base;
    }
    return ans;
}

namespace c1 {

int solve_unres(int a) {
    int cbase = 127 - a;
    return bpow<3>(cbase) + bpow<2>(cbase) + bpow<1>(cbase) + bpow<0>(cbase) +
           cbase * (cbase + 1) / 2;
}

int solve_unres(int a, int b) {
    int cbase = 127 - a;
    if (b < a) {
        return 0;
    } else if (b == a) {
        return bpow<2>(cbase) + bpow<1>(cbase) + bpow<0>(cbase);
    } else {
        return bpow<2>(cbase) + 128 - b;
    }
}

int solve_unres(int a, int b, int c) {
    int cbase = 127 - a;
    if (b < a || c < a) {
        return 0;
    } else if (c == a && b != a) {
        return 128 - b;
    } else if (c == a) {
        return bpow<1>(cbase) + bpow<0>(cbase);
    } else {
        return bpow<1>(cbase);
    }
}

int solve_unres(int a, int b, int c, int d) {
    if (b < a || c < a || d < a) {
        return 0;
    } else if (a == c && b > a && d >= b) {
        return 1;
    } else if ((a == b) >= (a == c) && (a == c) >= (a == d)) {
        return 1;
    } else {
        return 0;
    }
}

int count_le(array<int, 4> v) {
    int ans = 0;
    for (int i = 0; i < v[0]; i++) {
        ans += solve_unres(i);
    }
    for (int i = 0; i < v[1]; i++) {
        ans += solve_unres(v[0], i);
    }
    for (int i = 0; i < v[2]; i++) {
        ans += solve_unres(v[0], v[1], i);
    }
    for (int i = 0; i <= v[3]; i++) {
        ans += solve_unres(v[0], v[1], v[2], i);
    }
    return ans;
}

}  // namespace c1

int count_le(array<int, 4> v) {
    return c1::count_le(v);
}

array<int, 4> get_kth(int kv) {
    auto conv = [&](int x) -> array<int, 4> {
        array<int, 4> ans;
        for (int i = 3; i >= 0; i--) {
            ans[i] = x & 127;
            x >>= 7;
        }
        return ans;
    };
    int l = 0, r = 1 << 28;
    while (l < r) {
        int mid = (l + r) / 2;
        if (count_le(conv(mid)) >= kv) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return conv(l);
}

constexpr void rotate(int& x, int& y) {
    int nx = 2 - y, ny = x;
    x = nx;
    y = ny;
}

constexpr void c_swap(int& x, int& y) {
    int tmp = x;
    x = y;
    y = tmp;
}

constexpr array<int, 4> rotate(array<int, 4> v) {
    rotate(v[0], v[1]);
    rotate(v[2], v[3]);
    return v;
}

constexpr array<int, 4> norm(array<int, 4> v) {
    if (tie(v[0], v[1]) > tie(v[2], v[3])) {
        c_swap(v[0], v[2]);
        c_swap(v[1], v[3]);
    }
    return v;
}

constexpr array<array<array<int, 4>, 7>, 4> pcomp() {
    constexpr array<int, 4> segs[7] {
        {2, 0, 2, 1}, {2, 1, 2, 2}, {1, 1, 2, 1}, {1, 1, 2, 0},
        {2, 1, 1, 2}, {0, 0, 2, 1}, {0, 0, 1, 2},
    };
    array<array<array<int, 4>, 7>, 4> ans {};
    for (int i = 0; i < 7; i++) {
        ans[0][i] = segs[i];
        for (int j = 0; j < 3; j++) {
            ans[j + 1][i] = rotate(ans[j][i]);
        }
    }
    for (auto& a : ans) {
        for (auto& b : a) {
            b = norm(b);
        }
    }
    return ans;
}

constexpr auto segs = pcomp();

array<int, 4> conv(pair<pair<int, int>, pair<int, int>> p) {
    return {p.first.first, p.first.second, p.second.first, p.second.second};
}

pair<pair<int, int>, pair<int, int>> conv(array<int, 4> v) {
    return {{v[0], v[1]}, {v[2], v[3]}};
}

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

vector<pair<pair<int, int>, pair<int, int>>> BuildPattern(int n) {
    array<int, 4> v = get_kth(n);
    vector<pair<pair<int, int>, pair<int, int>>> ans;
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 7; j++) {
            if (on(v[i], j)) {
                ans.push_back(conv(segs[i][j]));
            }
        }
    }
    return ans;
}

int GetCardNumber(vector<pair<pair<int, int>, pair<int, int>>> arr) {
    set<array<int, 4>> vis;
    for (auto& a : arr) {
        vis.insert(norm(conv(a)));
    }
    array<int, 4> v {};
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 7; j++) {
            if (vis.count(segs[i][j])) {
                v[i] |= 1 << j;
            }
        }
    }
    array<int, 4> mn = v;
    for (int i = 0; i < 4; i++) {
        rotate(begin(v), v.begin() + 1, end(v));
        mn = min(mn, v);
    }
    return count_le(mn);
}

void solve() {
    mt19937 cowng(1412);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        int n;
        cin >> n;
        dbg(n);
        auto cans = BuildPattern(n);
        for (auto& [p1, p2] : cans) {
            dbg(p1.first, p1.second, p2.first, p2.second);
        }
        shuffle(begin(cans), end(cans), cowng);
        auto rotate = [&](pair<int, int>& p) -> void {
            int x = 2 - p.second, y = p.first;
            p = {x, y};
        };
        int ccnt = cowng() % 4;
        for (auto& a : cans) {
            for (int i = 0; i < ccnt; i++) {
                rotate(a.first);
                rotate(a.second);
            }
            if (cowng() % 2) {
                swap(a.first, a.second);
            }
        }
        dbg("---");
        for (auto& [p1, p2] : cans) {
            dbg(p1.first, p1.second, p2.first, p2.second);
        }
        assert(GetCardNumber(cans) == n);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}