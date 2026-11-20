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

vector<pair<int, int>> blocks[19] {{{0, 4}},
                                   {{0, 1}, {0, 1}, {0, 1}, {0, 1}},
                                   {{0, 2}, {0, 2}},
                                   {{1, 1}, {0, 2}, {0, 1}},
                                   {{0, 1}, {0, 2}, {1, 1}},
                                   {{1, 2}, {0, 2}},
                                   {{0, 2}, {1, 2}},
                                   {{0, 1}, {0, 2}, {0, 1}},
                                   {{1, 1}, {0, 2}, {1, 1}},
                                   {{1, 1}, {0, 3}},
                                   {{0, 3}, {1, 1}},
                                   {{0, 2}, {1, 1}, {1, 1}},
                                   {{1, 1}, {1, 1}, {0, 2}},
                                   {{0, 2}, {0, 1}, {0, 1}},
                                   {{0, 1}, {0, 1}, {0, 2}},
                                   {{0, 3}, {2, 1}},
                                   {{2, 1}, {0, 3}},
                                   {{0, 3}, {0, 1}},
                                   {{0, 1}, {0, 3}}};

constexpr int maxn = 1505;

using dp_t = pair<int, pair<int, int>>;

dp_t memo[10 << 8];

template <int N>
int hsh(const array<int, N>& arr) {
    int key = N;
    for (auto& a : arr) {
        key = key * 4 + a;
    }
    return key;
}

template <int N>
dp_t dp(array<int, N> arr) {
    {
        int mn = *min_element(begin(arr), end(arr));
        for (auto& a : arr) {
            a -= mn;
        }
    }
    int mx = *max_element(begin(arr), end(arr));
    if (!mx) {
        return {0, {-1, -1}};
    }
    if (mx >= 4) {
        for (int i = 0; i < N; i++) {
            if (arr[i] + 4 <= mx) {
                arr[i] += 4;
                return {1 + get<0>(dp<N>(arr)), {i, 0}};
            }
        }
        assert(false);
    }
    auto& ans = memo[hsh<N>(arr)];
    if (get<0>(ans) != -1) {
        return ans;
    }
    ans = {int(1e9), {}};
    for (int b = 0; b < 19; b++) {
        int m = sz(blocks[b]);
        for (int i = 0; i + m <= N; i++) {
            int cmn = *min_element(arr.begin() + i, arr.begin() + i + m);
            for (int j = 0; j < m; j++) {
                if (blocks[b][j].first != arr[i + j] - cmn) {
                    goto loop;
                }
            }
            for (int j = 0; j < m; j++) {
                arr[i + j] += blocks[b][j].second;
            }
            ans = min(ans, dp_t {1 + get<0>(dp<N>(arr)), {i, b}});
            for (int j = 0; j < m; j++) {
                arr[i + j] -= blocks[b][j].second;
            }
        loop:;
        }
    }
    return ans;
}

int n, arr[maxn];

void insert(int ind, int b) {
    assert(0 <= ind && ind + sz(blocks[b]) <= n);
    cout << b + 1 << " " << ind + 1 << endl;
    for (int i = 0; i < sz(blocks[b]); i++) {
        arr[ind + i] += blocks[b][i].second;
    }
}

void balance(int mn, int mx) {
    if (mn == mx) {
        return;
    }
    if (arr[mx] < arr[mn]) {
        while (arr[mx] < arr[mn]) {
            insert(mx, 0);
        }
    } else {
        while (arr[mn] + 4 <= arr[mx]) {
            insert(mn, 0);
        }
    }
}

template <int N>
void solve(int ind) {
    while (accumulate(arr + ind, arr + ind + N, 0) % 4) {
        int last = ind + N - 1;
        balance(last, ind + N);
        switch (arr[ind + N] - arr[last]) {
            case 0:
                insert(last, 18);
                break;
            case 1:
                insert(last, 10);
                insert(last, 5);
                break;
            case 2:
                insert(last, 15);
                insert(last, 2);
                break;
            case 3:
                insert(last, 0);
                insert(last, 9);
                break;
            default:
                assert(false);
        }
    }
    int mx = int(max_element(arr + ind, arr + ind + N) - arr);
    for (int i = 0; i < N; i++) {
        balance(ind + i, mx);
    }
    dbg("A");
    while (true) {
        array<int, N> carr;
        int mn = *min_element(arr + ind, arr + ind + N);
        for (int i = 0; i < N; i++) {
            carr[i] = arr[ind + i] - mn;
        }
        auto [oi, ob] = dp<N>(carr).second;
        if (oi == -1) {
            break;
        }
        insert(ind + oi, ob);
    }
}

void solve() {
    memset(memo, -1, sizeof(memo));
    mt19937 cowng(1412);
    for (int it = 0; it < int(1e6); it++) {
        int a = cowng() % 4, b = cowng() % 4, c = cowng() % 4, d = cowng() % 4;
        if ((a + b + c + d) % 4 == 0) {
            memo[hsh<4>({a, b, c, d})] = {-1, {}};
            dp<4>({a, b, c, d});
        }
        if ((a + b + c) % 4 == 0) {
            memo[hsh<3>({a, b, c})] = {-1, {}};
            dp<3>({a, b, c});
        }
        if ((a + b) % 4 == 0) {
            memo[hsh<2>({a, b})] = {-1, {}};
            dp<2>({a, b});
        }
    }
    cin >> n;
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    int mx = 0;
    for (int i = 0; i + 4 <= n; i += 4) {
        solve<4>(i);
        mx = max(mx, arr[i]);
    }
    if (n % 4) {
        while (arr[n - 1] < mx) {
            insert(n - 1, 0);
        }
        int base = n & (~3);
        if (n % 4 == 1) {
            solve<1>(base);
        } else if (n % 4 == 2) {
            solve<2>(base);
        } else {
            solve<3>(base);
        }
        assert(mx <= arr[base]);
        mx = arr[base];
    }
    for (int i = 0; i + 4 <= n; i += 4) {
        while (arr[i] < mx) {
            insert(i, 1);
        }
    }
    for (int i = 0; i < n - 1; i++) {
        assert(arr[i] == arr[i + 1]);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}