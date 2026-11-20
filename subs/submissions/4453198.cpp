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
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	pair<int, int> arr[n];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	int q;
	cin >> q;
	while (q--) {
		int t;
		cin >> t;
		if (t == 0) {
			int targ;
			cin >> targ;
			pair<int, int> tmp[n];
			copy(arr, arr + n, tmp);
			sort(tmp, tmp + n, [&](const auto& a, const auto& b) -> bool {
				return a.first - a.second > b.first - b.second;
			});
			int suf[n];
			suf[n - 1] = tmp[n - 1].first;
			for (int i = n - 2; i >= 0; i--) {
				suf[i] = max(tmp[i].first, suf[i + 1]);
			}
			long cur = 0;
			int cmx = 0;
			bool found = false;
			for (int i = 0; i < n; i++) {
				if (cur + suf[i] >= targ) {
					found = true;
					cout << i + 1 << endl;
					break;
				}
				cur += tmp[i].first - tmp[i].second;
				cmx = max(cmx, tmp[i].second);
				if (cur + cmx >= targ) {
					found = true;
					cout << i + 1 << endl;
					break;
				}
			}
			if (!found) {
				cout << -1 << endl;
			}
		} else {
			t--;
			int x, y;
			cin >> x >> y;
			arr[t] = {x, y};
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}