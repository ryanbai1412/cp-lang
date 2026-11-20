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

template <int N>
vector<pair<int, int>> solve(const array<int, N>& arr) {
    map<array<int, N>, vector<pair<int, int>>> mp;
    queue<array<int, N>> q;
    auto push = [&](const array<int, N>& u,
                    const vector<pair<int, int>>& ops) -> void {
        if (mp.emplace(u, ops).second) {
            q.push(u);
        }
    };
    push(arr, {});
    while (true) {
        auto u = q.front();
        q.pop();
        auto ops = mp[u];
        if (*max_element(begin(u), end(u)) == *min_element(begin(u), end(u))) {
            return ops;
        }
        for (int b = 0; b < 19; b++) {
            int m = sz(blocks[b]);
            for (int i = 0; i + m <= N; i++) {
                int mn = *min_element(u.begin() + i, u.begin() + i + m);
                for (int j = 0; j < m; j++) {
                    if (u[i + j] - mn != blocks[b][j].first) {
                        goto loop;
                    }
                }
                for (int j = 0; j < m; j++) {
                    u[i + j] += blocks[b][j].second;
                }
                ops.emplace_back(i, b);
                push(u, ops);
                ops.pop_back();
                for (int j = 0; j < m; j++) {
                    u[i + j] -= blocks[b][j].second;
                }
            loop:;
            }
        }
    }
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
    while (N == 4 && accumulate(arr + ind, arr + ind + N, 0) % 4) {
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
    array<int, N> carr;
    copy(arr + ind, arr + ind + N, carr.begin());
    for (auto& [oi, ob] : solve<N>(carr)) {
        insert(ind + oi, ob);
    }
}

void solve() {
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