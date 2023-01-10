update:
    .body:
        ; Update the body
        
        ; Load the len into the X register
        LDX SNAKE_ADDR 
        ; Load the tail position into the accumulator
        LDA SNAKE_ADDR + 1, x
        ; Put the tail into the previous tail addr so it can be cleared (or not if the snake ate an apple)
        STA PREVIOUS_TAIL_ADDR 
        ; Mark the tail position to be cleared
        LDA #1
        STA CLEAR_ADDR 
        ..loop:
            ; First, check for collisions.
            ; Load the current segment into the accumulator.
            LDA SNAKE_ADDR + 1, x
            ; Compare it with the snake head
            CMP SNAKE_ADDR + 1
            ; Break if equal
            BNE ...continue
            BRK 
            ...continue:
            ; Load the next segment into the accumulator
            ; Load SNAKE_ADDR + 1 (where the snake head is) + x - 1
            ; This is the same as loading SNAKE_ADDR + x
            LDA SNAKE_ADDR, x
            ; Store this in the current segment, this ensures every segment is set to the next one
            STA SNAKE_ADDR + 1, x  
            ; Decrement X
            DEX 
        ; Exit the loop if X = 0 meaning the next iteration would be the head which we update with the direction instead
        BNE ..loop
    .head:
        ; Next is updating the head of the snake, this involves matching against the direction (like a c switch statement)

        ; Split the position / load the position in the X and Y registers
        LDA SNAKE_ADDR + 1
        JSR split_pos
        ; load the direction
        LDA DIRECTION_ADDR

        ; Down is stored as zero, so if the zero flag was set, we jump to down
        BEQ ..down

        CMP #RIGHT
        BEQ ..right

        CMP #LEFT
        BEQ ..left
        ; it's not down, left or right, so it must be up (we don't need to jump)
        ..up:
            ; The Y axis is "inverted" meaning y=0 is the top of the screen and y=255 is the bottom of the screen, so we decrement the Y axis to go up. 
            DEY 
            ; Jump to the end so we don't fall through to the next case
            JMP ..break
        ..down:
            ; The Y axis is "inverted" meaning y=0 is the top of the screen and y=255 is the bottom of the screen, so we increment the Y axis to go down. 
            INY 
            ; Jump to the end so we don't fall through to the next case
            JMP ..break
        ..left:
            DEX 
            ; Jump to the end so we don't fall through to the next case
            JMP ..break
        ..right:
            INX 
        ..break:
        JSR combine_pos
        STA SNAKE_ADDR + 1
    ; Detect if we ate the food, if so, grow the snake and spawn new food
    .food:
        LDA <FOOD_ADDR
        CMP SNAKE_ADDR + 1 
        ; Return if the snake didn't eat the food
        BNE .ret
        ; Grow the snake
        ; Increment the len
        INC SNAKE_ADDR
        ; Load the new len into x
        LDX SNAKE_ADDR
        ; Load the previous tail position into the accumulator
        LDA PREVIOUS_TAIL_ADDR
        ; Grow the snake, set the new segment to the previous tail position
        STA SNAKE_ADDR + 1, x
        ; Make sure the tail position isn't cleared
        LDA #0
        STA CLEAR_ADDR
        ; spawn new food
        JSR spawn_food
.ret:
RTS 
