stdin! x00 @const
stdout! x00 @const

fgetc! lda # char = fgetc(stream)
getc! !stdin !fgetc # char = getc()
fgetc.def! fgetc: @err # to be implemented
getc.def! getc: !getc swp !ret # char = getc()
fgets! # fgets(stream, *str)
  swp for_c.
    ld1 !fgetc
    !char.check_null
    ld1 sta
  inc .for_c !bcc pop pop
gets! !stdin !fgets # gets(*str)
fgets.def! fgets: swp ld2 swp !fgets !rt1 # fgets(stream, *str)
gets.def! gets: swp !gets !ret # gets(*str)

gets.min! # gets.min(*str)
  for_c.
    !getc
    !char.check_null
    ld1 sta
  inc .for_c !bcc pop
gets.min.def! gets.min: swp !gets.min !ret # gets.min(*str)

fputc! sta # fputc(stream, char)
putc! !stdout !fputc # putc(char)
fputc.def! fputc: @err # to be implemented
putc.def! putc: swp !putc !ret # putc(char)
fputs! # fputs(stream, *str)
  swp for_c.
    ld0 lda
    !char.check_null
    ld2 !fputc
  inc .for_c !bcc pop pop
puts! !stdout !fputs # puts(*str)
fputs.def! fputs: swp ld2 swp !fputs !rt1 # fputs(stream, *str)
puts.def! puts: swp !puts !ret # puts(*str)

puts.min! # puts.min(*str)
  for_c.
    ld0 lda
    !char.check_null
    !putc
  inc .for_c !bcc pop
puts.min.def! puts.min: swp !puts.min !ret # puts.min(*str)
