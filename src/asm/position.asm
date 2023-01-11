; Positions are stored as bytes. 
; The low nybble is used to represent the x axis and the high nybble is used to represent the y axis.

; Combines an X and a Y coordinate into one byte.
; This function assumes the X register holds the X position and the Y register holds the Y position.
; The result is placed in the accumulator
combine_pos:
    ; Store X position at 0x10 to use AND
    STX <0x10
    ; Make sure the X position does not contaminate the Y position by using the AND operation.
    LDA #0x0f 
    AND <0x10
    ; Store the sanitised x position at <0x10
    STA <0x10
    TYA 
    ; Shift the Y position to the high nybble
    ASL a
    ASL a
    ASL a
    ASL a
    
    ; Finally use OR to combine the positions
    ORA <0x10
RTS 

; Splits a single byte position into two bytes.
; This function assumes the position is stored in the Accumulator
; The result is placed in the X and Y registers
split_pos:
    ; Transfer accumulator to Y to preserve the value
    TAY 
    AND #0x0f
    ; The result of the AND operation is the X coordinate and it's placed inside of the X register
    TAX 
    ; Put the initial value back in the accumulator
    TYA 
    ; Shift the position into the low nybble to extract the Y value
    LSR a
    LSR a
    LSR a
    LSR a
    ; Put the Y value in the Y register
    TAY 
RTS 
