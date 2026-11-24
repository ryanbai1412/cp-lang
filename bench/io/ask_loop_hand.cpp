// ask_loop.cp by hand: one query per line, flushed before each answer is read.
#include <cstdio>
int main() {
    long long s = 0;
    for (int i = 0; i < 100000; i++) {
        printf("? %d\n", i);
        fflush(stdout);
        long long x;
        if (scanf("%lld", &x) != 1) return 1;
        s += x;
    }
    printf("! %lld\n", s);
}
