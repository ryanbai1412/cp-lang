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

struct DS {
	vector<pair<int, int>> v;

	void insert(int x, int y) {
		v.emplace_back(x, y);
		sort(begin(v), end(v), [&](const auto &a, const auto &b) -> bool {
			return a.first - a.second > b.first - b.second;
		});
	}

	void erase(int x, int y) {
		v.erase(find(begin(v), end(v), pair<int, int> {x, y}));
	}

	long qsum(int ind) const {
		long ans = 0;
		for (int i = 0; i < ind; i++) {
			ans += v[i].first - v[i].second;
		}
		return ans;
	}

	int qmaxp(int ind) const {
		int ans = 0;
		for (int i = 0; i < ind; i++) {
			ans = max(ans, v[i].second);
		}
		return ans;
	}

	int qmaxs(int ind) const {
		int ans = 0;
		for (int i = ind; i < sz(v); i++) {
			ans = max(ans, v[i].first);
		}
		return ans;
	}
};

void solve() {
	int n;
	cin >> n;
	pair<int, int> arr[n];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	int q;
	cin >> q;
	DS ds;
	int pos = 0;
	for (auto& [x, y] : arr) {
		ds.insert(x, y);
		pos += x >= y;
	}
	while (q--) {
		int t;
		cin >> t;
		if (t == 0) {
			int x;
			cin >> x;
			int l = 1, r = pos + 1;
			auto valid = [&](int k) -> bool {
				if (ds.qsum(k) + ds.qmaxp(k) >= x) {
					return true;
				} else if (ds.qsum(k - 1) + ds.qmaxs(k - 1) >= x) {
					return true;
				}
				return false;
			};
			while (l < r) {
				int mid = (l + r) / 2;
				if (valid(mid)) {
					r = mid;
				} else {
					l = mid + 1;
				}
			}
			if (!valid(l)) {
				cout << -1 << endl;
			} else {
				cout << l << endl;
			}
		} else {
			t--;
			int x, y;
			cin >> x >> y;
			pos += x >= y;
			pos -= arr[t].first >= arr[t].second;
			ds.erase(arr[t].first, arr[t].second);
			ds.insert(x, y);
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