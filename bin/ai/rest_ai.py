#!/usr/bin/env python3
import io
from sys import stderr
import logging

logging.basicConfig(level=logging.DEBUG, stream=stderr)


class Snake:
    def __init__(self, player_id):
        self.player_id = player_id
        self.segments = []


def parse_pos(s):
    if s.startswith("("):
        s = s[1:]
    if s.endswith(")"):
        s = s[:-1]
    x, y = tuple(s.split(","))
    return float(x), float(y)


def read_line():
    command = input()
    logging.debug('< %s', command)
    return command


def print_to_string(*args, **kwargs):
    output = io.StringIO()
    print(*args, file=output, **kwargs)
    contents = output.getvalue()
    output.close()
    return contents


def print_line(*args, **kwargs):
    content = print_to_string(*args, **kwargs)
    print(content, end='')
    logging.debug('> %s', content)


if __name__ == '__main__':
    initializing = False
    reading_map = False
    player_id = -1
    snakes = []
    foods = []
    while True:
        try:
            command = read_line()
        except EOFError:
            logging.info("program shutdowns")
            break
        if command == "INIT BEGIN":
            initializing = True
        elif command == "INIT END":
            initializing = False
            print_line("username rest_ai")
        elif command == "player_id":
            player_id = int(command.split()[1])
        elif command == "MAP BEGIN":
            reading_map = True
            snakes.clear()
            foods.clear()
        elif command == "MAP END":
            reading_map = False
        elif command.startswith("snake"):
            player_id, *segments = command.split()[1:]
            snake = Snake(player_id)
            snake.segments.extend([parse_pos(x) for x in segments])
            snakes.append(snake)
        elif command.startswith("food"):
            pos = command.split()[1]
            foods.append(parse_pos(pos))
        elif command.startswith("REQUEST_ACTION"):
            print_line("straight")
        else:
            logging.warning("Could not process %s", command)
