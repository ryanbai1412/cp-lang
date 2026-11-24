// The judge's side for the accepted C++, whose own `main` reads the input and
// calls SocialEngineering: GetMove is `?` and Maria's answer, MakeMove prints
// the vertex, and the function returning (the program ending) is `!`.
#include <iostream>

int GetMove() {
    std::cout << "?" << std::endl;
    int v;
    std::cin >> v;
    return v;
}

void MakeMove(int v) {
    std::cout << v << std::endl;
}

static struct Returned {
    ~Returned() { std::cout << "!" << std::endl; }
} returned;
