#include "6502.asm"
#include "addrs.asm"
#include "direction.asm"
#include "lib.asm"

#bankdef main {
    #addr 0x0200
    #size 0x0800
    #outp 0
    #fill
}

main:
    ; First, I initialise the snake.
    ; The snake starts out with a len of 4

    ; The len
    ; The len is stored as the real len - 1 because it will never be 0 anyways.
    LDA #0x03
    STA SNAKE_ADDR

    ; The body
    LDA #0x03
    STA SNAKE_ADDR + 1
    LDA #0x02
    STA SNAKE_ADDR + 2
    LDA #0x01
    STA SNAKE_ADDR + 3
    LDA #0x00
    STA SNAKE_ADDR + 4

    ; The direction
    LDA #DOWN
    STA DIRECTION_ADDR
    
    ; Zero initialise the display buffer
    JSR clear

    ; Spawn the food
    JSR spawn_food

    ; The main loop
    .loop:
        JSR update
        JSR draw
        JSR timing
    JMP .loop

