#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("sse4,avx2")

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

mt19937 cowng(1412);

struct Node {
	Node *l = 0, *r = 0;
	int ix, iy, mxx, mxy, y, c = 1;
	long sum;
	Node(int x, int y) : ix(x), iy(y), mxx(x), mxy(y), y(int(cowng())), sum(x - y) {}
	void recalc();
	long key() {
		return (iy - ix) * long(2e9) + iy;
	}
};

int cnt(Node* n) {
	return n ? n->c : 0;
}
void Node::recalc() {
	c = cnt(l) + cnt(r) + 1;
	sum = ix - iy;
	mxx = ix;
	mxy = iy;
	if (l) {
		sum += l->sum;
		mxx = max(mxx, l->mxx);
		mxy = max(mxy, l->mxy);
	}
	if (r) {
		sum += r->sum;
		mxx = max(mxx, r->mxx);
		mxy = max(mxy, r->mxy);
	}
}

pair<Node*, Node*> split(Node* n, int k) {
	if (!n)
		return {};
	if (cnt(n->l) >= k) {  // "n->val >= k" for lower_bound(k)
		auto pa = split(n->l, k);
		n->l = pa.second;
		n->recalc();
		return {pa.first, n};
	} else {
		auto pa = split(n->r, k - cnt(n->l) - 1);  // and just "k"
		n->r = pa.first;
		n->recalc();
		return {n, pa.second};
	}
}

pair<Node*, Node*> split1(Node* n, long k) {
	if (!n)
		return {};
	if (n->key() >= k) {  // "n->val >= k" for lower_bound(k)
		auto pa = split1(n->l, k);
		n->l = pa.second;
		n->recalc();
		return {pa.first, n};
	} else {
		auto pa = split1(n->r, k);  // and just "k"
		n->r = pa.first;
		n->recalc();
		return {n, pa.second};
	}
}

Node* merge(Node* l, Node* r) {
	if (!l)
		return r;
	if (!r)
		return l;
	if (l->y > r->y) {
		l->r = merge(l->r, r);
		l->recalc();
		return l;
	} else {
		r->l = merge(l, r->l);
		r->recalc();
		return r;
	}
}

int bsearch(Node* n, int targ, long csm, int cmx) {
	if (!n) {
		return 1;
	}
	if (n->l) {
		long ncsm = csm + n->l->sum;
		int ncmx = max(cmx, n->l->mxy);
		if (ncsm + ncmx >= targ) {
			return bsearch(n->l, targ, csm, cmx);
		}
		csm = ncsm;
		cmx = ncmx;
	}
	csm += n->ix - n->iy;
	cmx = max(cmx, n->iy);
	if (csm + cmx >= targ) {
		return cnt(n->l);
	}
	return cnt(n->l) + 1 + bsearch(n->r, targ, csm, cmx);
}

struct DS {
	Node* root = nullptr;

	void insert(int x, int y) {
		auto n = new Node(x, y);
		if (!root) {
			root = n;
			return;
		}
		auto [l, r] = split1(root, n->key());
		root = merge(merge(l, n), r);
	}

	void erase(int x, int y) {
		auto [l, r] = split1(root, Node(x, y).key());
		root = merge(l, split(r, 1).second);
	}
	
	int bsearch(int ind, int targ) {
		auto [l, r] = split(root, ind);
		int ans = ::bsearch(root, targ, 0, 0);
		root = merge(l, r);
		return ans;
	}

	long q1(int ind) {
		auto [l, r] = split(root, ind);
		long ans = l ? l->sum : 0;
		ans += l->mxy;
		root = merge(l, r);
		return ans;
	}

	long q2(int ind) {
		auto [l, r] = split(root, ind);
		long ans = l ? l->sum : 0;
		ans += r ? r->mxx : 0;
		root = merge(l, r);
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
			auto valid1 = [&](int k) -> bool {
				return k <= n && ds.q1(k) >= x;
			};
			auto valid = [&](int k) -> bool {
				if (ds.q2(k - 1) >= x) {
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
			int ans = l;
			int nans = ds.bsearch(pos, x);
			if (nans < ans) {
				if (valid1(nans)) {
					ans = nans;
				}
			}
			if (!valid(l) && !valid1(l)) {
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