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

using dp_t = tuple<int, array<int, 4>, pair<int, int>>;

dp_t memo[1 << 8];

int hsh(const array<int, 4>& arr) {
    int key = 0;
    for (auto& a : arr) {
        key = key * 4 + a;
    }
    return key;
}

dp_t dp(array<int, 4> arr) {
    {
        int mn = *min_element(begin(arr), end(arr));
        for (auto& a : arr) {
            a -= mn;
        }
    }
    int mx = *max_element(begin(arr), end(arr));
    if (!mx) {
        return {0, {}, {-1, -1}};
    }
    if (mx >= 4) {
        for (int i = 0; i < 4; i++) {
            if (arr[i] + 4 <= mx) {
                arr[i] += 4;
                return {1 + get<0>(dp(arr)), arr, {i, 0}};
            }
        }
        assert(false);
    }
    auto& ans = memo[hsh(arr)];
    if (get<0>(ans) != -1) {
        return ans;
    }
    ans = {int(1e9), {}, {}};
    for (int b = 0; b < 19; b++) {
        int m = sz(blocks[b]);
        for (int i = 0; i + m <= 4; i++) {
            int cmn = *min_element(arr.begin() + i, arr.begin() + i + m);
            for (int j = 0; j < m; j++) {
                if (blocks[b][j].first != arr[i + j] - cmn) {
                    goto loop;
                }
            }
            for (int j = 0; j < m; j++) {
                arr[i + j] += blocks[b][j].second;
            }
            ans = min(ans, dp_t {1 + get<0>(dp(arr)), arr, {i, b}});
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

void solve4(int ind) {
    while (accumulate(arr + ind, arr + ind + 4, 0) % 4) {
        balance(ind + 3, ind + 4);
        switch (arr[ind + 4] - arr[ind + 3]) {
            case 0:
                insert(ind + 3, 18);
                break;
            case 1:
                insert(ind + 3, 10);
                break;
            case 2:
                insert(ind + 3, 15);
                break;
            case 3:
                insert(ind + 3, 0);
                insert(ind + 3, 9);
                break;
            default:
                assert(false);
        }
    }
    int mx = int(max_element(arr + ind, arr + ind + 4) - arr);
    for (int i = 0; i < 4; i++) {
        balance(ind + i, mx);
    }
    dbg("A");
    while (true) {
        array<int, 4> carr;
        int mn = *min_element(arr + ind, arr + ind + 4);
        for (int i = 0; i < 4; i++) {
            carr[i] = arr[ind + i] - mn;
        }
        dbg(carr[0], carr[1], carr[2], carr[3], get<0>(dp(carr)));
        auto ops = get<2>(dp(carr));
        if (ops.first == -1) {
            break;
        }
        auto& [oi, ob] = ops;
        insert(ind + oi, ob);
    }
}

void solve() {
    for (int i = 0; i < 100; i++) {
        for (int a = 0; a < 4; a++) {
            for (int b = 0; b < 4; b++) {
                for (int c = 0; c < 4; c++) {
                    for (int d = 0; d < 4; d++) {
                        if ((a + b + c + d) % 4) {
                            continue;
                        }
                        memo[hsh({a, b, c, d})] = {-1, {}, {}};
                        dp({a, b, c, d});
                    }
                }
            }
        }
    }
    cin >> n;
    assert(n % 4 == 0 || n % 4 == 1);
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    int mx = 0;
    for (int i = 0; i < n; i += 4) {
        solve4(i);
        mx = max(mx, arr[i]);
    }
//    for (int i = 0; i < n; i += 4) {
//        while (arr[i] < mx) {
//            insert(i, 1);
//        }
//    }
    if (n % 4 == 1) {
        balance(0, n - 1);
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}