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

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

//struct DS {
//	vector<int> v;
//
//	void insert(int x, int k) {
//		for (int i = 0; i < k; i++) {
//			v.push_back(x);
//		}
//	}
//
//	void truncate(int x) {
//		if (sz(v) > x) {
//			v.resize(x);
//		}
//	}
//
//	int size() const {
//		return sz(v);
//	}
//
//	vector<pair<int, int>> pop(int x) {
//		assert(x <= sz(v));
//		vector<pair<int, int>> ans;
//		for (int i = 0; i < x; i++) {
//			ans.emplace_back(v.back(), 1);
//			v.pop_back();
//		}
//		return ans;
//	}
//};

struct N {
	int x;
	long cnt, psum;
};

struct DS {
	long siz = 0;
	vector<N> v;

	void insert(int x, long k) {
		v.push_back({x, k, siz});
		siz += k;
	}

	void truncate(long x) {
		if (siz <= x) {
			return;
		}
		while (sz(v) && v.back().psum >= x) {
			v.pop_back();
		}
		if (sz(v)) {
			v.back().cnt = x - v.back().psum;
		}
		siz = x;
	}

	long size() const {
		return siz;
	}

	vector<pair<int, long>> pop(long x) {
		assert(x <= size());
		siz -= x;
		vector<pair<int, long>> ans;
		while (x) {
			long sub = min(x, v.back().cnt);
			x -= sub;
			ans.emplace_back(v.back().x, sub);
			v.back().cnt -= sub;
			if (!v.back().cnt) {
				v.pop_back();
			}
		}
		return ans;
	}
};

void solve() {
	int n;
	cin >> n;
	long bus[n], ppl[n - 1], umbr[n - 1];
	for (auto& a : bus) {
		cin >> a;
	}
	for (auto& a : ppl) {
		cin >> a;
	}
	for (auto& a : umbr) {
		cin >> a;
	}
	long ans[n - 1][3], psum[n + 1] {};
	DS ds;
	long prv = 0;
	for (int i = 0; i < n - 1; i++) {
		long x = min(ppl[i], bus[i] - prv), z = min(bus[i + 1], ppl[i] - x), y = ppl[i] - x - z;
		dbg(i, x, y, z, bus[i], ppl[i], umbr[i]);
		if (y <= umbr[i]) {
			ds.insert(i, umbr[i] - y);
		} else {
			long need = y - umbr[i];
			if (ds.size() < need) {
				cout << "NO" << endl;
				return;
			}
			for (auto& [j, k] : ds.pop(need)) {
				ans[j][1] += k;
				ans[j][2] -= k;
				psum[j + 1] += k;
				psum[i] -= k;
			}
			x += need;
			y -= need;
		}
		ans[i][0] = x;
		ans[i][1] = y;
		ans[i][2] = z;
		prv = z;
		ds.truncate(z);
	}
	long cx = 0;
	for (int i = 0; i < n - 1; i++) {
		cx += psum[i];
		ans[i][0] += cx;
		ans[i][2] -= cx;
	}
	cout << "YES" << endl;
	long cans = 0;
	for (auto& a : ans) {
		cans += a[1];
	}
	cout << cans << endl;
#ifndef STRESS
	for (auto& [x, y, z] : ans) {
		cout << x << " " << y << " " << z << endl;
	}
#endif
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}