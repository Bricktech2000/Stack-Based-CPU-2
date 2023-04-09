# Asm

Optimizing assembler for Atto-8 microprocessor

## Overview

Assembly consists of following operations:

1. Preprocess then tokenize source code from file `argv[1]`.
2. Expand macros recursively from entry point `"main"`.
3. Identify expressions that can be evaluated at compile time.
4. Convert IR to list of instructions by evaluating expressions.
5. Generate binary and write to file `argv[2]`.

Code generation adheres to microprocessor specification as defined in [/spec/microprocessor.md](../spec/microprocessor.md).

Labels are global by default; local labels are local to a macro. Macros are global. Macro definitions end either at the start of the next macro definition or at the end of the token stream. The token stream must start with a macro definition so every token belongs to a macro. Tokens are to be seperated by whitespace.

## Preprocessing

| Pattern   | Operation                                    |
| --------- | -------------------------------------------- |
| `/#.*/`   | Textually replace with `""`                  |
| `/@(.*)/` | Textually replace with contents of file `$1` |

## Tokens

| Token    | Operation                                         |
| -------- | ------------------------------------------------- |
| `LABEL:` | Define label `LABEL` at current address           |
| `LABEL.` | Define local label `LABEL` at current address     |
| `:LABEL` | Push address of label `LABEL`                     |
| `.LABEL` | Push address of local label `LABEL`               |
| `MACRO%` | Define start of macro `MACRO`                     |
| `%MACRO` | Token-wise replace with contents of macro `MACRO` |
| `dDD`    | Insert `DD` in output binary at current address   |
| `xXX`    | Push `XX` through `psh` and `phn`                 |
| `ldO`    | Instruction `ldo O`                               |
| `stO`    | Instruction `sto O`                               |
| `add`    | Instruction `add 0x01`                            |
| `addS`   | Instruction `add S`                               |
| `adc`    | Instruction `adc 0x01`                            |
| `adcS`   | Instruction `adc S`                               |
| `sub`    | Instruction `sub 0x01`                            |
| `subS`   | Instruction `sub S`                               |
| `sbc`    | Instruction `sbc 0x01`                            |
| `sbcS`   | Instruction `sbc S`                               |
| `shf`    | Instruction `shf 0x01`                            |
| `shfS`   | Instruction `shf S`                               |
| `sfc`    | Instruction `sfc 0x01`                            |
| `sfcS`   | Instruction `sfc S`                               |
| `rot`    | Instruction `rot 0x01`                            |
| `rotS`   | Instruction `rot S`                               |
| `iff`    | Instruction `iff 0x01`                            |
| `iffS`   | Instruction `iff S`                               |
| `orr`    | Instruction `orr 0x01`                            |
| `orrS`   | Instruction `orr S`                               |
| `and`    | Instruction `and 0x01`                            |
| `andS`   | Instruction `and S`                               |
| `xor`    | Instruction `xor 0x01`                            |
| `xorS`   | Instruction `xor S`                               |
| `xnd`    | Instruction `xnd 0x01`                            |
| `xndS`   | Instruction `xnd S`                               |
| `inc`    | Instruction `inc`                                 |
| `dec`    | Instruction `dec`                                 |
| `neg`    | Instruction `neg`                                 |
| `not`    | Instruction `not`                                 |
| `buf`    | Instruction `buf`                                 |
| `nop`    | Instruction `nop`                                 |
| `clc`    | Instruction `clc`                                 |
| `sec`    | Instruction `sec`                                 |
| `flc`    | Instruction `flc`                                 |
| `swp`    | Instruction `swp`                                 |
| `pop`    | Instruction `pop`                                 |
| `lda`    | Instruction `lda`                                 |
| `sta`    | Instruction `sta`                                 |
| `ldi`    | Instruction `ldi`                                 |
| `sti`    | Instruction `sti`                                 |
| `lds`    | Instruction `lds`                                 |
| `sts`    | Instruction `sts`                                 |