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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e4;

struct Bit {
	int v[maxn + 1];

	void reset() {
		memset(v, 0, sizeof(v));
	}

	void upd(int ind, int x) {
		ind++;
		while (ind <= maxn) {
			v[ind] += x;
			ind += ind & -ind;
		}
	}

	int psum(int ind) const {
		int ans = 0;
		while (ind > 0) {
			ans += v[ind];
			ind -= ind & -ind;
		}
		return ans;
	}

	int query(int l, int r) const {
		return psum(r + 1) - psum(l);
	}
} bit1, bit2;

void solve() {
	int n, m;
	cin >> n >> m;
	vector<array<int, 3>> hori(n), vert(m);
	for (auto& [y, x1, x2] : hori) {
		cin >> y >> x1 >> x2;
	}
	for (auto& [x, y1, y2] : vert) {
		cin >> x >> y1 >> y2;
	}
	auto uniqu = [&](vector<int>& arr) -> void {
		sort(begin(arr), end(arr));
		arr.erase(unique(begin(arr), end(arr)), arr.end());
	};
	{
		vector<int> x, y;
		for (auto& [y1, x1, x2] : hori) {
			y.push_back(y1);
			x.push_back(x1);
			x.push_back(x2);
		}
		for (auto& [x1, y1, y2] : vert) {
			x.push_back(x1);
			y.push_back(y1);
			y.push_back(y2);
		}
		auto index = [&](const vector<int>& arr, int x) -> int {
			return int(lower_bound(begin(arr), end(arr), x) - arr.begin());
		};
		uniqu(x);
		uniqu(y);
		for (auto& [y1, x1, x2] : hori) {
			y1 = index(y, y1);
			x1 = index(x, x1);
			x2 = index(x, x2);
		}
		for (auto& [x1, y1, y2] : vert) {
			x1 = index(x, x1);
			y1 = index(y, y1);
			y2 = index(y, y2);
		}
	}
	vector<int> hs;
	for (auto& [y, _a, _b] : hori) {
		hs.push_back(y);
	}
	vector<pair<int, int>> horis[maxn];
	for (auto& [y, x1, x2] : hori) {
		horis[y].emplace_back(x1, x2);
	}
	uniqu(hs);
	long ans = 0;
	auto c2 = [&](int x) -> int {
		return x * (x - 1) / 2;
	};
	int cnt[maxn];
	for (int i1 = 0; i1 < sz(hs); i1++) {
		int y1 = hs[i1];
		memset(cnt, 0, sizeof(cnt));
		auto upd2 = [&](int ind, int x) -> void {
			int cur = c2(cnt[ind]);
			int nxt = c2(cnt[ind] += x);
			bit2.upd(ind, nxt - cur);
		};
		bit1.reset();
		bit2.reset();
		vector<pair<int, int>> dels;
		for (auto& [x, cy1, cy2] : vert) {
			if (cy1 <= y1 && y1 < cy2) {
				bit1.upd(x, +1);
				upd2(x, +1);
				dels.emplace_back(cy2, x);
			}
		}
		sort(begin(dels), end(dels));
		int j = 0;
		for (int i2 = i1 + 1; i2 < sz(hs); i2++) {
			int y2 = hs[i2];
			while (j < sz(dels) && dels[j].first < y2) {
				bit1.upd(dels[j].second, -1);
				upd2(dels[j].second, -1);
			}
			for (auto [l1, r1] : horis[y1]) {
				for (auto [l2, r2] : horis[y2]) {
					int l = max(l1, l2), r = min(r1, r2);
					if (l <= r) {
						dbg(l, r);
						ans += c2(bit1.query(l, r)) - bit2.query(l, r);
					}
				}
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}