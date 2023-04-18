use std::collections::HashMap;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    println!("Usage: asm <input> <output>");
    std::process::exit(1);
  }

  let mut errors: Vec<(Pos, Error)> = vec![];
  let file: File = File {
    path: args[1].clone(),
  };

  let preprocessed: String = preprocess(file, &mut errors, None);
  let tokens: Vec<(Pos, Token)> = tokenize(preprocessed, &mut errors);
  let instructions: Vec<(Pos, Instruction)> = assemble(tokens, &mut errors, "main");
  let bytes: Vec<(Pos, u8)> = codegen(instructions, &mut errors);

  match errors[..] {
    [] => {
      let output = &args[2];
      std::fs::write(output, bytes.iter().map(|(_, b)| *b).collect::<Vec<u8>>()).unwrap();

      println!("Done.");
    }
    _ => {
      let errors = errors
        .iter()
        .map(|error| format!("{}  {}", error.0, error.1))
        .collect::<Vec<String>>()
        .join("\n");

      println!("{}\n\nAborting.", errors);
      std::process::exit(1);
    }
  }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Label {
  scope_id: Option<usize>,
  identifier: String,
}

impl std::fmt::Display for Label {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self.scope_id {
      Some(_) => write!(f, ".{}", self.identifier),
      None => write!(f, ":{}", self.identifier),
    }
  }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Macro(String);

impl std::fmt::Display for Macro {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "!{}", self.0)
  }
}

#[derive(Clone, Eq, PartialEq)]
struct Error(String);

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Clone, Eq, PartialEq)]
struct Pos {
  scope: String,
  index: usize,
}

impl std::fmt::Display for Pos {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}#{}", self.scope, self.index)
  }
}

#[derive(Clone, Eq, PartialEq)]
struct File {
  path: String,
}

impl std::fmt::Display for File {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use path_clean::PathClean;
    use std::path::Path;
    write!(
      f,
      "@{}",
      Path::new(&self.path).clean().to_str().unwrap().to_string()
    )
  }
}

#[derive(Clone, Eq, PartialEq)]
enum Token {
  LabelDef(Label),
  LabelRef(Label),
  MacroDef(Macro),
  MacroRef(Macro),
  AtConst,
  AtDyn,
  AtOrg,
  DDD(u8),
  XXX(u8),
  LdO(u8),
  StO(u8),
  Add,
  Adc,
  AddS(u8),
  AdcS(u8),
  Sub,
  Sbc,
  SubS(u8),
  SbcS(u8),
  Shf,
  Sfc,
  ShfS(u8),
  SfcS(u8),
  Rot,
  RotS(u8),
  Iff,
  IffS(u8),
  Orr,
  OrrS(u8),
  And,
  AndS(u8),
  Xor,
  XorS(u8),
  Xnd,
  XndS(u8),
  Adn,
  Sbn,
  Inc,
  Dec,
  Neg,
  Not,
  Buf,
  Nop,
  Clc,
  Sec,
  Flc,
  Swp,
  Pop,
  Lda,
  Sta,
  Ldi,
  Sti,
  Lds,
  Sts,
}

impl std::fmt::Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Token::LabelDef(label) => write!(f, "{}", label),
      Token::LabelRef(label) => write!(f, "{}", label),
      Token::MacroDef(macro_) => write!(f, "{}", macro_),
      Token::MacroRef(macro_) => write!(f, "{}", macro_),
      Token::AtConst => write!(f, "@const"),
      Token::AtDyn => write!(f, "@dyn"),
      Token::AtOrg => write!(f, "@org"),
      Token::DDD(n) => write!(f, "d{:02x}", n),
      Token::XXX(n) => write!(f, "x{:02x}", n),
      Token::LdO(n) => write!(f, "ld{:01x}", n),
      Token::StO(n) => write!(f, "st{:01x}", n),
      Token::Add => write!(f, "add"),
      Token::Adc => write!(f, "adc"),
      Token::AddS(n) => write!(f, "add{:01x}", n),
      Token::AdcS(n) => write!(f, "adc{:01x}", n),
      Token::Sub => write!(f, "sub"),
      Token::Sbc => write!(f, "sbc"),
      Token::SubS(n) => write!(f, "sub{:01x}", n),
      Token::SbcS(n) => write!(f, "sbc{:01x}", n),
      Token::Shf => write!(f, "shf"),
      Token::Sfc => write!(f, "sfc"),
      Token::ShfS(n) => write!(f, "shf{:01x}", n),
      Token::SfcS(n) => write!(f, "sfc{:01x}", n),
      Token::Rot => write!(f, "rot"),
      Token::RotS(n) => write!(f, "rot{:01x}", n),
      Token::Iff => write!(f, "iff"),
      Token::IffS(n) => write!(f, "iff{:01x}", n),
      Token::Orr => write!(f, "orr"),
      Token::OrrS(n) => write!(f, "orr{:01x}", n),
      Token::And => write!(f, "and"),
      Token::AndS(n) => write!(f, "and{:01x}", n),
      Token::Xor => write!(f, "xor"),
      Token::XorS(n) => write!(f, "xor{:01x}", n),
      Token::Xnd => write!(f, "xnd"),
      Token::XndS(n) => write!(f, "xnd{:01x}", n),
      Token::Adn => write!(f, "adn"),
      Token::Sbn => write!(f, "sbn"),
      Token::Inc => write!(f, "inc"),
      Token::Dec => write!(f, "dec"),
      Token::Neg => write!(f, "neg"),
      Token::Not => write!(f, "not"),
      Token::Buf => write!(f, "buf"),
      Token::Nop => write!(f, "nop"),
      Token::Clc => write!(f, "clc"),
      Token::Sec => write!(f, "sec"),
      Token::Flc => write!(f, "flc"),
      Token::Swp => write!(f, "swp"),
      Token::Pop => write!(f, "pop"),
      Token::Lda => write!(f, "lda"),
      Token::Sta => write!(f, "sta"),
      Token::Ldi => write!(f, "ldi"),
      Token::Sti => write!(f, "sti"),
      Token::Lds => write!(f, "lds"),
      Token::Sts => write!(f, "sts"),
    }
  }
}

#[derive(Clone, Eq, PartialEq)]
enum Instruction {
  Psh(u8),
  Phn(u8),
  Ldo(u8),
  Sto(u8),
  Add(u8),
  Adc(u8),
  Sub(u8),
  Sbc(u8),
  Shf(u8),
  Sfc(u8),
  Rot(u8),
  Iff(u8),
  Orr(u8),
  And(u8),
  Xor(u8),
  Xnd(u8),
  Adn,
  Sbn,
  Inc,
  Dec,
  Neg,
  Not,
  Buf,
  Nop,
  Clc,
  Sec,
  Flc,
  Swp,
  Pop,
  Lda,
  Sta,
  Ldi,
  Sti,
  Lds,
  Sts,
  Raw(u8),
}

fn preprocess(file: File, errors: &mut Vec<(Pos, Error)>, scope: Option<&str>) -> String {
  // remove comments and resolve includes

  use std::path::Path;
  let source = match std::fs::read_to_string(&file.path) {
    Ok(source) => source,
    Err(_) => {
      errors.push((
        Pos {
          scope: scope.unwrap_or("[bootstrap]").to_string(),
          index: 0,
        },
        Error(format!("Unable to read file: {}", file)),
      ));
      "".to_string()
    }
  };

  let source: String = source
    .lines()
    .map(|line| line.split("#").next().unwrap())
    .map(|line| match line.find("@ ") {
      Some(i) => {
        line[..i].to_owned()
          + preprocess(
            File {
              path: Path::new(&file.path)
                .parent()
                .unwrap()
                .join(&line[i..][2..])
                .to_str()
                .unwrap()
                .to_string(),
            },
            errors,
            Some(&format!("{}", file)),
          )
          .as_str()
      }
      None => line.to_string(),
    })
    .collect::<Vec<_>>()
    .join("\n");

  source
}

fn tokenize(source: String, errors: &mut Vec<(Pos, Error)>) -> Vec<(Pos, Token)> {
  // tokenize to valid tokens. tokens might be invalid instructions

  fn parse_hex(input: &str, errors: &mut Vec<(Pos, Error)>, position: &Pos) -> u8 {
    use std::num::IntErrorKind::*;
    match u8::from_str_radix(input, 16) {
      Ok(value) => value,
      Err(e) => {
        match e.kind() {
          InvalidDigit => errors.push((
            position.clone(),
            Error(format!("Invalid digits in hexadecimal literal: {}", input)),
          )),
          Empty => errors.push((
            position.clone(),
            Error(format!("Invalid empty hexadecimal literal")),
          )),
          NegOverflow | PosOverflow => errors.push((
            position.clone(),
            Error(format!("Hexadecimal literal out of range: {}", input)),
          )),
          _ => panic!("Unexpected error parsing hexadecimal literal"),
        };
        0x00
      }
    }
  }

  let tokens: Vec<&str> = source.split_whitespace().collect();

  let tokens: Vec<(Pos, Token)> = tokens
    .into_iter()
    .enumerate()
    .map(|(index, token)| {
      let position = Pos {
        scope: "[token stream]".to_string(),
        index,
      };

      let token = match token {
        _ if token.ends_with(":") => Token::LabelDef(Label {
          scope_id: None,
          identifier: token[..token.len() - 1].to_string(),
        }),
        _ if token.starts_with(":") => Token::LabelRef(Label {
          scope_id: None,
          identifier: token[1..].to_string(),
        }),
        _ if token.ends_with(".") => Token::LabelDef(Label {
          scope_id: Some(0),
          identifier: token[..token.len() - 1].to_string(),
        }),
        _ if token.starts_with(".") => Token::LabelRef(Label {
          scope_id: Some(0),
          identifier: token[1..].to_string(),
        }),
        "@const" => Token::AtConst,
        "@dyn" => Token::AtDyn,
        "@org" => Token::AtOrg,
        _ if token.ends_with("!") => Token::MacroDef(Macro(token[..token.len() - 1].to_string())),
        _ if token.starts_with("!") => Token::MacroRef(Macro(token[1..].to_string())),
        "add" => Token::Add,
        "adc" => Token::Adc,
        _ if token.starts_with("add") => Token::AddS(parse_hex(&token[3..], errors, &position)),
        _ if token.starts_with("adc") => Token::AdcS(parse_hex(&token[3..], errors, &position)),
        "sub" => Token::Sub,
        "sbc" => Token::Sbc,
        _ if token.starts_with("sub") => Token::SubS(parse_hex(&token[3..], errors, &position)),
        _ if token.starts_with("sbc") => Token::SbcS(parse_hex(&token[3..], errors, &position)),
        "shf" => Token::Shf,
        _ if token.starts_with("shf") => Token::ShfS(parse_hex(&token[3..], errors, &position)),
        "shc" => Token::Sfc,
        _ if token.starts_with("sfc") => Token::SfcS(parse_hex(&token[3..], errors, &position)),
        "rot" => Token::Rot,
        _ if token.starts_with("rot") => Token::RotS(parse_hex(&token[3..], errors, &position)),
        "iff" => Token::Iff,
        _ if token.starts_with("iff") => Token::IffS(parse_hex(&token[3..], errors, &position)),
        "orr" => Token::Orr,
        _ if token.starts_with("orr") => Token::OrrS(parse_hex(&token[3..], errors, &position)),
        "and" => Token::And,
        _ if token.starts_with("and") => Token::AndS(parse_hex(&token[3..], errors, &position)),
        "xor" => Token::Xor,
        _ if token.starts_with("xor") => Token::XorS(parse_hex(&token[3..], errors, &position)),
        "xnd" => Token::Xnd,
        _ if token.starts_with("xnd") => Token::XndS(parse_hex(&token[3..], errors, &position)),
        "adn" => Token::Adn,
        "sbn" => Token::Sbn,
        "inc" => Token::Inc,
        "dec" => Token::Dec,
        "neg" => Token::Neg,
        "not" => Token::Not,
        "buf" => Token::Buf,
        "nop" => Token::Nop,
        "clc" => Token::Clc,
        "sec" => Token::Sec,
        "flc" => Token::Flc,
        "swp" => Token::Swp,
        "pop" => Token::Pop,
        "lda" => Token::Lda,
        "sta" => Token::Sta,
        "ldi" => Token::Ldi,
        "sti" => Token::Sti,
        "lds" => Token::Lds,
        "sts" => Token::Sts,
        _ if token.starts_with("d") => Token::DDD(parse_hex(&token[1..], errors, &position)),
        _ if token.starts_with("x") => Token::XXX(parse_hex(&token[1..], errors, &position)),
        _ if token.starts_with("ld") => Token::LdO(parse_hex(&token[2..], errors, &position)),
        _ if token.starts_with("st") => Token::StO(parse_hex(&token[2..], errors, &position)),
        _ => {
          errors.push((
            position.clone(),
            Error(format!("Unexpected token: {}", token)),
          ));
          Token::Nop
        }
      };

      (position, token)
    })
    .collect();

  tokens
}

fn assemble(
  tokens: Vec<(Pos, Token)>,
  errors: &mut Vec<(Pos, Error)>,
  entry_point: &str,
) -> Vec<(Pos, Instruction)> {
  // resolve macros recursively from `entry_point`

  let mut macro_definitions: HashMap<Macro, Vec<(Pos, Token)>> = HashMap::new();
  let mut current_macro: Option<Macro> = None;

  for token in tokens.into_iter() {
    match token.1 {
      Token::MacroDef(macro_) => {
        current_macro = Some(macro_.clone());
        macro_definitions.entry(macro_).or_insert(vec![]);
      }
      _ => match current_macro
        .as_ref()
        .and_then(|macro_| macro_definitions.get_mut(&macro_))
      {
        Some(macro_tokens) => macro_tokens.push((
          Pos {
            scope: format!("{}", current_macro.as_ref().unwrap()),
            index: macro_tokens.len(),
          },
          token.1,
        )),
        None => errors.push((
          token.0,
          Error(format!("Orphan instruction found: {}", token.1)),
        )),
      },
    }
  }

  let entry_point = vec![(
    Pos {
      scope: "[bootstrap]".to_string(),
      index: 0,
    },
    Token::MacroRef(Macro(entry_point.to_string())),
  )];
  let mut scope_id: usize = 1;
  let mut parents: Vec<Macro> = vec![];
  let tokens: Vec<(Pos, Token)> = expand_macros(
    &macro_definitions,
    &entry_point,
    &mut parents,
    &mut scope_id,
    errors,
  );

  fn expand_macros<'a>(
    macro_definitions: &HashMap<Macro, Vec<(Pos, Token)>>,
    tokens: &Vec<(Pos, Token)>,
    parents: &mut Vec<Macro>,
    scope_id: &mut usize,
    errors: &mut Vec<(Pos, Error)>,
  ) -> Vec<(Pos, Token)> {
    tokens
      .into_iter()
      .flat_map(|token| match &token.1 {
        Token::MacroRef(macro_) => {
          if parents.contains(macro_) {
            errors.push((
              token.0.clone(),
              Error(format!(
                "Macro self-reference: {} -> {}",
                parents
                  .iter()
                  .map(|macro_| format!("{}", macro_))
                  .collect::<Vec<String>>()
                  .join(" -> "),
                macro_
              )),
            ));
            vec![]
          } else {
            let tokens = match macro_definitions.get(macro_) {
              Some(tokens) => tokens.clone(),
              None => {
                errors.push((
                  token.0.clone(),
                  Error(format!("Definition not found for macro: {}", macro_)),
                ));
                vec![]
              }
            };

            let tokens = tokens
              .into_iter()
              .map(|token| match token.1 {
                Token::LabelDef(Label {
                  scope_id: Some(_),
                  identifier,
                }) => (
                  token.0,
                  Token::LabelDef(Label {
                    scope_id: Some(*scope_id),
                    identifier,
                  }),
                ),
                Token::LabelRef(Label {
                  scope_id: Some(_),
                  identifier,
                }) => (
                  token.0,
                  Token::LabelRef(Label {
                    scope_id: Some(*scope_id),
                    identifier,
                  }),
                ),
                _ => token,
              })
              .collect();

            *scope_id += 1;
            parents.push(macro_.clone());
            let expanded = expand_macros(&macro_definitions, &tokens, parents, scope_id, errors);
            parents.pop();
            expanded
          }
        }
        _ => vec![token.clone()],
      })
      .collect()
  }

  fn assert_immediate(immediate: u8, errors: &mut Vec<(Pos, Error)>, position: &Pos) -> u8 {
    match immediate {
      0b00000000..=0b11111111 => immediate,
      #[allow(unreachable_patterns)]
      _ => {
        errors.push((
          position.clone(),
          Error(format!("Invalid immediate operand: {:02x}", immediate)),
        ));
        0b00000000
      }
    }
  }

  fn assert_size(size: u8, errors: &mut Vec<(Pos, Error)>, position: &Pos) -> u8 {
    match size {
      0x01 | 0x02 | 0x04 | 0x08 => size,
      _ => {
        errors.push((
          position.clone(),
          Error(format!("Invalid size operand: {:02x}", size)),
        ));
        0x01
      }
    }
  }

  fn assert_offset(offset: u8, errors: &mut Vec<(Pos, Error)>, position: &Pos) -> u8 {
    match offset {
      0b00000000..=0b00001111 => offset,
      _ => {
        errors.push((
          position.clone(),
          Error(format!("Invalid offset operand: {:02x}", offset)),
        ));
        0b00000000
      }
    }
  }

  // turn assembly tokens into roots, an intermediate representation. roots correspond to valid instructions

  #[derive(Clone, Eq, PartialEq)]
  enum Root {
    Instruction(Instruction),
    Node(Node),
    LabelDef(Label),
    Const,
    Dyn(Option<Instruction>),
    Org(Option<Node>),
  }

  #[derive(Clone, Eq, PartialEq)]
  enum Node {
    LabelRef(Label),
    Immediate(u8),
    Not(Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Shf(Box<Node>, Box<Node>),
    Rot(Box<Node>, Box<Node>),
    Orr(Box<Node>, Box<Node>),
    And(Box<Node>, Box<Node>),
    Xor(Box<Node>, Box<Node>),
    Xnd(Box<Node>, Box<Node>),
  }

  let roots: Vec<(Pos, Root)> = tokens
    .into_iter()
    .map(|token| {
      let position = token.0;
      let token = match token.1 {
        Token::LabelDef(label) => Root::LabelDef(label),
        Token::LabelRef(label) => Root::Node(Node::LabelRef(label)),
        Token::MacroDef(_) => panic!("Macro definition found in intermediate representation"),
        Token::MacroRef(_) => panic!("Macro reference found in intermediate representation"),
        Token::AtConst => Root::Const,
        Token::AtDyn => Root::Dyn(None),
        Token::AtOrg => Root::Org(None),
        Token::XXX(immediate) => Root::Node(Node::Immediate(assert_immediate(
          immediate, errors, &position,
        ))),
        Token::LdO(offset) => {
          Root::Instruction(Instruction::Ldo(assert_offset(offset, errors, &position)))
        }
        Token::StO(offset) => {
          Root::Instruction(Instruction::Sto(assert_offset(offset, errors, &position)))
        }
        Token::Add => Root::Instruction(Instruction::Add(assert_size(0x01, errors, &position))),
        Token::Adc => Root::Instruction(Instruction::Adc(assert_size(0x01, errors, &position))),
        Token::AddS(size) => {
          Root::Instruction(Instruction::Add(assert_size(size, errors, &position)))
        }
        Token::AdcS(size) => {
          Root::Instruction(Instruction::Adc(assert_size(size, errors, &position)))
        }
        Token::Sub => Root::Instruction(Instruction::Sub(assert_size(0x01, errors, &position))),
        Token::Sbc => Root::Instruction(Instruction::Sbc(assert_size(0x01, errors, &position))),
        Token::SubS(size) => {
          Root::Instruction(Instruction::Sub(assert_size(size, errors, &position)))
        }
        Token::SbcS(size) => {
          Root::Instruction(Instruction::Sbc(assert_size(size, errors, &position)))
        }
        Token::Shf => Root::Instruction(Instruction::Shf(assert_size(0x01, errors, &position))),
        Token::Sfc => Root::Instruction(Instruction::Sfc(assert_size(0x01, errors, &position))),
        Token::ShfS(size) => {
          Root::Instruction(Instruction::Shf(assert_size(size, errors, &position)))
        }
        Token::SfcS(size) => {
          Root::Instruction(Instruction::Sfc(assert_size(size, errors, &position)))
        }
        Token::Rot => Root::Instruction(Instruction::Rot(assert_size(0x01, errors, &position))),
        Token::RotS(size) => {
          Root::Instruction(Instruction::Rot(assert_size(size, errors, &position)))
        }
        Token::Iff => Root::Instruction(Instruction::Iff(assert_size(0x01, errors, &position))),
        Token::IffS(size) => {
          Root::Instruction(Instruction::Iff(assert_size(size, errors, &position)))
        }
        Token::Orr => Root::Instruction(Instruction::Orr(assert_size(0x01, errors, &position))),
        Token::OrrS(size) => {
          Root::Instruction(Instruction::Orr(assert_size(size, errors, &position)))
        }
        Token::And => Root::Instruction(Instruction::And(assert_size(0x01, errors, &position))),
        Token::AndS(size) => {
          Root::Instruction(Instruction::And(assert_size(size, errors, &position)))
        }
        Token::Xor => Root::Instruction(Instruction::Xor(assert_size(0x01, errors, &position))),
        Token::XorS(size) => {
          Root::Instruction(Instruction::Xor(assert_size(size, errors, &position)))
        }
        Token::Xnd => Root::Instruction(Instruction::Xnd(assert_size(0x01, errors, &position))),
        Token::XndS(size) => {
          Root::Instruction(Instruction::Xnd(assert_size(size, errors, &position)))
        }
        Token::Adn => Root::Instruction(Instruction::Adn),
        Token::Sbn => Root::Instruction(Instruction::Sbn),
        Token::Inc => Root::Instruction(Instruction::Inc),
        Token::Dec => Root::Instruction(Instruction::Dec),
        Token::Neg => Root::Instruction(Instruction::Neg),
        Token::Not => Root::Instruction(Instruction::Not),
        Token::Buf => Root::Instruction(Instruction::Buf),
        Token::Nop => Root::Instruction(Instruction::Nop),
        Token::Clc => Root::Instruction(Instruction::Clc),
        Token::Sec => Root::Instruction(Instruction::Sec),
        Token::Flc => Root::Instruction(Instruction::Flc),
        Token::Swp => Root::Instruction(Instruction::Swp),
        Token::Pop => Root::Instruction(Instruction::Pop),
        Token::Lda => Root::Instruction(Instruction::Lda),
        Token::Sta => Root::Instruction(Instruction::Sta),
        Token::Ldi => Root::Instruction(Instruction::Ldi),
        Token::Sti => Root::Instruction(Instruction::Sti),
        Token::Lds => Root::Instruction(Instruction::Lds),
        Token::Sts => Root::Instruction(Instruction::Sts),
        Token::DDD(immediate) => Root::Instruction(Instruction::Raw(immediate)),
      };

      (position, token)
    })
    .collect();

  // build a tree of nodes representing everything we can compute at compile time
  // this removes redundant instructions and makes macros usable

  // a convenience function to replace slice patterns within a vector
  fn match_replace<const N: usize>(
    roots: &Vec<(Pos, Root)>,
    func: fn(&[Root; N]) -> Option<Vec<Root>>,
  ) -> Vec<(Pos, Root)> {
    if roots.len() < N {
      return roots.clone();
    }

    let mut output: Vec<(Pos, Root)> = vec![];

    let mut skip_next = 0;
    for window in roots.windows(N) {
      if skip_next > 0 {
        skip_next -= 1;
      } else {
        match func(
          window
            .iter()
            .map(|(_, root)| root.clone())
            .collect::<Vec<Root>>()
            .as_slice()
            .try_into()
            .unwrap(),
        ) {
          Some(roots) => {
            output.extend(
              roots
                .into_iter()
                .map(|root| (window[0].0.clone(), root))
                .collect::<Vec<(Pos, Root)>>(),
            );
            skip_next = N - 1;
          }
          None => output.push(window[0].clone()),
        }
      }
    }
    output.extend(roots.iter().skip(1 + roots.len() - N + skip_next).cloned());

    output
  }

  let mut roots = roots;
  let mut last_roots = vec![];

  roots = match_replace(&roots, |window| match window {
    [Root::Instruction(instruction), Root::Dyn(None)] => {
      Some(vec![Root::Dyn(Some(instruction.clone()))])
    }
    _ => None,
  });

  while roots != last_roots {
    last_roots = roots.clone();
    // println!("roots: {:?}\nlen: {}", roots, roots.len());

    roots = match_replace(&roots, |window| match window {
      [Root::Instruction(Instruction::Nop)] => Some(vec![]),
      _ => None,
    });

    roots =
      match_replace(&roots, |window| match window {
        [Root::Node(node), Root::Const] => Some(vec![Root::Node(node.clone())]),
        [Root::Node(node), Root::Org(None)] => Some(vec![Root::Org(Some(node.clone()))]),
        [Root::Node(node), Root::Instruction(Instruction::Ldo(0x00))] => {
          Some(vec![Root::Node(node.clone()), Root::Node(node.clone())])
        }
        [Root::Node(node), Root::Instruction(Instruction::Inc)] => Some(vec![Root::Node(
          Node::Add(Box::new(node.clone()), Box::new(Node::Immediate(1))),
        )]),
        [Root::Node(node), Root::Instruction(Instruction::Dec)] => Some(vec![Root::Node(
          Node::Sub(Box::new(node.clone()), Box::new(Node::Immediate(1))),
        )]),
        [Root::Node(node), Root::Instruction(Instruction::Neg)] => Some(vec![Root::Node(
          Node::Sub(Box::new(node.clone()), Box::new(Node::Immediate(0))),
        )]),
        [Root::Node(node), Root::Instruction(Instruction::Not)] => {
          Some(vec![Root::Node(Node::Not(Box::new(node.clone())))])
        }
        [Root::Node(node), Root::Instruction(Instruction::Buf)] => {
          Some(vec![Root::Node(node.clone())])
        }
        [Root::Node(_), Root::Instruction(Instruction::Pop)] => Some(vec![]),
        _ => None,
      });

    roots = match_replace(&roots, |window| match window {
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Ldo(0x01))] => {
        Some(vec![
          Root::Node(node1.clone()),
          Root::Node(node2.clone()),
          Root::Node(node1.clone()),
        ])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Add(0x01))] => {
        Some(vec![Root::Node(Node::Add(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Adc(0x01))] => {
        Some(vec![Root::Node(Node::Add(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Sub(0x01))] => {
        Some(vec![Root::Node(Node::Sub(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Sbc(0x01))] => {
        Some(vec![Root::Node(Node::Sub(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Shf(0x01))] => {
        Some(vec![Root::Node(Node::Shf(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Sfc(0x01))] => {
        Some(vec![Root::Node(Node::Shf(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Rot(0x01))] => {
        Some(vec![Root::Node(Node::Rot(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Orr(0x01))] => {
        Some(vec![Root::Node(Node::Orr(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::And(0x01))] => {
        Some(vec![Root::Node(Node::And(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Xor(0x01))] => {
        Some(vec![Root::Node(Node::Xor(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Xnd(0x01))] => {
        Some(vec![Root::Node(Node::Xnd(
          Box::new(node2.clone()),
          Box::new(node1.clone()),
        ))])
      }
      [Root::Node(node1), Root::Node(node2), Root::Instruction(Instruction::Swp)] => {
        Some(vec![Root::Node(node2.clone()), Root::Node(node1.clone())])
      }
      [Root::Instruction(Instruction::Ldo(offset)), Root::Node(node2), Root::Instruction(Instruction::Swp)] => {
        Some(vec![
          Root::Node(node2.clone()),
          Root::Instruction(Instruction::Ldo(*offset + 1)),
        ])
      }
      [Root::Node(node1), Root::Instruction(Instruction::Ldo(offset)), Root::Instruction(Instruction::Swp)] => {
        Some(vec![
          Root::Instruction(Instruction::Ldo(*offset - 1)),
          Root::Node(node1.clone()),
        ])
      }
      [Root::Instruction(Instruction::Ldo(offset1)), Root::Instruction(Instruction::Ldo(offset2)), Root::Instruction(Instruction::Swp)] => {
        Some(vec![
          Root::Instruction(Instruction::Ldo(*offset2 - 1)),
          Root::Instruction(Instruction::Ldo(*offset1 + 1)),
        ])
      }
      _ => None,
    });
  }

  roots = match_replace(&roots, |window| match window {
    [Root::Node(node1), Root::Node(node2)] if node1 == node2 => Some(vec![
      Root::Node(node1.clone()),
      Root::Instruction(Instruction::Ldo(0x00)),
    ]),
    [Root::Instruction(Instruction::Swp), Root::Instruction(Instruction::Pop)] => {
      Some(vec![Root::Instruction(Instruction::Sto(0x00))])
    }
    _ => None,
  });

  roots = match_replace(&roots, |window| match window {
    [Root::Node(node1), Root::Node(node2), Root::Node(node3)] if node1 == node3 => Some(vec![
      Root::Node(node1.clone()),
      Root::Node(node2.clone()),
      Root::Instruction(Instruction::Ldo(0x01)),
    ]),
    _ => None,
  });

  // assemble roots into instructions by computing the value of every node and resolving labels

  fn eval<'a>(node: &'a Node, labels: &HashMap<Label, u8>) -> Result<u8, Label> {
    Ok(match node {
      Node::LabelRef(label) => *labels.get(label).ok_or(label.clone())?,
      Node::Immediate(immediate) => *immediate,
      Node::Not(node) => !eval(node, labels)?,
      Node::Add(node1, node2) => eval(node2, labels)?.wrapping_add(eval(node1, labels)?),
      Node::Sub(node1, node2) => eval(node2, labels)?.wrapping_sub(eval(node1, labels)?),
      Node::Shf(node1, node2) => {
        let a = eval(node1, labels)? as u16;
        let b = eval(node2, labels)? as u16;

        let shifted = if a as i8 >= 0 {
          (b as u16).wrapping_shl(a as u32)
        } else {
          (b as u16).wrapping_shr(a.wrapping_neg() as u32)
        } as u16;

        shifted as u8
      }
      Node::Rot(node1, node2) => {
        let a = eval(node1, labels)? as u16;
        let b = eval(node2, labels)? as u16;

        let shifted = if a as i8 >= 0 {
          (b as u16).wrapping_shl(a as u32)
        } else {
          (b as u16).wrapping_shr(a.wrapping_neg() as u32)
        } as u16;

        (shifted & 0xFF) as u8 | (shifted >> 8) as u8
      }
      Node::Orr(node1, node2) => eval(node2, labels)? | eval(node1, labels)?,
      Node::And(node1, node2) => eval(node2, labels)? & eval(node1, labels)?,
      Node::Xor(node1, node2) => eval(node2, labels)? ^ eval(node1, labels)?,
      Node::Xnd(_, _) => 0,
    })
  }

  fn make_push_instruction(immediate: u8, position: &Pos) -> Vec<(Pos, Instruction)> {
    // the `Psh` instruction allows us to push arbitrary 7-bit immediates onto the stack.
    // we then optionally use `Neg` and `Inc` to get the ability to push arbitrary 8-bit
    // immediates. we also use `Phn` as a shorthand when possible.

    if immediate & 0b11110000 == 0b11110000 {
      vec![(position.clone(), Instruction::Phn(immediate & 0b00001111))]
    } else if immediate == 0b10000000 {
      vec![
        (position.clone(), Instruction::Psh(0b01111111)),
        (position.clone(), Instruction::Inc),
      ]
    } else {
      match immediate & 0b10000000 {
        0b00000000 => vec![(position.clone(), Instruction::Psh(immediate & 0b01111111))],
        0b10000000 => vec![
          (position.clone(), Instruction::Psh(immediate.wrapping_neg())),
          (position.clone(), Instruction::Neg),
        ],
        _ => unreachable!(),
      }
    }
  }

  // if every label a node depends on could be resolved, we can replace it with an immediate.
  // if not, assume the worst case and reserve two bytes for pushing an immediate later

  let mut label_definitions: HashMap<Label, u8> = HashMap::new();
  let mut unevaluated_nodes: HashMap<u8, (Pos, Node)> = HashMap::new();

  let mut location_counter: u8 = 0;
  let instructions: Vec<(Pos, Instruction)> = roots
    .into_iter()
    .flat_map(|root| match root.1 {
      Root::Instruction(instruction) | Root::Dyn(Some(instruction)) => {
        let instructions = vec![(root.0, instruction)];
        location_counter = location_counter.wrapping_add(instructions.len() as u8);
        instructions
      }

      Root::Node(node) => match eval(&node, &label_definitions) {
        Ok(value) => {
          let instructions = make_push_instruction(value, &root.0);
          location_counter = location_counter.wrapping_add(instructions.len() as u8);
          instructions
        }
        Err(_) => {
          let instructions = vec![
            (root.0.clone(), Instruction::Nop),
            (root.0.clone(), Instruction::Nop),
          ];
          unevaluated_nodes.insert(location_counter, (root.0, node));
          location_counter = location_counter.wrapping_add(instructions.len() as u8);
          instructions
        }
      },

      Root::LabelDef(Label {
        scope_id: Some(0),
        identifier: _,
      }) => panic!("Local label has no scope specified"),

      Root::LabelDef(label) => {
        if label_definitions.contains_key(&label) {
          errors.push((root.0, Error(format!("Label already defined: {}", label))));
        }
        label_definitions.insert(label, location_counter);
        vec![]
      }

      Root::Org(Some(node)) => match eval(&node, &label_definitions) {
        Ok(value) => {
          if value > location_counter {
            let difference = value - location_counter;
            location_counter = location_counter.wrapping_sub(difference);
            vec![(root.0, Instruction::Raw(0x00)); difference as usize]
          } else {
            errors.push((
              root.0,
              Error(format!(
                "Origin cannot move location counter backward from: {} to: {}",
                location_counter, value
              )),
            ));
            vec![]
          }
        }
        Err(label) => {
          errors.push((
            root.0,
            Error(format!(
              "Origin argument contains currently unresolved label: {}",
              label
            )),
          ));
          vec![]
        }
      },

      Root::Org(None) | Root::Const => {
        errors.push((
          root.0,
          Error(format!(
            "Origin or constant argument is not a constant expression"
          )),
        ));
        vec![]
      }

      Root::Dyn(None) => {
        errors.push((
          root.0,
          Error(format!("Dynamic argument is not an instruction")),
        ));
        vec![]
      }
    })
    .collect();

  // poke into the instructions and evaluate all nodes that couldn't be evaluated before

  let mut instructions = instructions;

  for (location_counter, node) in unevaluated_nodes.iter() {
    let immediate = match eval(&node.1, &label_definitions) {
      Ok(value) => value,
      Err(label) => {
        errors.push((
          node.0.clone(),
          Error(format!("Definition not found for label: {}", label)),
        ));
        0x00
      }
    };

    for (index, instruction) in make_push_instruction(immediate, &node.0).iter().enumerate() {
      instructions[*location_counter as usize + index] = instruction.clone();
    }
  }

  instructions
}

#[allow(unused)]
fn codegen(
  instructions: Vec<(Pos, Instruction)>,
  errors: &mut Vec<(Pos, Error)>,
) -> Vec<(Pos, u8)> {
  fn encode_immediate(immediate: u8) -> u8 {
    match immediate {
      0b00000000..=0b01111111 => immediate,
      _ => panic!("Invalid immediate in codegen stage"),
    }
  }

  fn encode_size(size: u8) -> u8 {
    match size {
      0x01 => 0x00,
      0x02 => 0x01,
      0x04 => 0x02,
      0x08 => 0x03,
      _ => panic!("Invalid size in codegen stage"),
    }
  }

  fn encode_offset(offset: u8) -> u8 {
    match offset {
      0b00000000..=0b00001111 => offset,
      _ => panic!("Invalid offset in codegen stage"),
    }
  }

  // codegen instructions into bytes and sanity-check operands

  let bytes: Vec<(Pos, u8)> = instructions
    .into_iter()
    .map(|instruction| {
      let position = instruction.0;
      let instruction = match instruction.1 {
        Instruction::Psh(immediate) => 0b00000000 | encode_immediate(immediate),
        Instruction::Phn(immediate) => 0b11110000 | encode_offset(immediate),
        Instruction::Ldo(offset) => 0b11000000 | encode_offset(offset),
        Instruction::Sto(offset) => 0b11010000 | encode_offset(offset),
        Instruction::Add(size) => 0b10000000 | encode_size(size),
        Instruction::Adc(size) => 0b10000100 | encode_size(size),
        Instruction::Sub(size) => 0b10001000 | encode_size(size),
        Instruction::Sbc(size) => 0b10001100 | encode_size(size),
        Instruction::Shf(size) => 0b10010000 | encode_size(size),
        Instruction::Sfc(size) => 0b10010100 | encode_size(size),
        Instruction::Rot(size) => 0b10011000 | encode_size(size),
        Instruction::Iff(size) => 0b10011100 | encode_size(size),
        Instruction::Orr(size) => 0b10100000 | encode_size(size),
        Instruction::And(size) => 0b10100100 | encode_size(size),
        Instruction::Xor(size) => 0b10101000 | encode_size(size),
        Instruction::Xnd(size) => 0b10101100 | encode_size(size),
        Instruction::Adn => 0b10110000,
        Instruction::Sbn => 0b10110001,
        Instruction::Inc => 0b10110010,
        Instruction::Dec => 0b10110011,
        Instruction::Neg => 0b10110100,
        Instruction::Not => 0b10110110,
        Instruction::Buf => 0b10110111,
        Instruction::Nop => 0xE0,
        Instruction::Clc => 0xE1,
        Instruction::Sec => 0xE2,
        Instruction::Flc => 0xE3,
        Instruction::Swp => 0xE4,
        Instruction::Pop => 0xE5,
        Instruction::Lda => 0xE8,
        Instruction::Sta => 0xE9,
        Instruction::Ldi => 0xEA,
        Instruction::Sti => 0xEB,
        Instruction::Lds => 0xEC,
        Instruction::Sts => 0xED,
        Instruction::Raw(data) => data,
      };

      (position, instruction)
    })
    .collect();

  let available_memory = 0x100;
  let mut bytes = bytes;
  let position = Pos {
    scope: "[codegen]".to_string(),
    index: 0,
  };

  if bytes.len() > available_memory {
    errors.push((
      position,
      Error(format!(
        "Program size: {:02x} exceeds available memory: {:02x}",
        bytes.len(),
        available_memory
      )),
    ));
  } else {
    bytes.extend(vec![(position, 0x00); available_memory - bytes.len()]);
  }

  bytes
}
