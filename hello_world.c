#include <stdio.h>

int world() {
    int a = 1 + (2 + 3);
    printf("a = %d", a);
}

// \( this should be ignored

int main() {
    printf("Hello, world()!\n");
    world();
}