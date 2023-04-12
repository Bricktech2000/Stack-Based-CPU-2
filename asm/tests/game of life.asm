# to count neighbours, front buffer is read from and back buffer is written to.
# back buffer is copied to front buffer at the end of each iteration.
#
# rules used:
#
# ```rust
# let next_state = match neighbour_count {
#   3 => State::Alive,
#   4 => current_state,
#   _ => State::Dead,
# }
# ```

@ ../../lib/bit.asm
@ ../../lib/core.asm
@ ../../lib/memcpy.asm
@ ../../lib/microcomputer/core.asm

main!
  !back_buffer !alloc_buffer

  loop:
    # copy back buffer to front buffer.
    !front_buffer !back_buffer sub @const !back_buffer !front_buffer :memcpy !call
    # loop through every cell
    x00 for_xy: dec
      x00 # allocate neighbour count

      # count neighbours
      x02 for_dx: dec
        x20 for_dy: x10 sub
          # neighbour_addr = (for_xy + for_dx) & 0x0F | (for_xy + for_dy) & 0xF0
          # neighbour_value = load_bit(neighbour_addr, &FRONT_BUFFER)
          !front_buffer ld4 ld3 add x0F and ld5 ld3 add xF0 and orr :load_bit !call
          # neighbour_count += neighbour_value
          ld3 add st2
        ld0 xF0 xor pop :for_dy !bcc pop
      ld0 xFF xor pop :for_dx !bcc pop

      # apply rules outlined above
      ld0 x04 xor pop :ignore !bcs
      ld0 x03 xor pop x00 x01 iff !back_buffer ld3 :store_bit !call
      ignore:

      pop # pop neighbour count
    buf :for_xy !bcc pop
  :loop sti

  !memcpy
  !bit_addr
  !load_bit
  !store_bit

  # !glider
  # !blinker
  # !r-pentomino
  # !lightweight_spaceship
  !heavyweight_spaceship
  # !copperhead
  # !diehard
  # !compact_pulsar


blinker!
  !back_buffer x0C add @org
  d07 d00

glider!
  !back_buffer x0C add @org
  d07 d00
  d01 d00
  d02 d00

diehard!
  # already advanced 2 generations
  !back_buffer x0C add @org
  d30 d80
  d31 dC0

r-pentomino!
  !back_buffer x0C add @org
  d06 d00
  d0C d00
  d04 d00

lightweight_spaceship!
  !back_buffer x0A add @org
  d00 d09
  d00 d10
  d00 d11
  d00 d1E

heavyweight_spaceship!
  !back_buffer x0A add @org
  d00 d0C
  d00 d21
  d00 d40
  d00 d41
  d00 d7E

compact_pulsar!
  # pattern that turns into a pulsar
  !back_buffer x0C add @org
  d07 dC0
  d08 d40
  d07 dC0

copperhead!
  !back_buffer x08 add @org
  d06 d60
  d01 d80
  d01 d80
  d0A d50
  d08 d10
  d00 d00
  d08 d10
  d06 d60
  d03 dC0
  d00 d00
  d01 d80
  d01 d80
