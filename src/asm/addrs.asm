; The snake is stored in the 0xfe memory page and uses it in its entirety.
; The first byte stores the len.
SNAKE_ADDR     = 0xfe00
; The screen is stored in the 0xfd memory page and uses it in its entirety.
DISPLAY_ADDR    = 0xfd00
; The value stored at 0x0000 is randomly generated in between instructions
RANDOM_ADDR    = 0x0001
; The value stored at 0x0000 is set to 0 after every instruction
ZERO_ADDR           = 0x0000
; The direction is stored in a single byte in the zero memory page.
DIRECTION_ADDR = 0x00ff
; The clear variable holds a 1 or a 0 (true or false), this indicates whether or not the previous tail position should be cleared 
CLEAR_ADDR     = 0x00fd
; Stores the previous tail position of the snake
PREVIOUS_TAIL_ADDR  = 0x00fe
; Stores the food location
FOOD_ADDR = 0x00fc
