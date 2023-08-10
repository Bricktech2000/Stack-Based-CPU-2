fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    println!("Dasm: Usage: dasm <memory image file> <disassembly output file>");
    std::process::exit(1);
  }

  let memory_image_file: &String = &args[1];
  let disassembly_output_file: &String = &args[2];

  let memory: Vec<u8> = match std::fs::read(memory_image_file) {
    Ok(source) => source,
    Err(_) => {
      println!("Dasm: Error: Unable to read file `{}`", memory_image_file);
      std::process::exit(1);
    }
  };

  match memory.try_into() {
    Ok(slice) => {
      let disassembly_output = format!("# Generated by dasm\n\n{}", disassemble(slice, "main"));
      std::fs::write(disassembly_output_file, disassembly_output).unwrap();
      println!("Dasm: Done");
    }
    Err(_) => {
      println!(
        "Dasm: Error: Memory image `{}` has incorrect size",
        memory_image_file
      );
      std::process::exit(1);
    }
  };
}

const MEM_SIZE: usize = 0x100;

fn disassemble(memory: [u8; MEM_SIZE], entry_point: &str) -> String {
  format!(
    "{}!\n{}",
    entry_point,
    memory
      .iter()
      .map(|instruction| {
        match (instruction & 0b10000000) >> 7 {
          0b0 => {
            let imm = instruction; // decode_imm
            let value = imm;
            format!("x{:02X} @dyn", value)
          }

          0b1 => {
            match (instruction & 0b01000000) >> 6 {
              0b0 => {
                // (arithmetic and logic)
                let size = 1 << (instruction & 0b00000011); // decode_size
                let opcode = (instruction & 0b00111100) >> 2;

                fn shorthand(instruction: &str, size: u8) -> String {
                  // generates a shorthand for instruction parameter when available
                  // for example, `shorthand("add", 0x01)` returns `add` instead of `ad1`

                  match size {
                    0x01 => format!("{}", &instruction[0..instruction.len()]),
                    _ => format!("{}{:01X}", &instruction[0..instruction.len() - 1], size),
                  }
                }

                match opcode {
                  0x0 => format!("{} @dyn", shorthand("add", size)),

                  0x1 => format!("{} @dyn", shorthand("sub", size)),

                  0x4 => format!("{} @dyn", shorthand("iff", size)),

                  0x5 => format!("{} @dyn", shorthand("rot", size)),

                  0x8 => format!("{} @dyn", shorthand("orr", size)),

                  0x9 => format!("{} @dyn", shorthand("and", size)),

                  0xA => format!("{} @dyn", shorthand("xor", size)),

                  0xB => format!("{} @dyn", shorthand("xnd", size)),

                  _ => match (opcode, instruction & 0b00000011) {
                    // (size used as part of opcode)
                    (0xC, 0b00) => format!("inc @dyn"),

                    (0xC, 0b01) => format!("dec @dyn"),

                    (0xC, 0b10) => format!("neg @dyn"),

                    (0xD, 0b00) => format!("shl @dyn"),

                    (0xD, 0b01) => format!("shr @dyn"),

                    (0xD, 0b10) => format!("not @dyn"),

                    (0xD, 0b11) => format!("buf @dyn"),

                    (0b1110, 0b11) => format!("dBB     "),

                    _ => {
                      format!("d{:02X}     ", instruction)
                    }
                  },
                }
              }

              0b1 => {
                match (instruction & 0b00100000) >> 5 {
                  0b0 => {
                    // (offset operations)
                    match (instruction & 0b00010000) >> 4 {
                      0b0 => {
                        let ofst = instruction & 0b00001111; // decode_ofst
                        format!("ld{:01X} @dyn", ofst)
                      }

                      0b1 => {
                        let ofst = instruction & 0b00001111; // decode_ofst
                        format!("st{:01X} @dyn", ofst)
                      }

                      _ => unreachable!(),
                    }
                  }

                  0b1 => {
                    match (instruction & 0b00010000) >> 4 {
                      0b0 => {
                        // (carry and flags and stack)
                        match instruction & 0b00001111 {
                          0x0 => format!("lda @dyn"),

                          0x1 => format!("sta @dyn"),

                          0x2 => format!("ldi @dyn"),

                          0x3 => format!("sti @dyn"),

                          0x4 => format!("lds @dyn"),

                          0x5 => format!("sts @dyn"),

                          0x8 => format!("nop @dyn"),

                          0x9 => format!("clc @dyn"),

                          0xA => format!("sec @dyn"),

                          0xB => format!("flc @dyn"),

                          0xC => format!("swp @dyn"),

                          0xD => format!("pop @dyn"),

                          _ => {
                            format!("d{:02X}     ", instruction)
                          }
                        }
                      }

                      0b1 => {
                        let imm = instruction; // decode_imm
                        let value = imm;
                        format!("x{:02X} @dyn", value)
                      }

                      _ => unreachable!(),
                    }
                  }

                  _ => unreachable!(),
                }
              }

              _ => unreachable!(),
            }
          }

          _ => unreachable!(),
        }
      })
      .enumerate()
      .zip(memory.iter())
      .map(|((index, mnemonic), byte)| format!("{} # x{:02X} @org d{:02X}", mnemonic, index, byte))
      .collect::<Vec<String>>()
      .join("\n")
  )
}
