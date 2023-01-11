draw:
    ; Fist, draw the food
    LDA #FOOD_COLOUR
    LDX <FOOD_ADDR
    STA DISPLAY_ADDR, x

    ; Clear the previous tail position if CLEAR_ADDR != 0
    LDA <CLEAR_ADDR 
    BEQ .skip_clear
    LDX <PREVIOUS_TAIL_ADDR 
    LDA #0
    STA DISPLAY_ADDR, x

    ; Draw the body
    .skip_clear:
    ; Load the len into the X register
    LDX SNAKE_ADDR 
    .loop:
        ; Load the segment into the Y register
        LDY SNAKE_ADDR + 1, x
        ; Set the tile to the SNAKE colour
        LDA #SNAKE_COLOUR 
        STA DISPLAY_ADDR, y
        ; Decrement X
        DEX 
    ; Exit the loop if X = 0 meaning the next iteration would be the head which we update with the direction instead
    BNE .loop
    ; Draw the head
    LDY SNAKE_ADDR + 1
    LDA #HEAD_COLOUR 
    STA DISPLAY_ADDR, y
RTS 

; Zero initialises the display
clear:
    LDA #0
    LDX #0
    .loop:
        ; the screen clearing has a nice visual effect but at 1MHz you can't see it so I inserted this loop to make it visible again
        LDY #0x0f
        .._loop:
            DEY 
        BNE .._loop
        STA DISPLAY_ADDR, x
        INX 
        ; Exit the loop if zero is set meaning x overflowed, meaning the buffer has been fully initialised.
    BNE .loop
RTS 
    