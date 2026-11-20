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

struct Solver {
    int lim;
    map<string, int> ans;
    vector<pair<string, string>> rules;

    Solver(const vector<pair<string, string>>& rules, int lim)
        : lim(lim), rules(rules) {}

    void dfs(const string& s, int d = 0) {
        auto [it, inserted] = ans.emplace(s, d);
        if (!inserted) {
            it->second = min(it->second, d);
        }
        if (d == lim) {
            return;
        }
        for (auto& [r1, r2] : rules) {
            for (int i = 0; i < sz(s); i++) {
                if (s.substr(i, sz(r1)) == r1) {
                    dfs(s.substr(0, i) + r2 + s.substr(i + sz(r1)), d + 1);
                }
            }
        }
    }

    static map<string, int> solve(const vector<pair<string, string>>& rules,
                                  int lim,
                                  const string& s) {
        Solver solver(rules, lim);
        solver.dfs(s);
        return solver.ans;
    }
};

template <typename T>
T swapped(T arr) {
    for (auto& [a, b] : arr) {
        swap(a, b);
    }
    return arr;
}

void solve() {
    string s, t;
    cin >> s >> t;
    vector<pair<string, string>> rules;
    string r1, r2;
    while (cin >> r1 >> r2) {
        rules.emplace_back(r1, r2);
    }
    auto m1 = Solver::solve(rules, 5, s),
         m2 = Solver::solve(swapped(rules), 5, t);
    int ans = 1e9;
    for (auto& [s1, d1] : m1) {
        auto it = m2.find(s1);
        if (it != m2.end()) {
            ans = min(ans, d1 + it->second);
        }
    }
    if (ans == 1e9) {
        cout << "NO ANSWER!" << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    solve();
}