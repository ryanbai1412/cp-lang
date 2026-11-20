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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace s1 {

constexpr int maxn = 2e5 + 5;

long memo[maxn];
vector<int> arr, graph[maxn];

void pdfs(int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
    }
}

long dp(int u) {
    long& ans = memo[u];
    if (ans != -1) {
        return ans;
    }
    ans = 0;
    for (auto& v : graph[u]) {
        ans = max(ans, dp(v));
    }
    ans += arr[u];
    return ans;
}

vector<int> out_ans;

void print(int u) {
    out_ans.push_back(u);
    long cans = dp(u) - arr[u];
    if (!cans) {
        return;
    }
    dbg(u, cans);
    for (auto& v : graph[u]) {
        if (dp(v) == cans) {
            print(v);
            return;
        }
    }
    assert(false);
}

void solve(const vector<pair<int, int>>& edges, const vector<int>& _arr) {
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    arr = _arr;
    pdfs(0);
    memset(memo, -1, sizeof(memo));
    cout << dp(0) << endl;
    print(0);
    cout << sz(out_ans) << endl;
    for (int i = 0; i < sz(out_ans); i++) {
        cout << out_ans[i] + 1 << " \n"[i == sz(out_ans) - 1];
    }
}

}  // namespace s1

namespace s2 {

constexpr int maxn = 2e5 + 5;

long memo[maxn][2][2];
vector<int> arr, graph[maxn];

void pdfs(int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
    }
}

template <typename Cb1, typename Cb2, typename Cb3>
tuple<long, int, int, int> opt3(const vector<int>& ch,
                                Cb1 cb1,
                                Cb2 cb2,
                                Cb3 cb3) {
    auto top3 = [&](auto cb) -> vector<pair<long, int>> {
        vector<pair<long, int>> ans;
        for (auto& v : ch) {
            ans.emplace_back(cb(v), v);
        }
        sort(begin(ans), end(ans), greater<>());
        if (sz(ans) >= 3) {
            ans.resize(3);
        }
        return ans;
    };
    tuple<long, int, int, int> opt {long(-1e18), -1, -1, -1};
    auto t1 = top3(cb1), t2 = top3(cb2), t3 = top3(cb3);
    for (auto& [w1, v1] : t1) {
        for (auto& [w2, v2] : t2) {
            for (auto& [w3, v3] : t3) {
                if (v1 == v2 || v2 == v3 || v1 == v3) {
                    continue;
                }
                opt = max(opt, {w1 + w2 + w3, v1, v2, v3});
            }
        }
    }
    return opt;
}

long dp(int u, bool me, bool back) {
    long& ans = memo[u][me][back];
    if (ans != -1) {
        return ans;
    }
    ans = 0;
    long tot_sum = 0;
    for (auto& v : graph[u]) {
        tot_sum += arr[v];
    }
    if (!back) {
        for (auto& v : graph[u]) {
            ans = max(ans, dp(v, false, false));
        }
        for (auto& v1 : graph[u]) {
            for (auto& v2 : graph[u]) {
                if (v1 == v2) {
                    continue;
                }
                ans = max(ans, tot_sum - arr[v1] - arr[v2] +
                                   dp(v1, false, true) + dp(v2, true, false));
            }
        }
        if (!me) {
            {
                auto [vans, _v1, _v2, _v3] = opt3(
                    graph[u],
                    [&](int v1) -> long {
                        return -arr[v1] + dp(v1, true, true);
                    },
                    [&](int v2) -> long {
                        return -arr[v2] + dp(v2, false, true);
                    },
                    [&](int v3) -> long {
                        return -arr[v3] + dp(v3, true, false);
                    });
                ans = max(ans, tot_sum + vans);
            }
            // for (auto& v1 : graph[u]) {
            //     for (auto& v2 : graph[u]) {
            //         for (auto& v3 : graph[u]) {
            //             if (v1 == v2 || v2 == v3 || v1 == v3) {
            //                 continue;
            //             }
            //             ans = max(ans, tot_sum - arr[v1] - arr[v2] - arr[v3]
            //             +
            //                                dp(v1, true, true) +
            //                                dp(v2, false, true) +
            //                                dp(v3, true, false));
            //         }
            //     }
            // }
            for (auto& v1 : graph[u]) {
                for (auto& v2 : graph[u]) {
                    if (v1 == v2) {
                        continue;
                    }
                    ans =
                        max(ans, tot_sum - arr[v1] - arr[v2] +
                                     dp(v1, true, true) + dp(v2, false, false));
                }
            }
            for (auto& v : graph[u]) {
                ans = max(ans, tot_sum - arr[v] + dp(v, false, false));
            }
        }
    } else {
        if (me) {
            for (auto& v : graph[u]) {
                ans = max(ans, tot_sum - arr[v] + dp(v, false, true));
            }
        } else {
            for (auto& v : graph[u]) {
                ans = max(ans, tot_sum - arr[v] + dp(v, true, true));
            }
        }
    }
    ans += arr[u];
    return ans;
}

vector<int> out_ans;

void print(int u, bool me, bool back) {
    dbg(u, me, back);
    long cans = dp(u, me, back) - arr[u];
    if (!cans) {
        out_ans.push_back(u);
        return;
    }
    auto add_excluding = [&](initializer_list<int> nodes) -> void {
        for (auto& v : graph[u]) {
            if (find(begin(nodes), end(nodes), v) == nodes.end()) {
                out_ans.push_back(v);
            }
        }
    };
    long tot_sum = 0;
    for (auto& v : graph[u]) {
        tot_sum += arr[v];
    }
    if (!back) {
        for (auto& v : graph[u]) {
            if (dp(v, false, false) == cans) {
                out_ans.push_back(u);
                print(v, false, false);
                return;
            }
        }
        for (auto& v1 : graph[u]) {
            for (auto& v2 : graph[u]) {
                if (v1 == v2) {
                    continue;
                }
                if (tot_sum - arr[v1] - arr[v2] + dp(v1, false, true) +
                        dp(v2, true, false) ==
                    cans) {
                    out_ans.push_back(u);
                    print(v1, false, true);
                    add_excluding({v1, v2});
                    print(v2, true, false);
                    return;
                }
            }
        }
        if (!me) {
            {
                auto [vans, v1, v2, v3] = opt3(
                    graph[u],
                    [&](int v1) -> long {
                        return -arr[v1] + dp(v1, true, true);
                    },
                    [&](int v2) -> long {
                        return -arr[v2] + dp(v2, false, true);
                    },
                    [&](int v3) -> long {
                        return -arr[v3] + dp(v3, true, false);
                    });
                if (tot_sum + vans == cans) {
                    print(v1, true, true);
                    out_ans.push_back(u);
                    print(v2, false, true);
                    add_excluding({v1, v2, v3});
                    print(v3, true, false);
                    return;
                }
            }
            // for (auto& v1 : graph[u]) {
            //     for (auto& v2 : graph[u]) {
            //         for (auto& v3 : graph[u]) {
            //             if (v1 == v2 || v2 == v3 || v1 == v3) {
            //                 continue;
            //             }
            //             if (tot_sum - arr[v1] - arr[v2] - arr[v3] +
            //                     dp(v1, true, true) + dp(v2, false, true) +
            //                     dp(v3, true, false) ==
            //                 cans) {
            //                 print(v1, true, true);
            //                 out_ans.push_back(u);
            //                 print(v2, false, true);
            //                 add_excluding({v1, v2, v3});
            //                 print(v3, true, false);
            //                 return;
            //             }
            //         }
            //     }
            // }
            for (auto& v1 : graph[u]) {
                for (auto& v2 : graph[u]) {
                    if (v1 == v2) {
                        continue;
                    }
                    if (tot_sum - arr[v1] - arr[v2] + dp(v1, true, true) +
                            dp(v2, false, false) ==
                        cans) {
                        add_excluding({v1, v2});
                        print(v1, true, true);
                        out_ans.push_back(u);
                        print(v2, false, false);
                        return;
                    }
                }
            }
            for (auto& v : graph[u]) {
                if (tot_sum - arr[v] + dp(v, false, false) == cans) {
                    add_excluding({v});
                    out_ans.push_back(u);
                    print(v, false, false);
                    return;
                }
            }
        }
    } else {
        if (me) {
            for (auto& v : graph[u]) {
                if (tot_sum - arr[v] + dp(v, false, true) == cans) {
                    out_ans.push_back(u);
                    print(v, false, true);
                    add_excluding({v});
                    return;
                }
            }
        } else {
            for (auto& v : graph[u]) {
                if (tot_sum - arr[v] + dp(v, true, true) == cans) {
                    add_excluding({v});
                    print(v, true, true);
                    out_ans.push_back(u);
                    return;
                }
            }
        }
    }
    assert(false);
}

void solve(const vector<pair<int, int>>& edges, const vector<int>& _arr) {
    arr = _arr;
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    pdfs(0);
    memset(memo, -1, sizeof(memo));
    dbg(dp(1, false, false));
    dbg(dp(4, true, true));
    dbg(dp(2, false, false));
    cout << dp(0, true, false) << endl;
    print(0, true, false);
    int m = sz(out_ans);
    for (int i = 0; i < m; i++) {
        cerr << out_ans[i] << " \n"[i == m - 1];
    }
    int n = sz(arr), dist[n][n];
    memset(dist, 0x3f, sizeof(dist));
    for (int i = 0; i < n; i++) {
        dist[i][i] = 0;
    }
    for (auto& [u, v] : edges) {
        dist[u][v] = dist[v][u] = 1;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < n; k++) {
                dist[j][k] = min(dist[j][k], dist[j][i] + dist[i][k]);
            }
        }
    }
    for (int i = 0; i < m - 1; i++) {
        dbg(out_ans[i], out_ans[i + 1], dist[out_ans[i]][out_ans[i + 1]]);
        assert(dist[out_ans[i]][out_ans[i + 1]] <= 2);
    }
    assert(sz(set<int>(begin(out_ans), end(out_ans))) == m);
    long csum = 0;
    for (auto& a : out_ans) {
        csum += arr[a];
    }
    assert(csum == dp(0, true, false));
#ifndef STRESS
    cout << m << endl;
    for (int i = 0; i < m; i++) {
        cout << out_ans[i] + 1 << " \n"[i == m - 1];
    }
#endif
}

}  // namespace s2

void solve() {
    int n, kv;
    cin >> n >> kv;
    vector<pair<int, int>> edges(n - 1);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
        dbg(u, v);
    }
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    if (kv == 1) {
        s1::solve(edges, arr);
    } else if (kv == 2) {
        s2::solve(edges, arr);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}