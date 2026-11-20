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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	vector<array<int, 3>> in(n);
	for (auto& [x, y, w] : in) {
		cin >> x >> y >> w;
	}
	long ans = 0;
	for (auto& [x, y, w] : in) {
		vector<pair<ld, int>> arr;
		for (auto& [x1, y1, w1] : in) {
			if (tie(x, y) != tie(x1, y1)) {
				ld d = atan2(y1, x1);
				arr.emplace_back(d, w1);
				arr.emplace_back(d + 2 * M_PIl, w1);
			}
		}
		sort(begin(arr), end(arr));
		int r = 0;
		long cur = 0;
		for (int l = 0; l < n - 1;) {
			for (; r < sz(arr) && arr[r].first - arr[l].first <= M_PIl; r++) {
				cur += arr[r].second;
			}
			ans = max(ans, cur + w);
			ld init = arr[l].first;
			for (; l < n - 1 && arr[l].first == init; l++) {
				cur -= arr[l].second;
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}