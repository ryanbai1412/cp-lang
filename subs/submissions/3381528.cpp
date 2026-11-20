#pragma GCC optimize("Ofast,O3,no-stack-protector,unroll-loops,fast-math")
#pragma GCC target("sse,sse2,sse3,ssse3,sse4,popcnt,abm,mmx,avx,avx2,fma")

#include <bits/stdc++.h>

using namespace std;

const int maxn = 2e5;

int n, ind, x;
int arr[maxn], v[maxn*4];

void build(int o, int l, int r) {
	if(l+1==r) {
		v[o] = arr[l];
	}else {
		int mid = (l+r)/2, lc = o*2, rc = lc+1;
		build(lc, l, mid);
		build(rc, mid, r);
		v[o] = min(v[lc], v[rc]);
	}
}

void update(int o, int l, int r) {
	if(l+1==r) {
		v[o] = x;
	}else {
		int mid = (l+r)/2, lc = o*2, rc = lc+1;
		if(ind<mid) {
			update(lc, l, mid);
		}else {
			update(rc, mid, r);
		}
		v[o] = min(v[lc], v[rc]);
	}
}

void update(int ind, int x) {
	::ind = ind;
	::x = x;
	update(1, 0, n);
}

void query(int o, int l, int r) {
	if(l+1==r) {
		ind = l;
	}else {
		int mid = (l+r)/2, lc = o*2, rc = lc+1;
		if(v[lc]<=x) {
			query(lc, l, mid);
		}else {
			query(rc, mid, r);
		}
	}
}

int query(int x) {
	::x = x;
	query(1, 0, n);
	return ind;
}

signed main() {
	cin.tie(0)->sync_with_stdio(0);
	int k;
	cin>>n>>k;
	deque<int> ind[k];
	for(int i = 0; i<n; i++) {
		cin>>arr[i];
		ind[--arr[i]].push_back(i);
	}
	pair<int, int> last[k];
	for(int i = 0; i<k; i++) {
		last[i] = {*prev(ind[i].end()), i};
	}
	sort(last, last+k);
	bool removed[k];
	memset(removed, 0, sizeof(removed));
	build(1, 0, n);
	bool first = true;
	int prev = 0;
	for(auto& i: last) {
		if(!removed[i.second]) {
			while(true) {
				int cur = query(i.first);
				if(!first) {
					cout<<" ";
				}
				first = false;
				cout<<cur+1;
				removed[cur] = true;
				update(cur, n+1);
				deque<int> tmp;
				int time = *ind[cur].begin();
				for(int j = prev; j<=time; j++) {
					ind[arr[j]].pop_front();
					if(!removed[arr[j]]) {
						if(ind[arr[j]].empty()) {
							update(arr[j], n+1);
						}else {
							update(arr[j], *ind[arr[j]].begin());
						}
					}
				}
				prev = time+1;
				if(cur==i.second) {
					break;
				}
			}
		}
	}
	cout<<"\n";
}