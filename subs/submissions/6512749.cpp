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

int count_inv(const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    for (int i = 0; i < n; i++) {
        for (int j = i + 1; j < n; j++) {
            ans += arr[i] > arr[j];
        }
    }

    return ans;
}

int count2(const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    for (int i = 0; i < n; i++) {
        ans += abs(arr[i] - i);
    }

    return ans / 2;
}

int count3(const vector<int>& arr) {
    int n = sz(arr);

    vector<int> inv(n);
    for (int i = 0; i < n; i++) {
        inv[arr[i]] = i;
    }

    return count2(inv);
}

bool check1(const vector<int>& arr) {
    return count_inv(arr) == count2(arr);
}

bool check2(const vector<int>& arr) {
    int n = sz(arr);

    for (int i = 0; i < n; i++) {
        bool b1 = false, b2 = false;

        for (int j = 0; j < i; j++) {
            b1 |= arr[j] > arr[i];
        }
        for (int j = i + 1; j < n; j++) {
            b2 |= arr[i] > arr[j];
        }

        if (b1 && b2) {
            return false;
        }
    }

    return true;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    int ans = 0;
    while (next_permutation(begin(arr), end(arr))) {
        // assert(count3(arr) == count2(arr));
        // dbg(arr);
        // assert(check1(arr) == check2(arr));
        // if (check1(arr)) {
        //     dbg(arr);
        // }
        ans += check2(arr);
        // ans += count_inv(arr) == count2(arr);
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}