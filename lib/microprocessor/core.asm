ror! neg rot
norr! orr not
nand! and not
xnorr! xor not
xnand! xnd not
abs! ld0 x01 rot x01 and neg swp ld1 add xor @const
abs_dyn! ld0 neg ld1 shl @dyn pop iff
min! ld1 ld1 sub @dyn pop flc iff
max! ld1 ld1 sub @dyn pop iff

jmp! sti
bcc_dyn! .skip iff !jmp skip.
bcc! @const !bcc_dyn
bcs_dyn! .skip swp iff !jmp skip.
bcs! @const !bcs_dyn
call_dyn! .ret swp !jmp ret.
call! @const !call_dyn
ret! !jmp
rt0! !ret
rt1! st0 !ret
rt2! st1 pop !ret
rt3! st2 pop pop !ret
rt4! st3 pop pop pop !ret

dbg! dBB # emulator treats unofficial `BB` as debug request
nop! nop @dyn
here! lbl. .lbl
hlt! !here !jmp
