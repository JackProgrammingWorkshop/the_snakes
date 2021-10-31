#include <stdio.h>
#include <string.h>

int main() {
    setbuf(stdout, NULL);

    // stdin, stdout, stderr
    char buffer[1000];
    int initializing = 0;
    int player_id = -1;
    int reading_map = 0;
    while (1) {
        char *line = fgets(buffer, 1000, stdin);
        if (line == NULL) // End of File Ctrl + D
            break;
        line = strtok(line, "\n");
        fprintf(stderr, "\"%s\"\n", line);
        if (strcmp(line, "INIT BEGIN") == 0) {
            fprintf(stderr, "INIT BEGIN\n");
            initializing = 1;
        } else if (strcmp(line, "INIT END") == 0) {
            fprintf(stderr, "INIT END\n");
            initializing = 0;
            printf("username c_ai\n");

        } else if (strstr(line, "player_id") == line) {
            sscanf(line + strlen("player_id "), "%d", &player_id);
        } else if (strcmp(line, "MAP BEGIN") == 0) {
            reading_map = 1;
            // TODO
        } else if (strcmp(line, "MAP END") == 0) {
            reading_map = 0;
        } else if (strstr(line, "snake") == line) {
            // TODO
        } else if (strstr(line, "food") == line) {
            // TODO
        } else if (strcmp(line, "REQUEST_ACTION") == 0) {
            printf("straight\n");
        } else {
            fprintf(stderr, "Cannot recognize %s", line);
        }
    }
    fprintf(stderr, "Program ends\n");
    return 0;
}