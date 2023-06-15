#include <stdio.h>
#include <unistd.h>

int main(int argc, char *argv[]){
    for (int i = 0; i < 53; ++i) {
        printf("%i\n", i);
    }
    printf("\x1b[53Atest");
    printf("\x1b[52Btest2");
    
    sleep(1);
    return 0;
}
