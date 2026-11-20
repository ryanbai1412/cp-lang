
inline void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}
template <typename T>
struct PSA {
    int n;
    vector<T> psum;
    template <typename U>
    PSA(const vector<U>& arr) : n(sz(arr)), psum(n + 1) {
        for (int i = 0; i < n; i++) {
            psum[i + 1] = psum[i] + arr[i];
        }
    }
    T query(int l, int r) const {
        return psum[r] - psum[l];
    }
};
template <typename T>
bool on(T mask, int bit) {
    return (mask >> bit) & 1;
}
template <typename T>
vector<T> iota(int n, T x) {
    vector<T> arr(n);
    iota(begin(arr), end(arr), x);
    return arr;
}
template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}
int solve(const vector<int>& arr, const vector<int>& par, const vector<int>& ty) {
    int n = sz(arr);
    vector<int> w0(n), w1 = arr;
    for (int u = n - 1; u >= 1; u--) {
        int p = par[u];
        if (ty[u] == 0) {
            w0[p] += max(w0[u], w1[u]);
            w1[p] += w0[u];
        } else if (ty[u] == 1) {
            w1[p] = max(w1[p] + max(w0[u], w1[u]), w0[p] + w1[u]);
            w0[p] += w0[u];
        } else {
            w1[p] = max(w1[p] + w0[u], w0[p] + w1[u]);
            w0[p] += w0[u];
        }
    }
    return max(w0[0], w1[0]);
}
int findSample(int n, int confidence[], int host[], int protocol[]) {
    return solve(vector(confidence, confidence + n), vector(host, host + n),
                 vector(protocol, protocol + n));
}
#include <cstdio>
#include <cassert>
#define __MAXSIZE__ 100002
// Define
#ifndef __FRIENDHEAD__
#define __FRIENDHEAD__
// Find out answer
int findSample(int n,int confidence[],int host[],int protocol[]);
#endif
using namespace std;
namespace {
// Confidence
int confidence[__MAXSIZE__];
// Host
int host[__MAXSIZE__];
// Protocol
int protocol[__MAXSIZE__];
}
// Main
int main(void)
{
    int n,i;
    // Number of people
    assert(scanf("%d",&n)==1);
    // Confidence
    for(i=0;i<n;i++)
        assert(scanf("%d",&confidence[i])==1);
    // Host and Protocol
    for(i=1;i<n;i++)
        assert(scanf("%d %d",&host[i],&protocol[i])==2);
    // Answer
    printf("%d\n",findSample(n,confidence,host,protocol));
    return 0;
}