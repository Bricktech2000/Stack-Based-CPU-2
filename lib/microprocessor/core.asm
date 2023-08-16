rneg! xF8 orr neg # prepares for rotate right
norr! orr not
nand! and not
xnorr! xor not
xnand! xnd not
abs! ld0 x01 rot x01 and neg swp ld1 add xor @const
abs.dyn! ld0 neg ld1 shl @dyn pop iff
min.dyn! ld1 ld1 sub @dyn pop flc iff
max.dyn! ld1 ld1 sub @dyn pop iff

jmp! sti
bcc! @const !bcc.dyn
bcc.dyn! .skip iff !jmp skip.
bcs! @const !bcs.dyn
bcs.dyn! .skip swp iff !jmp skip.
call! @const !call.dyn
call.dyn! .ret swp !jmp ret.
ret! !jmp
rt0! !ret
rt1! st0 !ret
rt2! st1 pop !ret
rt3! st2 pop pop !ret
rt4! st3 pop pop pop !ret

dbg! @BB # unofficial `BB` treated as debug request
nop! nop @dyn
here! lbl. .lbl
hlt! !here !jmp
pad! .lbl add lbl. @org
