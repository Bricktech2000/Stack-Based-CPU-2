@ lib/core.asm
@ lib/types.asm
@ lib/stdlib.asm
@ lib/stdio.asm
@ lib/display.asm

main!
  pop pop !display_buffer sts

  # xF0 # rand_seed

  x77 !u4u4 # xy_pos
  x00 !i4i4 # xy_vel

  loop:
    # xy_pos += xy_vel
    !u8u8.ld0 !u4u4.add !u4u4.st1
    # invert pixel at xy_pos
    !u4u4.ld1 !display_buffer !bit_addr !flip_bit
    # sleep
    x20 !delay
    # input = getc()
    !getc
    # input = (1 << rand()) & 0x0F
    # ld2 !rand.min st2 x01 ld3 rot x0F and
    # ignore if input is empty
    x0F and :ignore !bcs
      # vel = (input & 0b1010) ? 0x0F : 0x01
      ld0 !primary_down !primary_right orr and pop x01 x0F iff !i4i4
      # rot = (input & 0b0011) ? 0x04 : 0x00
      ld1 !primary_up !primary_down orr and pop x04 x00 iff
      # xy_vel = vel << rot
      rot !i4i4.st1
    # pop input
    ignore: pop
  :loop !jmp
