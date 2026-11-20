#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

#include "bits/extc++.h"

using namespace std;

using u32 = uint32_t;
using u64 = uint64_t;

inline u32 next_integer(u32 x) {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

bool output_arr(void *a, u32 size) {
    if (size % 4) {
        return puts("-1"), 0;
    }

    u32 blocks = size / 4;
    u32 *A = (u32 *)a;
    u32 ret = size;
    u32 x = 23333333;
    for (u32 i = 0; i < blocks; i++) {
        ret = ret ^ (A[i] + x);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
    }

    return printf("%u\n", ret), 1;
}

// ===== header ======

namespace Sorting {
void init_data(u32 *a, int n, u32 seed) {
    for (int i = 0; i < n; i++) {
        seed = next_integer(seed);
        a[i] = seed;
    }
}

void main() {
    int n;
    u32 seed;
    scanf("%d%u", &n, &seed);

    const int m = 8, msz = 1 << m, msk = msz - 1;
    u32 arr[2][n];
    int cind = 0, pind = 1;
    init_data(arr[cind], n, seed);

    for (int i = 0; i < 32; i += m) {
        swap(cind, pind);
        int cnt[msz + 1] {};
        for (int j = 0; j < n; j++) {
            cnt[(arr[pind][j] >> i) & msk]++;
        }
        copy(cnt, cnt + msz, cnt + 1);
        cnt[0] = 0;
        for (int j = 1; j < msz; j++) {
            cnt[j] += cnt[j - 1];
        }
        for (auto &a: arr[pind]) {
            arr[cind][cnt[(a >> i) & msk]++] = a;
        }
    }

    output_arr(arr[cind], n * sizeof(u32));
}
}

namespace Game {
void main() {
    int n, q;
    scanf("%d%d", &n, &q);

    char *s1 = new char[n + 1];
    char *s2 = new char[n + 1];
    scanf("%s%s", s1, s2);

    u32 *anss = new u32[q];
    int *q_x = new int[q];
    int *q_y = new int[q];
    int *q_len = new int[q];

    for (int i = 0; i < q; i++) {
        scanf("%d%d%d", q_x + i, q_y + i, q_len + i);
    }

    // solve(n, q, s1, s2, q_x, q_y, q_len, anss);

    output_arr(anss, q * sizeof(u32));
}
}

namespace Parentheses {
void main() {
    int n;
    scanf("%d", &n);

    char *s = new char[n + 1];
    scanf("%s", s);

    u32 ans;
    // ans = solve(n, s);

    printf("%u\n", ans);
}
}

int main() {
    int task_id;
    scanf("%d", &task_id);

    switch (task_id) {
        case 1:
            Sorting::main();
            break;
        case 2:
            Game::main();
            break;
        case 3:
            Parentheses::main();
            break;
    }

    return 0;
}