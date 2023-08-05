stdin! x00 @const
stdout! x00 @const

primary_up! x01 @const
primary_down! x02 @const
primary_left! x04 @const
primary_right! x08 @const
secondary_up! x10 @const
secondary_down! x20 @const
secondary_left! x40 @const
secondary_right! x80 @const

fgetc! lda # char = fgetc(stream)
getc! !stdin !fgetc # char = getc()
fgets! @err # to be implemented
gets! @err # to be implemented
fgets.def! fgets: @err # to be implemented
gets.def! gets: @err # to be implemented

gets.min! @err # to be implemented
gets.min.def! @err # to be implemented

fputc! sta # fputc(stream, char)
putc! !stdout !fputc # putc(char)
fputs! # fputs(stream, *str)
  swp for_c.
    ld0 lda
    !char.null xor
    ld2 !fputc inc
  .for_c !bcc pop pop
puts! !stdout !fputs # puts(*str)
fputs.def! fputs: swp ld2 swp !fputs !rt1 # fputs(stream, *str)
puts.def! puts: swp !puts !ret # puts(*str)

puts.min! # puts.min(*str)
  for_c.
    ld0 lda
    !char.null xor
    !putc inc
  .for_c !bcc pop
puts.min.def! puts.min: swp !puts.min !ret # puts.min(*str)

wait_char! @const !wait_char.dyn
wait_char.dyn! .skip swp !getc !char.null xor pop iff !jmp skip.
wait_null! @const !wait_null.dyn
wait_null.dyn! .skip !getc !char.null xor pop iff !jmp skip.
