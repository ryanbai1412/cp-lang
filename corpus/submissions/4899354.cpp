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

template <int MOD, typename T>
struct I {
    static constexpr int norm(int x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return x;
    }

    int x;
    T t;

    constexpr I(int x = 0) : x(norm(x)), t(x) {}

    I& operator+=(const I& i) {
        x += i.x;
        if (x >= MOD) {
            x -= MOD;
        }
        t += i.t;
        return *this;
    }
    I& operator-=(const I& i) {
        x -= i.x;
        if (x < 0) {
            x += MOD;
        }
        t -= i.t;
        return *this;
    }
    I& operator*=(const I& i) {
        x = int(long(x) * i.x % MOD);
        t *= i.t;
        return *this;
    }

    friend I operator+(I a, const I& b) {
        return a += b;
    }
    friend I operator-(I a, const I& b) {
        return a -= b;
    }
    friend I operator*(I a, const I& b) {
        return a *= b;
    }
    
    bool operator==(const I& i) const {
        return x == i.x && t == i.t;
    }
    bool operator<(const I& i) const {
        if (x == i.x) {
            return t < i.t;
        }
        return x < i.x;
    }
};

using HI = I<int(1e9 + 7), uint64_t>;

struct H {
    static constexpr HI BASE = 1412;
    
    vector<HI> psum, pow;
    
    H(const string& s) {
        psum.push_back(0);
        pow.push_back(1);
        for (auto& c : s) {
            psum.push_back(psum.back() * BASE + c);
            pow.push_back(pow.back() * BASE);
        }
    }
    
    HI query(int l, int r) const {
        return psum[r] - psum[l] * pow[r - l];
    }
    
    int size() const {
        return sz(psum) - 1;
    }
};

constexpr int maxn = 2e4 + 5;

vector<int> graph[maxn];
vector<H> arr;

struct Solver {
    int x;
    bool ans = false;
    
    Solver(int x): x(x) {}
    
    set<HI> dfs(int u) {
        set<HI> cur;
        int csz = 0;
        for (auto& v : graph[u]) {
            auto ncur = dfs(v);
            csz += sz(ncur);
            if (sz(cur) < sz(ncur)) {
                swap(cur, ncur);
            }
            cur.insert(begin(ncur), end(ncur));
        }
        if (csz != sz(cur)) {
            ans = true;
        }
        for (int i = 0; i + x <= sz(arr[u]); i++) {
            cur.insert(arr[u].query(i, i + x));
        }
        return cur;
    }
    
    static bool solve(int x) {
        Solver s(x);
        s.dfs(0);
        return s.ans;
    }
};

void solve() {
    int n;
    cin >> n;
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
        p--;
        graph[p].push_back(i);
    }
    for (int i = 0; i < n; i++) {
        string s;
        cin >> s;
        arr.emplace_back(s);
    }
    int l = -1, r = 1e5 + 5;
    while (l < r) {
        int mid = (l + r + 1) / 2;
        if (Solver::solve(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    cout << l << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}