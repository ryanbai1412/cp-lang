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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

vector<pair<int, int>> min_sort(vector<int> arr) {
    int n = sz(arr);

    vector<int> pos(n);
    for (int i = 0; i < n; i++) {
        pos[arr[i]] = i;
    }
    vector<pair<int, int>> ans;

    auto do_swap = [&](int u, int v) -> void {
        ans.emplace_back(u, v);
        swap(arr[u], arr[v]);
        swap(pos[arr[u]], pos[arr[v]]);
    };

    for (int i = 0; i < n; i++) {
        if (arr[i] == i) {
            continue;
        }
        do_swap(i, pos[i]);
    }

    return ans;
}

bool solve_check(const vector<int>& arr, const vector<pair<int, int>>& swaps) {
    int n = sz(arr);

    vector<int> res(n);

    {
        auto tmp = arr;
        for (auto& [u, v] : swaps) {
            swap(tmp[u], tmp[v]);
        }
        for (int i = 0; i < n; i++) {
            res[tmp[i]] = i;
        }
    }

    auto ans_swaps = min_sort(res);

    if (sz(ans_swaps) > sz(swaps)) {
        return false;
    }
    return true;
}

optional<vector<pair<int, int>>> solve(vector<int> arr,
                                       const vector<pair<int, int>>& swaps) {
    int n = sz(arr);

    vector<int> res(n), pos(n);

    for (int i = 0; i < n; i++) {
        pos[arr[i]] = i;
    }

    {
        auto tmp = arr;
        for (auto& [u, v] : swaps) {
            swap(tmp[u], tmp[v]);
        }
        for (int i = 0; i < n; i++) {
            res[tmp[i]] = i;
        }
    }

    auto ans_swaps = min_sort(res);

    if (sz(ans_swaps) > sz(swaps)) {
        return {};
    }

    vector<pair<int, int>> ans;

    auto do_swap = [&](int u, int v, bool swap_res) -> void {
        swap(arr[u], arr[v]);
        swap(pos[arr[u]], pos[arr[v]]);
        if (swap_res) {
            swap(res[arr[u]], res[arr[v]]);
        }
    };

    for (int t = 0; t < sz(ans_swaps); t++) {
        {
            auto& [u, v] = swaps[t];
            do_swap(u, v, false);
        }
        {
            auto& [u, v] = ans_swaps[t];
            ans.emplace_back(pos[u], pos[v]);
            do_swap(pos[u], pos[v], true);
        }
    }
    for (int t = sz(ans_swaps); t < sz(swaps); t++) {
        ans.emplace_back(0, 0);
    }

    return ans;
}

void grade(vector<int> arr,
           const vector<pair<int, int>>& swaps,
           const vector<pair<int, int>>& ans_swaps) {
    assert(sz(ans_swaps) <= sz(swaps));

    for (int i = 0; i < sz(ans_swaps); i++) {
        if (is_sorted(begin(arr), end(arr))) {
            return;
        }
        {
            auto [u, v] = swaps[i];
            swap(arr[u], arr[v]);
        }
        {
            auto [u, v] = ans_swaps[i];
            swap(arr[u], arr[v]);
        }
    }

    assert(is_sorted(begin(arr), end(arr)));
}

int findSwapPairs(int n,
                  int p_arr[],
                  int m,
                  int swaps_x[],
                  int swaps_y[],
                  int out_p[],
                  int out_q[]) {
    vector<int> arr(p_arr, p_arr + n);

    vector<pair<int, int>> swaps(m);
    for (int i = 0; i < m; i++) {
        swaps[i] = {swaps_x[i], swaps_y[i]};
    }

    int l = 0, r = sz(swaps);
    while (l < r) {
        int mid = (l + r) / 2;
        if (solve_check(arr, vector(swaps.begin(), swaps.begin() + mid))) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    auto ans = solve(arr, vector(swaps.begin(), swaps.begin() + l)).value();

    for (int i = 0; i < sz(ans); i++) {
        auto& [p, q] = ans[i];
        out_p[i] = p;
        out_q[i] = q;
    }

    // grade(arr, swaps, ans);
    return sz(ans);
}

// The sample grader, reading the input the port reads (corpus/submissions/5723358.cpp is the grader function).
int main() {
    int n; scanf("%d", &n);
    vector<int> s(n); for (auto& x : s) scanf("%d", &x);
    int m; scanf("%d", &m);
    vector<int> x(m), y(m), p(m), q(m);
    for (int i = 0; i < m; i++) scanf("%d %d", &x[i], &y[i]);
    int r = findSwapPairs(n, s.data(), m, x.data(), y.data(), p.data(), q.data());
    printf("%d\n", r);
    for (int i = 0; i < r; i++) printf("%d %d\n", p[i], q[i]);
}
