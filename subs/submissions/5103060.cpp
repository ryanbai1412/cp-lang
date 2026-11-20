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

using i128 = __int128;

ostream& operator<<(ostream& out, i128 x) {
    return out << long(x);
}

i128 gcd(i128 a, i128 b) {
    return b ? gcd(b, a % b) : a;
}

struct Frac {
    __int128 num, den;

    Frac() : num(0), den(1) {}
    Frac(long x) : num(x), den(1) {}

    friend ostream& operator<<(ostream& out, const Frac& f) {
        return out << f.num << "/" << f.den;
    }

    void reduce() {
        i128 g = gcd(num, den);
        num /= g;
        den /= g;
    }

    Frac& operator+=(const Frac& f) {
        num = num * f.den + den * f.num;
        den *= f.den;
        reduce();
        return *this;
    }
    Frac& operator/=(long x) {
        den *= x;
        reduce();
        return *this;
    }

    friend Frac operator+(Frac a, const Frac& b) {
        return a += b;
    }
    friend Frac operator/(Frac a, int b) {
        return a /= b;
    }
};

constexpr int maxn = 1e5 + 5;

int kv;
vector<int> graph[maxn], igraph[maxn];
bool mvis[maxn];
Frac memo[maxn];

Frac dp(int u) {
    if (mvis[u]) {
        return memo[u];
    }
    mvis[u] = true;
    auto& ans = memo[u];
    for (auto& v : igraph[u]) {
        ans += dp(v) / sz(graph[v]);
    }
    if (u < kv) {
        ans += 1;
    }
    return ans;
}

void solve() {
    int n;
    cin >> n >> kv;
    for (int i = 0; i < n; i++) {
        int m;
        cin >> m;
        for (int j = 0; j < m; j++) {
            int v;
            cin >> v;
            v--;
            graph[i].push_back(v);
            igraph[v].push_back(i);
        }
    }
    for (int i = 0; i < n; i++) {
        if (!sz(graph[i])) {
            auto ans = dp(i);
            cout << ans.num << " " << ans.den << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}