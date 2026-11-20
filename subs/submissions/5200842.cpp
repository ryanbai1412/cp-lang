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

long egcd(long a, long b, long& x, long& y) {
    if (!b) {
        x = 1;
        y = 0;
        return a;
    }
    long x1, y1, p = a / b, g = egcd(b, a % b, x1, y1);
    y = x1 - p * y1;
    x = y1;
    return g;
}

pair<int, int> solve(long a, long b, long kv, int limx, int limy) {
    if (!kv) {
        return {0, 0};
    }
    long x, y, g = egcd(a, b, x, y);
    if (!g || kv % g) {
        return {-1, -1};
    }
    a /= g;
    b /= g;
    kv /= g;
    x *= kv;
    y *= kv;
    pair<int, int> ans {-1, -1};
    auto check = [&](long k) -> void {
        long cx = x + k * b, cy = y - k * a;
        dbg(cx, cy);
        if (0 <= cx && cx <= limx && 0 <= cy && cy <= limy) {
            ans = {cx, cy};
        }
    };
    auto check1 = [&](long k) -> void {
        check(k - 1);
        check(k);
        check(k + 1);
    };
    check1(-x / b);
    check1((limx - x) / b);
    check1(y / a);
    check1((y - limy) / a);
    return ans;
}

array<int, 3> solve(array<long, 3> mul, long kv, array<int, 3> lim) {
    auto [mulx, muly, mulz] = mul;
    auto [limx, limy, limz] = lim;
    for (int z = 0; z <= limz; z++) {
        auto [x, y] = solve(mulx, muly, kv - z * mulz, limx, limy);
        if (x != -1) {
            return {x, y, z};
        }
    }
    return {-1, -1, -1};
}

int b_ty(char c) {
    switch (c) {
        case '(':
            return 1;
        case ')':
            return -1;
        case '[':
            return 2;
        case ']':
            return -2;
        case '{':
            return 3;
        case '}':
            return -3;
        default:
            assert(false);
    }
}

int b_ind(char c) {
    return abs(b_ty(c)) - 1;
}

void solve() {
    int n;
    long kv;
    array<long, 3> mul;
    cin >> n >> kv;
    for (auto& a : mul) {
        cin >> a;
    }
    string s;
    cin >> s;
    vector<pair<int, int>> pairs;
    {
        vector<int> st;
        for (int i = 0; i < n; i++) {
            if (b_ty(s[i]) > 0) {
                st.push_back(i);
            } else {
                pairs.emplace_back(st.back(), i);
                st.pop_back();
            }
        }
    }
    array<int, 3> lim {};
    for (auto& a : s) {
        lim[abs(b_ty(a)) - 1]++;
    }
    bool odd[3] {};
    for (auto& [l, r] : pairs) {
        if (b_ind(s[l]) != b_ind(s[r])) {
            odd[b_ind(s[l])] = odd[b_ind(s[r])] = true;
        }
    }
    for (int i = 0; i < 3; i++) {
        if (!odd[i]) {
            lim[i] /= 2;
            mul[i] *= 2;
        }
    }
    dbg(mul[0], mul[1], mul[2], kv, lim[0], lim[1], lim[2]);
    auto ans = solve(mul, kv, lim);
    if (ans[0] == -1) {
        cout << "impossible" << endl;
        return;
    }
    for (int i = 0; i < 3; i++) {
        if (!odd[i]) {
            ans[i] *= 2;
        }
    }
    bool change[n] {};
    for (auto& [l, r] : pairs) {
        if (b_ind(s[l]) == b_ind(s[r]) && ans[b_ind(s[l])] >= 2) {
            ans[b_ind(s[l])] -= 2;
            change[l] = change[r] = true;
        }
    }
    for (auto& [l, r] : pairs) {
        if (b_ind(s[l]) != b_ind(s[r])) {
            auto go = [&](int ind) -> void {
                if (ans[b_ind(s[ind])]) {
                    ans[b_ind(s[ind])]--;
                    change[ind] = true;
                }
            };
            go(l);
            go(r);
        }
    }
    assert(!ans[0] && !ans[1] && !ans[2]);
    for (auto& [l, r] : pairs) {
        if (!change[l] && !change[r]) {
            s[l] = '<';
            s[r] = '>';
        } else if (change[l] && change[r]) {
            int cind;
            if (b_ind(s[l]) == b_ind(s[r])) {
                cind = (b_ind(s[l]) + 1) % 3;
            } else {
                cind = 0 ^ 1 ^ 2 ^ b_ind(s[l]) ^ b_ind(s[r]);
            }
            s[l] = "([{"[cind];
            s[r] = ")]}"[cind];
        } else if (change[l]) {
            s[l] = "([{"[b_ind(s[r])];
        } else if (change[r]) {
            s[r] = ")]}"[b_ind(s[l])];
        } else {
            assert(false);
        }
    }
    cout << s << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}