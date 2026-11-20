//region template
#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(T t) {
	cerr << t << endl;
}

template<class T, class... U>
void dbgh(T t, U... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__); cerr
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

using ld = long double;

#define long int64_t
#define sz(x) static_cast<int>((x).size())
//endregion

const ld ninf = -numeric_limits<ld>::infinity();

vector<ld> scores;

void pcomp(int n, int k) {
	scores.resize(n + 1412);
	if(k == 4) {
		for(int i = 0; i < sz(scores); i++) {
			scores[i] = ld(i) * i;
		}
	}else {
		for(int i = 0; i < sz(scores); i++) {
			ld x = sqrt(i);
			scores[i] = x * x * x;
		}
	}
}

// (m - x) ^ k + b
struct Curve {
	int m;
	ld b;

	ld operator()(int x) {
		if(m - x < 0) {
			return ninf;
		}
		return scores[m - x] + b;
	}

	void swap(Curve &c) {
		std::swap(m, c.m);
		std::swap(b, c.b);
	}
};

struct DS {
	int n;
	bool debug;
	vector<Curve> v;

	DS(int n, bool debug = false): n(n), debug(debug), v(4 * n, {0, ninf}) {
		if(debug) {
			dbg("CONSTRUCT", n);
		}
	}

	void update(int o, int l, int r, Curve c) {
		int mid = (l + r) / 2;
		Curve &cur = v[o];
		bool lv = c(l) > cur(l);
		bool mv = c(mid) > cur(mid);
		if(mv) {
			swap(cur, c);
		}
		if(l < r) {
			if(lv != mv || c(mid) == ninf) {
				update(o * 2, l, mid, c);
			}else {
				update(o * 2 + 1, mid + 1, r, c);
			}
		}
	}

	void update(const Curve &c) {
		if(debug) {
			dbg("UPDATE", c.m, c.b);
		}
		update(1, 0, n - 1, c);
	}

	ld query(int o, int l, int r, int x) {
		if(l == r) {
			return v[o](x);
		}
		int mid = (l + r) / 2;
		if(x <= mid) {
			return max(v[o](x), query(o * 2, l, mid, x));
		}
		return max(v[o](x), query(o * 2 + 1, mid + 1, r, x));
	}

	ld query(int x) {
		if(debug) {
			dbg("QUERY", x, query(1, 0, n - 1, x));
		}
		return query(1, 0, n - 1, x);
	}
};

void solve() {
	int n, k;
	cin >> k >> n;
	if(k == 2) {
		cout << ld(n) << endl;
		return;
	}
	pcomp(n, k);
	int arr[n], cnt[n] {};
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		arr[i]--;
		cnt[arr[i]]++;
	}
	vector<DS> ds;
	for(int i = 0; i < n; i++) {
		ds.emplace_back(cnt[i] + 1, i == 117);
	}
	ld dp[n + 1];
	dp[n] = 0;
	for(int i = n - 1; i >= 0; i--) {
		cnt[arr[i]]--;
		ds[arr[i]].update({cnt[arr[i]] + 1, dp[i + 1]});
		dp[i] = ds[arr[i]].query(cnt[arr[i]]);
	}
	cout << dp[0] << endl;
}

int main() {
	cout << setprecision(6) << fixed;
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	if(false) {
		pcomp(10, 10);
		vector<DS> v;
		v.emplace_back(8, true);
		v[0].update({7, 48});
		v[0].query(6);
		v[0].update({6, 59});
		v[0].query(5);
		v[0].update({5, 203});
		v[0].query(4);
		v[0].update({4, 213});
		v[0].query(3);
		v[0].update({3, 234});
		v[0].query(2);
		v[0].update({2, 239});
		v[0].query(1);
		dbg("");
	}
	solve();
}