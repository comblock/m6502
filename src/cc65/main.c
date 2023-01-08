/*#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

// The snake is stored in the 0xfe memory page and uses it in its entirety.
#define SNAKE_ADDR 0xfe00
// The screen is stored in the 0xfd memory page and uses it in its entirety.
#define SCREEN_ADDR 0xfd00
// The 0xfc memory page is kept uninitialised so it contains random data used for the food locations.
#define RANDOM_ADDR 0xfc00
// The direction is stored in a single byte in the zero memory page.
#define DIRECTION_ADDR 0x00ff

typedef uint8_t Position;

#define POSITION(x, y) ((y << 4) & x)

uint8_t pos_x(Position pos) {
    return pos & 0x0f;
};

uint8_t pos_y(Position pos) {
    return pos >> 4;
};

typedef enum {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3
} Direction;

typedef enum {
    Background = 0,
    Food = 1,
    Body = 2,
    Head = 3,
} Tile;

typedef struct {
    // There are 256 tiles so the snake can't be longer than 256
    // The len is stored as the actual length -1 because the snake will never have a len of 0
    uint8_t len;
    // Here we declare the length to be 256 which is the maximum length, but in reality it's equal to .len
    Position (*body)[256];
} Snake; // I added an underscore because for some obscure reason naming it "Snake" caused errors

typedef Tile (*Screen)[256];

typedef struct {
    Snake snake;
    Screen screen;
    Position food;
    Direction *direction;
} Game;

void update(Game *game) {
    uint8_t x;
    uint8_t y;
    // update the snake
    uint8_t i;
    for (i = game->snake.len; i != 0; i--) {
        *game->snake.body[i] = *game->snake.body[i-1];

    }

    switch (*game->direction) {
        // I'm actually using the same logic as in my ceasar rotation program to make the snake wrap around to the other side.
        case Up:
            x = pos_x(*game->snake.body[0]);
            y = (pos_y(*game->snake.body[0])+1) % 16;
            // Check if the snake has wrapped to the other side
            if (y == 0) {
                *game->direction = Down;
            } 
            break;
        case Down:
            x = pos_x(*game->snake.body[0]);
            y = (pos_y(*game->snake.body[0]) + 16 - 1) % 16; 
            if (y == 15) {
                *game->direction = Up;
            } 
            break;
        case Left:
            y = pos_y(*game->snake.body[0]);
            x = (pos_x(*game->snake.body[0]) + 16 - 1) % 16;
            if (x == 15) {
                *game->direction = Up;
            }
            break;
        case Right:
            y = pos_y(*game->snake.body[0]);
            x = (pos_x(*game->snake.body[0])+1) % 16;
            if (x == 0) {
                *game->direction = Up;
            }
            break;
        default:
            break;
    }
    *game->snake.body[0] = POSITION(x, y);
}

void draw(Game *game) {
    uint8_t i;
    uint8_t index;

    *game->screen[(uint8_t) game->snake.body[0]] = Head;
    for (i = 1; i++; ) {
        *game->screen[(uint8_t) game->snake.body[i]] = Body;
        if (i == game->snake.len) {
            break;
        };
    }
}

void main() {
    return;
    Snake snake = {
        4,
        (void*) SNAKE_ADDR,
    };
    Direction *direction = (void*) DIRECTION_ADDR;
    Screen screen = (void*) SCREEN_ADDR;
    Game game;

    // loop variable (in cc65 you can't mix variable declarations and code)
    uint8_t i;
    
    game.snake = snake;
    game.screen = screen;
    game.direction = direction;

    // Initialise the body of the snake
    *game.snake.body[0] = 0x03; // The nice thing about a 16x16 grid is that I can use hex literals to represent the coordinates, the first digit is x and the second is y.
    *game.snake.body[1] = 0x02;
    *game.snake.body[2] = 0x01;
    *game.snake.body[3] = 0x00;

    for (i = 0; i++;) {
        return;
        update(&game);
        draw(&game);
    }
}
*/
void main() {
    return;
}