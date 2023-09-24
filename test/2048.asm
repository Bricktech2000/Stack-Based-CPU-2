@ lib/core.asm
@ lib/types.asm
@ lib/stdlib.asm
@ lib/stdio.asm
@ lib/display.asm
@ lib/controller.asm

# this implementation differs from the original game in a few ways:
# - numbers on tiles are the base-2 logarithm of the number on the original -- we only have one character per tile
# - implementation has `0`-tiles (would be equivalent to `1`-tiles in the original) -- ensures only educated people can play
# - implementation has greedy merge, meaning tiles are merged multiple times within a single move -- simplest way to implement
# - new tiles are generated regardless of whether the board has changed -- currenttly not enough memory to perform the check
# - only `0`-tiles (equivalent to `2`-tiles in the original) are generated -- currently not enough memory for generating `1`-tiles
#
# when no room is left for a new tile, the program stalls in an infinite loop, which indicates the game is over

main!
  pop pop !display_buffer sts

  !primary_up # direction

  while:
    # iteration count must be no less than x03.
    # a higher iteration count serves as a stall
    x04 for_iteration: dec
      # refer to C implementation for this main loop
      x10 for_n: dec
        # the fourth `case` is not needed if we set `default` to its value
        x01 neg @const # offset
        !primary_up xo4 x04 nop @const iff !primary_up xo4
        !primary_down xo4 x04 neg @const iff !primary_down xo4
        !primary_left xo4 x01 nop @const iff !primary_left xo4
        # !primary_right xo4 x01 neg @const iff !primary_right xo4
        ld3 !primary_up !primary_left orr @const and pop
        x0F x00 iff # negator
        x03 x00 iff # equality
        # `negator ^= n` to produce `curr`
        ld3 xo2
        ld5 !primary_up !primary_down orr @const and pop
        x06 x00 iff # orientation
        # if ((curr >> orientation & 0x03) == equality) continue;
        ld2 swp rot x03 and xor pop :continue !bcs
        # curr = &board + curr
        :board add
        # `offset += curr` to produce `prev`
        ld0 ad2

        ld1 lda # board[prev]
        ld1 lda # board[curr]

        !check_zero :zero !bcs
        xor pop :equal !bcs
        :continue !jmp

        zero: # `board[prev]` and `board[curr]` on the stack
          # board[curr] = board[prev] - 1
          pop dec ld1 sta
        equal: # `board[prev]` and `board[curr]` popped off the stack
          # board[curr] = board[curr] + 1
          ld0 lda inc ld1 sta
          # board[prev] = 0
          x00 ld2 sta
          # `prev` and `curr` are bled onto the stack for `continue` below,
          # which happens to pop two bytes off the stack

        continue: pop pop
      !check_zero :for_n !bcc pop

      # if we're at the last iteration, generate a `0x01` tile.
      # otherwise, generate a `0x00` tile, which is a no-op
      !check_zero x00 shl @dyn
      # poke into the display buffer and `xor` two randomly chosen
      # bytes to produce a random value
      ld4 ldF xor

      # keep adding `&board` to the random byte, modulo `0x10` to prevent
      # out-of-bounds access, until we find a zero tile. the cycle length
      # of this operation is `0x10` if and only if `&board & 0x0F` is
      # coprime with `0x10`. this is guaranteed at assembly time. see below
      generate:
        x0F and clc :board add
      ld0 lda !is_zero :generate !bcc sta

      !display_buffer_len for_byte: dec
        x00 # result
        x00 for_nibble:
          # tile = board[&board + byte]
          :board ld3 x06 not @const and clc shr x00 shl @dyn xFF xo4 shl @dyn orr clc add lda
          # 2048_char = 2048_chars[tile]
          :2048_chars add lda
          # nibble = (2048_char << (byte & 0x06}) & 0x60
          # we use `0x60` as a mask to center the character horizontally.
          # this is also why `shl @const` is used in `2048_chars` below
          ld3 x06 and rot x60 and
          # result |= nibble
          # result <<= 4
          or2 x04 ro2
        !check_zero :for_nibble !bcc pop
        # display_buffer[byte] = result
        !display_buffer dec @const ld2 add sta
      !check_zero :for_byte !bcc pop

    !check_zero :for_iteration !bcc pop

    # the controller being in any state other than `0x01 | 0x02 | 0x04 | 0x08`
    # unfortunately breaks the game logic. not much we can do about that
    pop !block_getc # direction
  :while !jmp

  # characters below are 2x4 pixels in size. the bits of their encoding from MSB to LSB correspond
  # to the pixels of the font from left to right, top to bottom. the padding for the font is at the
  # top of the character so that the first bit is always clear so that it fits in an `IMM`
  2048_chars:
    x00 shl @const #
    x0F shl @const # 0
    x15 shl @const # 1
    x1E shl @const # 2
    x37 shl @const # 3
    x2D shl @const # 4
    x36 shl @const # 5
    x1B shl @const # 6
    x35 shl @const # 7
    x3F shl @const # 8
    # x3D shl @const # 9
    # x27 shl @const # A
    # x2F shl @const # B
    # x19 shl @const # C
    # x1F shl @const # D
    # x29 shl @const # E
    # x1A shl @const # F

  # `&board & 0x0F` must be coprime with `0x10` for random number generation.
  # `x` is coprime with `0x10` if and only if `x` is odd. therefore we ensure
  # that `&board & 0x0F` is odd by setting its least significant bit to `1`
  !here x01 orr @org
  board:
    @00 @00 @00 @00
    @00 @00 @00 @00
    @00 @00 @00 @00
    @00 @00 @00 @00
