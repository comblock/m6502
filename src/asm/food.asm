spawn_food: 
    ; Load a random value into 0x10
    LDA <RANDOM_ADDR
    STA <0x10
    ; Load the snake len
    LDX SNAKE_ADDR
    ; Check if the food would spawn in the snake, if so, start over
    .loop:
        LDY SNAKE_ADDR + 1, x
        ; If the Y register is equal to the random value, this means the food would spawn in the snake, obviously we don't want this so we start over
        CPY <0x10
        BEQ spawn_food
        CPY <0x10
        DEX 
        ; Branch on underflow, this way I include the snake head (x=0)
        CPX #0xff
        BNE .loop
    ; Store the random value in the food address
    STA <FOOD_ADDR
RTS

