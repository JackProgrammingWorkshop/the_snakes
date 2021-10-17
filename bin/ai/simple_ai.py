#!/usr/bin/env python3
import io
import math
import time
from sys import stderr
import logging

logging.basicConfig(level=logging.DEBUG, stream=stderr)

initializing = False
reading_map = False
player_id = None
snakes = []
foods = []


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


def latest_food(pos):
    latest = None
    radius = 1e5
    x0, y0 = pos
    for f in foods:
        x, y = f
        new_radius = ((x0 - x) ** 2 + (y0 - y) ** 2) ** 0.5
        if new_radius < radius:
            radius = new_radius
            latest = f
    return latest


def diff(p1, p2):
    return p1[0] - p2[0], p1[1] - p2[1]


def angle(v):
    import math
    return math.atan2(v[1], v[0])


def get_angle(head, body, food):
    d1 = angle(diff(head, body))
    d2 = angle(diff(food, head))
    d = d2 - d1
    return d


def get_command():
    snake = [x for x in snakes if x.player_id == player_id][0]
    food = latest_food(snake.segments[0])
    if food:
        direction_diff = get_angle(snake.segments[0], snake.segments[1], food)

        # logging.info('direction %s', direction_diff)
        if abs(direction_diff) < 0.1:
            return "straight"
        elif 0 < direction_diff < math.pi:
            return "turn_left"
        else:
            return "turn_right"
    return "straight"


def main():
    global initializing, player_id, reading_map
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
            print_line("username simple_ai")
        elif command.startswith("player_id"):
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
            print_line(get_command())
            # more action
        else:
            logging.warning("Could not process %s", command)


if __name__ == '__main__':
    main()
