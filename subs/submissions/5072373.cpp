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

template <typename T>
struct S {
    const T* arr;
    int n;

    S(const vector<T>& arr) : arr(arr.data()), n(sz(arr)) {}
    S(const vector<T>& arr, int l, int r) : arr(arr.data() + l), n(r - l) {}
    S(const T* arr, int n) : arr(arr), n(n) {}

    int size() const {
        return n;
    }

    S slice(int l) const {
        return S(arr + l, n - l);
    }

    S slice(int l, int r) const {
        return S(arr + l, r - l);
    }

    const T& operator[](int i) const {
        return arr[i];
    }
};

void lcs(S<int> va, S<int> vb, vector<int>& ans) {
    int n = sz(va), m = sz(vb);
    if (!n || !m) {
        return;
    }
    dbg(n, m);
    int dp[n + 1][m + 1] {};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = m - 1; j >= 0; j--) {
            dp[i][j] = max(dp[i + 1][j], dp[i][j + 1]);
            if (va[i] == vb[j]) {
                dp[i][j] = max(dp[i][j], 1 + dp[i + 1][j + 1]);
            }
        }
    }
    int i = 0, j = 0;
    while (i < n && j < m) {
        if (dp[i][j] == dp[i + 1][j]) {
            i++;
        } else if (dp[i][j] == dp[i][j + 1]) {
            j++;
        } else {
            ans.push_back(va[i]);
            i++;
            j++;
        }
    }
}

void write(vector<bool>& arr, int bits, int x) {
    for (int i = 0; i < bits; i++) {
        arr.push_back((x >> i) & 1);
    }
}

int read(S<bool>& arr, int bits) {
    int ans = 0;
    for (int i = 0; i < bits; i++) {
        ans |= arr[i] << i;
    }
    arr = arr.slice(bits);
    return ans;
}

constexpr int bsize = 1000, bits = 18;

vector<bool> genHint(const vector<int>& va,
                     const vector<int>& vb,
                     const vector<int>& sol) {
    auto advance = [&](const vector<int>& arr, int& i, int x) -> void {
        for (; arr[i] != x; i++)
            ;
        i++;
    };
    vector<bool> ans;
    int i = 0, j = 0, si = 0, sj = 0;
    auto upd = [&](int ci, int cj) -> void {
        dbg(long(ci - si) * (cj - sj));
        write(ans, bits, ci - si);
        write(ans, bits, cj - sj);
        si = ci;
        sj = cj;
    };
    for (auto& a : sol) {
        int pi = i, pj = j;
        advance(va, i, a);
        advance(vb, j, a);
        if (long(i - si) * (j - sj) > bsize * bsize) {
            while (long(pi - si) * (pj - sj) < bsize * bsize &&
                   (pi < i - 1 || pj < j - 1)) {
                pi = min(i - 1, pi + 1);
                pj = min(j - 1, pj + 1);
            }
            dbg(pi, pj, i, j);
            upd(pi, pj);
        }
    }
    return ans;
}

vector<int> solve(const vector<int>& va,
                  const vector<int>& vb,
                  const vector<bool>& hint) {
    bool chint[sz(hint)];
    for (int i = 0; i < sz(hint); i++) {
        chint[i] = hint[i];
    }
    S<bool> r(chint, sz(hint));
    vector<int> ans;
    int i = 0, j = 0;
    while (sz(r)) {
        int di = read(r, bits), dj = read(r, bits);
        lcs(S(va, i, i + di), S(vb, j, j + dj), ans);
        i += di;
        j += dj;
    }
    lcs(S(va, i, sz(va)), S(vb, j, sz(vb)), ans);
    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    vector<int> va(n), vb(m);
    for (auto& a : va) {
        cin >> a;
    }
    for (auto& a : vb) {
        cin >> a;
    }
    vector<int> opt;
    lcs(va, vb, opt);
    auto hint = genHint(va, vb, opt);
    cout << sz(hint) << endl;
    auto ans = solve(va, vb, hint);
    dbg(sz(opt), sz(ans));
    assert(opt == ans);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}