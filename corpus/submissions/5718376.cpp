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

void solve() {
    int n, q;
    cin >> n >> q;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }

    while (q--) {
        string ty;
        cin >> ty;

        if (ty == "INSERT") {
            int ind, len;
            cin >> ind >> len;
            vector<int> carr(len);
            for (auto& a : carr) {
                cin >> a;
            }

            arr.insert(arr.begin() + ind, begin(carr), end(carr));
        } else if (ty == "DELETE") {
            int ind, len;
            cin >> ind >> len;
            ind--;
            arr.erase(arr.begin() + ind, arr.begin() + ind + len);
        } else if (ty == "MAKE-SAME") {
            int ind, len, x;
            cin >> ind >> len >> x;
            ind--;
            fill(arr.begin() + ind, arr.begin() + ind + len, x);
        } else if (ty == "REVERSE") {
            int ind, len;
            cin >> ind >> len;
            ind--;
            reverse(arr.begin() + ind, arr.begin() + ind + len);
        } else if (ty == "GET-SUM") {
            int ind, len;
            cin >> ind >> len;
            ind--;
            cout << accumulate(arr.begin() + ind, arr.begin() + ind + len, 0)
                 << endl;
        } else {
            int ans = 0, csum = 0;
            for (auto& a : arr) {
                csum = max(0, csum + a);
                ans = max(ans, csum);
            }
            cout << ans << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}