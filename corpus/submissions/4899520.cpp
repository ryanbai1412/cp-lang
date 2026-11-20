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

void solve() {
    mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int avg = accumulate(arr, arr + n, 0) / n;
    int give[n - 1];
    for (int i = 0, cadd = 0; i < n - 1; i++) {
        give[i] = arr[i] + cadd - avg;
        cadd = give[i];
    }
    pair<int, vector<pair<int, int>>> ans {int(1e9), {}};
    auto start = clock();
    while (double(clock() - start) < 0.9 * CLOCKS_PER_SEC) {
        vector<int> cur(n);
        cur[0] = cowng() % 2000;
        for (int j = 0; j < n - 1; j++) {
            cur[j + 1] = cur[j] - give[j];
        }
        if (*min_element(begin(cur), end(cur)) < 0) {
            continue;
        }
        int carr[n];
        copy(arr, arr + n, carr);
        bool vis[n] {};
        vector<pair<int, int>> cans;
        for (int _ = 0; _ < 3 * n; _++) {
            bool changed = false;
            for (int j = 0; j < n; j++) {
                if (!vis[j]) {
                    vector<int> nbs;
                    if (j) {
                        nbs.push_back(j - 1);
                    }
                    if (j < n - 1) {
                        nbs.push_back(j + 1);
                    }
                    if (carr[j] >= sz(nbs) * cur[j]) {
                        for (auto& a : nbs) {
                            carr[j] -= cur[j];
                            carr[a] += cur[j];
                        }
                        vis[j] = true;
                        if (cur[j]) {
                            cans.emplace_back(j, cur[j]);
                        }
                        cur[j] = 0;
                        changed = true;
                    }
                }
            }
            if (!changed) {
                vector<int> inds;
                for (int j = 0; j < n; j++) {
                    if (!vis[j]) {
                        inds.push_back(j);
                    }
                }
                if (!sz(inds)) {
                    break;
                }
                int j = inds[cowng() % sz(inds)];
                vector<int> nbs;
                if (j) {
                    nbs.push_back(j - 1);
                }
                if (j < n - 1) {
                    nbs.push_back(j + 1);
                }
                int csub = carr[j] / sz(nbs);
                assert(csub <= cur[j]);
                for (auto& a : nbs) {
                    carr[j] -= csub;
                    carr[a] += csub;
                }
                if (csub) {
                    cans.emplace_back(j, csub);
                }
                cur[j] -= csub;
            }
        }
        for (auto& a : vis) {
            if (!a) {
                goto loop;
            }
        }
        ans = min(ans, make_pair(sz(cans), cans));
    loop:;
    }
    auto [cans, xans] = ans;
    cout << cans << endl;
    for (auto& [i, x] : xans) {
        cout << i + 1 << " " << x << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}