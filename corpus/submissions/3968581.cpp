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

struct DS {
	int n, x;
	vector<int> v, arr;

	DS(int n, int x) : n(n), x(x), v(4 * n), arr(n, x) { build(1, 0, n - 1); }

	void build(int o, int l, int r) {
		if (l == r) {
			return void(v[o] = x);
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			build(lc, l, mid);
			build(rc, mid + 1, r);
			v[o] = v[lc] + v[rc];
		}
	}

	void update(int o, int l, int r, int ind, int x) {
		if (l == r) {
			v[o] = x;
			return;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ind <= mid) {
				update(lc, l, mid, ind, x);
			} else {
				update(rc, mid + 1, r, ind, x);
			}
			v[o] = v[lc] + v[rc];
		}
	}

	void update(int ind, int x) {
		update(1, 0, n - 1, ind, x);
		arr[ind] = x;
	}

	int query(int o, int l, int r, int ql, int qr) {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			int ans = 0;
			if (ql <= mid) {
				ans += query(lc, l, mid, ql, qr);
			}
			if (mid < qr) {
				ans += query(rc, mid + 1, r, ql, qr);
			}
			return ans;
		}
	}

	int query(int l, int r) {
		if (l > r) {
			return 0;
		}
		return query(1, 0, n - 1, l, r);
	}
};

const int maxn = 3e5;

int n, arr[maxn], rarr[maxn];

using Data = vector<tuple<int, int, int>>;

namespace S1 {

DS ds(maxn, 1);
vector<pair<int, int>> ans;

void solve(int l, int r, Data x) {
	if (l == r) {
		for (auto& [_, __, ind] : x) {
			ans.emplace_back(l, ind);
		}
		return;
	}
	int mid = (l + r) / 2;
	for (int a = l; a <= mid; a++) {
		ds.update(rarr[a], 0);
	}
	Data lq, rq;
	for (auto& cur : x) {
		auto& [fcnt, ind, _] = cur;
		if ((ind - fcnt >= 0 && ds.arr[ind - fcnt]) ||
			ds.query(max(0, ind - fcnt), n - 1) >= n - ind) {
			rq.push_back(cur);
		} else {
			lq.push_back(cur);
		}
	}
	for (int a = l; a <= mid; a++) {
		ds.update(rarr[a], 1);
	}
	solve(l, mid, lq);
	for (int a = l; a <= mid; a++) {
		ds.update(rarr[a], 0);
	}
	solve(mid + 1, r, rq);
}

vector<pair<int, int>> solve(Data queries) {
	return {};
	solve(0, n - 1, queries);
	return ans;
}

}  // namespace S1

namespace S2 {
vector<pair<int, int>> solve(Data queries) {
	sort(begin(queries), end(queries),
		 [&](auto& a, auto& b) { return get<1>(a) > get<1>(b); });
	DS ds(n, 0), indsm(n, 0);
	auto set = [&](int ind) -> void {
		ds.update(ind, 1);
		indsm.update(ind, ind);
	};
	auto uset = [&](int ind) -> void {
		ds.update(ind, 0);
		indsm.update(ind, 0);
	};
	vector<pair<int, int>> ans;
	int j = 0;
	for (int i = n - 1; i >= 0; i--) {
		set(rarr[i]);
		while (j < sz(queries) && get<1>(queries[j]) == i) {
			auto& [fcnt, data, qind] = queries[j];
			auto psum = [&](long x) -> long { return x * (x + 1) / 2; };
			auto sum = [&](int l, int r) -> long {
				return psum(r) - psum(l - 1);
			};
			auto ccount = [&]() -> long {
				int l = 0, r = n;
				while (l < r) {
					int mid = (l + r) / 2;
					if (ds.query(max(0, mid - fcnt), n - 1) >= n - mid) {
						r = mid;
					} else {
						l = mid + 1;
					}
				}
				int cntb = n - l, cnta = ds.query(0, n - 1) - cntb;
				long suf = sum(n - cntb, n - 1);
				suf += cnta * long(fcnt);
				suf += indsm.query(0, l - fcnt - 1);
				return suf;
			};
			uset(rarr[i]);
			auto start = ccount();
			set(rarr[i]);
			dbg(start, ccount());
			ans.emplace_back(ccount() - start, qind);
			j++;
		}
	}
	return ans;
}
}  // namespace S2

void solve() {
	int q;
	cin >> n >> q;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
		rarr[--arr[i]] = i;
	}
	int cnt = 0;
	vector<tuple<int, int, int>> q1, q2;
	for (int i = 0; i < q; i++) {
		int t;
		cin >> t;
		if (t == 1) {
			cnt++;
		} else if (t == 2) {
			int ind;
			cin >> ind;
			ind--;
			q1.emplace_back(cnt, ind, i);
		} else {
			int x;
			cin >> x;
			x--;
			q2.emplace_back(cnt, x, i);
		}
	}
	int qans[q]{};
	for (auto& [ans, ind] : S1::solve(q1)) {
		qans[ind] = ans + 1;
	}
	for (auto& [ans, ind] : S2::solve(q2)) {
		qans[ind] = ans + 1;
	}
	for (auto& a : qans) {
		if (a) {
			cout << a << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}