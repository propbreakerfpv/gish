#include <stdio.h>
#include <errno.h>
#include <stdlib.h>


int
main(int argc, char **argv)
{
    char *cmd = "./pinger";
    printf("Running '%s'\n", cmd);

    FILE *fp = popen(cmd, "r");
    if (!fp)
    {
        perror("popen failed:");
        exit(1);
    }

    printf("fp open\n");

    char inLine[1024];
    while (fgets(inLine, sizeof(inLine), fp) != NULL)
    {
        printf("Received: '%s'\n", inLine);
    }

    printf("feof=%d ferror=%d: %s\n", feof(fp), ferror(fp), strerror(errno));
    pclose(fp);
}
