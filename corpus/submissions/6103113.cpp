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

struct IO {
    char buf[20 << 20], *i = buf;

    IO() {
        fread_unlocked(buf, 1, sizeof(buf), stdin);
    }

    IO& operator>>(int& x) {
        x = 0;

        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        i++;

        return *this;
    }
} io;

template <typename Cb>
void solve(const vector<int>& arr,
           const vector<int>& arr_sorted,
           bool prefer_first,
           const Cb& cb) {
    int n = sz(arr), m = *max_element(begin(arr), end(arr)) + 1;

    set<int> s(begin(arr_sorted), end(arr_sorted));

    int rep[m], cur_inds[m], next_ind[m];

    memset(rep, -1, sizeof(rep));
    memset(cur_inds, -1, sizeof(cur_inds));

    for (int i = n - 1; i >= 0; i--) {
        next_ind[i] = cur_inds[arr[i]];
        cur_inds[arr[i]] = i;

        if (prefer_first || rep[arr[i]] == -1) {
            rep[arr[i]] = i;
        }
    }

    long sum_l = 0, sum_r = accumulate(begin(arr), end(arr), long(0));
    for (int i = 0; i < n; i++) {
        sum_r -= arr[i];

        if (prefer_first) {
            rep[arr[i]] = next_ind[i];
        } else if (rep[arr[i]] == i) {
            rep[arr[i]] = -1;
        }
        if (rep[arr[i]] == -1) {
            s.erase(arr[i]);
        }

        cb(abs(sum_l - sum_r), i, -1);

        long diff = sum_r - sum_l;
        if (0 <= diff && diff <= int(1e9)) {
            auto it = s.lower_bound(diff / 2);

            auto go = [&](int x) -> void { cb(abs(diff - 2 * x), i, rep[x]); };

            if (it != s.end()) {
                go(*it);
            }
            if (it != s.begin()) {
                go(*(--it));
            }
        }

        sum_l += arr[i];
    }
}

void solve() {
    int n;
    io >> n;

    vector<int> arr(n);
    for (auto& a : arr) {
        io >> a;
    }
    vector<int> arr_sorted = arr;
    sort(begin(arr_sorted), end(arr_sorted));

    tuple<long, int, int> opt {long(1e18), -1, -1};
    solve(arr, arr_sorted, true, [&](long x, int i1, int i2) -> void {
        opt = min(opt, {x, i1, i2});
    });
    reverse(begin(arr), end(arr));
    solve(arr, arr_sorted, false, [&](long x, int i1, int i2) -> void {
        if (i2 == -1) {
            return;
        }

        opt = min(opt, {x, n - i1 - 1, n - i2 - 1});
    });

    auto& [ans, i1, i2] = opt;
    dbg(ans);
    cout << i1 + 1 << " " << (i2 == -1 ? -1 : i2 + 1) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}