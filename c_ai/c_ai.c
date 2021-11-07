#include <stdio.h>
#include <string.h>

struct Position {
    float x;
    float y;
};
// HTTP Request -> CGI -> HTTP Response

int main() {
    setbuf(stdout, NULL);

    // stdin, stdout, stderr
    char buffer[1000];
    int initializing = 0;
    int player_id = -1;
    int reading_map = 0;
    struct Position snakes[10][100] = {};
    int snake_len[10] = {};

    struct Position food[100] = {};
    int food_len = 0;

    while (1) {
        char *line = fgets(buffer, 1000, stdin);
        if (line == NULL) // End of File Ctrl + D
            break;
        line = strtok(line, "\n");
//        fprintf(stderr, "\"%s\"\n", line);
        if (strcmp(line, "INIT BEGIN") == 0) {
            initializing = 1;
        } else if (strcmp(line, "INIT END") == 0) {
            initializing = 0;
            printf("username c_ai\n");
        } else if (strstr(line, "player_id") == line) {
            sscanf(line + strlen("player_id "), "%d", &player_id);
        } else if (strcmp(line, "MAP BEGIN") == 0) {
            reading_map = 1;
            food_len = 0;
            for (int i = 0; i < 10; ++i) {
                snake_len[i] = 0;
            }
        } else if (strcmp(line, "MAP END") == 0) {
            reading_map = 0;
        } else if (strstr(line, "snake") == line) {
            int snake_id;
            int pos;

            sscanf(line, "snake %d %n", &snake_id, &pos);
            int read_n;
            float x, y;
            while (sscanf(line + pos, "(%f,%f)%n", &x, &y, &read_n) != EOF) // End of file
            {
                pos += read_n;
                if (snake_len[snake_id] < 100) {
                    snakes[snake_id][snake_len[snake_id]].x = x;
                    snakes[snake_id][snake_len[snake_id]].y = y;
                    snake_len[snake_id] += 1;
                }
            }
            // 24 bytes, important data
            // x86-64
            // 4KB
            // stack, 400 bytes, 400000 = 400KB
            //

            fprintf(stderr, "read snake id %d %d\n", snake_id, snake_len[snake_id]);
        } else if (strstr(line, "food") == line) {
            int pos;
            sscanf(line, "food %n", &pos);
            int read_n;
            float x, y;
            while (sscanf(line + pos, "(%f,%f)%n", &x, &y, &read_n) != EOF) // End of file
            {
                pos += read_n;
                if (food_len < 100) {
                    food[food_len].x = x;
                    food[food_len].y = y;
                    food_len += 1;
                }
            }
            fprintf(stderr, "read food %d\n", food_len);
        } else if (strcmp(line, "REQUEST_ACTION") == 0) {
            printf("straight\n");
        } else {
            fprintf(stderr, "Cannot recognize %s", line);
        }
    }
    fprintf(stderr, "Program ends!\n");
    return 0;
}