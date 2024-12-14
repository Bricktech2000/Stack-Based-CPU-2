# constructive quine. data buffer `str` is generated by passing every line of
# code up to `str:` through 'stringify.asm' then joining the outputs with `@0A`

@ lib/core.asm
@ lib/types.asm
@ lib/stdio.asm
main! :str !puts :str !'@' !hex_puts !hlt str: @40 @20 @6C @69 @62 @2F @63 @6F @72 @65 @2E @61 @73 @6D @0A @40 @20 @6C @69 @62 @2F @74 @79 @70 @65 @73 @2E @61 @73 @6D @0A @40 @20 @6C @69 @62 @2F @73 @74 @64 @69 @6F @2E @61 @73 @6D @0A @6D @61 @69 @6E @21 @20 @3A @73 @74 @72 @20 @21 @70 @75 @74 @73 @20 @3A @73 @74 @72 @20 @21 @27 @40 @27 @20 @21 @68 @65 @78 @5F @70 @75 @74 @73 @20 @21 @68 @6C @74 @20 @73 @74 @72 @3A @00
