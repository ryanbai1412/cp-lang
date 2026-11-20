#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

long solve_bf(vector<int> arr) {
    int n = sz(arr) - 1, matched = 0;
    long twos = 0, free_t = 0;
    for (int i = n, j = n; i >= 1; i--) {
        while (i <= 2 * j - 1) {
            free_t += arr[j] / 2;
            j--;
        }
        if (arr[i] % 2 && free_t) {
            free_t--;
            matched++;
        }
        twos += arr[i] / 2;
    }
    return matched + (twos - matched) * 2 / 3;
}

long solve2(vector<pair<int, int>> arr) {
    long twos = 0, matched = 0, free_t = 0;
    for (int i = sz(arr) - 1; i >= 0; i--) {
        auto& [x, free] = arr[i];
        free_t += free;
        if (x % 2 && free_t) {
            matched++;
            free_t--;
        }
        twos += x / 2;
    }
    return matched + (twos - matched) * 2 / 3;
}

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n + 1);
    vector<pair<int, int>> s_arr(n + 1);
    for (int i = 1; i <= n; i++) {
        cin >> arr[i];
    }
    auto upd = [&](int ind, int mul) -> void {
        s_arr[ind].first += mul * arr[ind];
        s_arr[min(n, 2 * ind - 1)].second += mul * (arr[ind] / 2);
    };
    for (int i = 1; i <= n; i++) {
        upd(i, +1);
    }
    while (q--) {
        int ind, x;
        cin >> ind >> x;
        upd(ind, -1);
        arr[ind] += x;
        upd(ind, +1);
        cout << solve2(s_arr) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}