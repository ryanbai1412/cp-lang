#include "bits/stdc++.h"
#include "ext/pb_ds/assoc_container.hpp"

using namespace std;
using namespace __gnu_pbds;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int(x.size())

const int maxn = 2e5, tmaxn = 1 << 19, bsize = 1000;

int n;
int arr[maxn];

//segtree
gp_hash_table<int, int> v[tmaxn];

void build(int o, int l, int r) {
	for(int i = l; i <= r; i++) {
		v[o][arr[i]]++;
	}
	if(l < r) {
		int mid = (l + r) / 2;
		build(o * 2, l, mid);
		build(o * 2 + 1, mid + 1, r);
	}
}

void build() {
	build(1, 0, n - 1);
}

void update(int o, int l, int r, int ind, int x) {
	v[o][arr[ind]]--;
	v[o][x]++;
	if(l < r) {
		int mid = (l + r) / 2;
		if(mid >= ind) {
			update(o * 2, l, mid, ind, x);
		}else {
			update(o * 2 + 1, mid + 1, r, ind, x);
		}
	}
}

void update(int ind, int x) {
	update(1, 0, n - 1, ind, x);
	arr[ind] = x;
}

int query(int o, int l, int r, int ql, int qr, int x) {
	if(ql <= l && r <= qr) {
		return v[o][x];
	}else {
		int mid = (l + r) / 2;
		int ans = 0;
		if(mid >= ql) {
			ans += query(o * 2, l, mid, ql, qr, x);
		}
		if(mid < qr) {
			ans += query(o * 2 + 1, mid + 1, r, ql, qr, x);
		}
		return ans;
	}
}

int query(int l, int r, int x) {
	return query(1, 0, n - 1, l, r, x);
}

//not a bit
int barr[maxn + 1], bit[maxn + 1];

void add(int ind, int x) {
	if(x) {
		ind++;
		while(ind <= n) {
			bit[ind] += x;
			ind += ind & -ind;
		}
	}
}

void bset(int ind, int x) {
	add(ind, x - barr[ind]);
	barr[ind] = x;
}

int psum(int ind) {
	int ret = 0;
	while(ind > 0) {
		ret += bit[ind];
		ind -= ind & -ind;
	}
	return ret;
}

int sum(int l, int r) {
	return psum(r + 1) - psum(l);
}

void gen(int ind, int cur, vector<pair<int, int>> &factors, vector<int> &ans) {
	if(cur == 0) {
		abort();
	}
	if(cur <= bsize) {
		if(ind == sz(factors)) {
			ans.push_back(cur);
		}else {
			gen(ind + 1, cur, factors, ans);
			for(int i = 0; i < factors[ind].second; i++) {
				gen(ind + 1, cur *= factors[ind].first, factors, ans);
			}
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int mdiv[maxn + 1] {1, 1};
	for(int i = 2; i <= maxn; i++) {
		if(mdiv[i] == 0) {
			mdiv[i] = i;
			if(maxn / i >= i) {
				for(int j = i * i; j <= maxn; j += i) {
					if(mdiv[j] == 0) {
						mdiv[j] = i;
					}
				}
			}
		}
	}
	vector<int> factors[maxn + 1];
	for(int i = 1; i <= maxn; i++) {
		vector<pair<int, int>> pf;
		int j = i;
		while(j > 1) {
			int cur = mdiv[j], cnt = 0;
			while(mdiv[j] == cur) {
				cnt++;
				j /= cur;
			}
			pf.emplace_back(cur, cnt);
		}
		gen(0, 1, pf, factors[i]);
	}
	int q;
	cin >> n >> q;
	vector<array<int, 3>> qs[bsize + 1];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		for(auto &a: factors[arr[i]]) {
			qs[a].push_back({-1, i, 1});
		}
	}
	int ans[n];
	fill(ans, ans + n, -1);
	for(int t = 0; t < q; t++) {
		int type;
		cin >> type;
		if(type == 1) {
			int l, r, x;
			cin >> l >> r >> x;
			l--;
			r--;
			if(x <= bsize) {
				qs[x].push_back({t, l, r});
			}else {
				int cur = 0;
				for(int i = x; i <= maxn; i += x) {
					cur += query(l, r, x);
				}
				ans[t] = cur;
			}
		}else {
			int ind, x;
			cin >> ind >> x;
			ind--;
			for(auto &a: factors[arr[ind]]) {
				qs[a].push_back({-1, ind, (x % a) == 0});
			}
			update(ind, x);
			for(auto &a: factors[arr[ind]]) {
				qs[a].push_back({-1, ind, 1});
			}
		}
	}
	for(int i = 1; i <= bsize; i++) {
		memset(barr, 0, sizeof(barr));
		memset(bit, 0, sizeof(bit));
		for(auto &a: qs[i]) {
			if(a[0] == -1) {
				bset(a[1], a[2]);
			}else {
				ans[a[0]] = sum(a[1], a[2]);
			}
		}
	}
	for(int i = 0; i < q; i++) {
		if(ans[i] != -1) {
			cout << ans[i] << endl;
		}
	}
}