#include <string.h>
#include <stdio.h>
#include <unistd.h> 

int
main(int argc, char **argv)
{
    int i = 0;
    while (1)
    {
        printf("stdout %d\n", i++);
        fflush(stdout);
        sleep(1);
    }
}
