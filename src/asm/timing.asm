; This function helps balance the time between updates, if the snake len is high, the update subroutine takes a very long time.
; To counteract this, the timing subroutinge takes less and less time as the snake len goes up
timing:
    ; Load the snake len
    LDA SNAKE_ADDR 
    ; NOT the snake len (which is the same as 255 - snake len)
    EOR #0xff
    ; Transfer to X register so we can use DEX
    TAX 
    .loop:
        ; This inner loop just waits some cycles.
        LDY #0x06
        .._loop:
            DEY 
        BNE .._loop
        ; Decrement accumulator
        DEX 
        ; Exit the loop on underflow
        CPX #0xff
    BNE .loop
RTS 