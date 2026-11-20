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

template <typename Cb>
void solve(const vector<int>& arr, bool prefer_first, const Cb& cb) {
    int n = sz(arr), m = *max_element(begin(arr), end(arr)) + 1;

    set<int> s;
    int rep[m];
    memset(rep, -1, sizeof(rep));

    long sum_l = accumulate(begin(arr), end(arr), long(0)), sum_r = 0;
    for (int i = n - 1; i >= 0; i--) {
        sum_l -= arr[i];

        cb(abs(sum_l - sum_r), i, -1);

        long diff = sum_r - sum_l;
        if (0 <= diff && diff <= int(1e9)) {
            auto it = s.lower_bound(diff / 2);

            auto go = [&](int x) -> void {
                cb(abs(diff - 2 * x), i, rep[x]);
            };

            if (it != s.end()) {
                go(*it);
            }
            if (it != s.begin()) {
                go(*(--it));
            }
        }

        s.insert(arr[i]);
        if (prefer_first ||  rep[arr[i]] == -1) {
            rep[arr[i]] = i;
        }
        sum_r += arr[i];
    }
}

void solve() {
    int n;
    cin >> n;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }

    tuple<long, int, int> opt {long(1e18), -1, -1};
    solve(arr, true, [&](long x, int i1, int i2) -> void {
        opt = min(opt, {x, i1, i2});
    });
    reverse(begin(arr), end(arr));
    solve(arr, false, [&](long x, int i1, int i2) -> void {
        if (i2 == -1) {
            return;
        }

        opt = min(opt, {x, n - i1 - 1, n - i2 - 1});
    });

    auto& [ans, i1, i2] = opt;
    cout << i1 + 1 << " " << (i2 == -1 ? -1 : i2 + 1) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}