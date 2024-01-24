use crate::*;

#[rustfmt::skip] macro_rules! ret_label { () => { Label::Local(format!("ret"), None) }; }
#[rustfmt::skip] macro_rules! end_label { ($name:expr) => { Label::Local(format!("{}.end", $name), None) }; }
#[rustfmt::skip] macro_rules! else_label { ($name:expr) => { Label::Local(format!("{}.else", $name), None) }; }
#[rustfmt::skip] macro_rules! cond_label { ($name:expr) => { Label::Local(format!("{}.cond", $name), None) }; }
#[rustfmt::skip] macro_rules! begin_label { ($name:expr) => { Label::Local(format!("{}.begin", $name), None) }; }

#[rustfmt::skip] pub(crate) use ret_label;
#[rustfmt::skip] pub(crate) use end_label;
#[rustfmt::skip] pub(crate) use else_label;
#[rustfmt::skip] pub(crate) use cond_label;
#[rustfmt::skip] pub(crate) use begin_label;

pub fn codegen(
  program: TypedProgram,
  _errors: &mut Vec<(Pos, Error)>,
) -> Vec<Result<Token, String>> {
  codegen::program(program)
}

fn program(program: TypedProgram) -> Vec<Result<Token, String>> {
  match program {
    TypedProgram(globals) => globals
      .into_iter()
      .flat_map(|global| codegen::global(global))
      .collect(),
  }
}

fn global(global: TypedGlobal) -> Vec<Result<Token, String>> {
  match global {
    TypedGlobal::String(label, value) => {
      std::iter::empty()
        .chain(vec![
          Ok(Token::MacroDef(link::def_macro!(&label))),
          Ok(Token::LabelDef(link::global_label!(&label))),
        ])
        .chain(
          value
            .chars()
            .map(|c| Ok(Token::AtDD(c as u8)))
            .collect::<Vec<Result<Token, String>>>(),
        )
        .chain(vec![Ok(Token::AtDD(0x00))])
        // TODO uses debug formatting
        .chain(vec![Err(format!("# {:?}", value))])
        .collect()
    }

    TypedGlobal::Macro(label, statement) => std::iter::empty()
      .chain(vec![Ok(Token::MacroDef(link::global_macro!(&label)))])
      .chain(codegen::statement(statement))
      .chain(vec![Ok(Token::LabelDef(codegen::ret_label!()))])
      .chain(vec![Err(format!(""))])
      .collect(),

    TypedGlobal::Function(label, statement) => std::iter::empty()
      .chain(vec![
        Ok(Token::MacroDef(link::def_macro!(&label))),
        Ok(Token::LabelDef(link::global_label!(&label))),
      ])
      .chain(codegen::statement(statement))
      .chain(vec![Err(format!(""))])
      .collect(),

    // raw assembly that might not be valid is encoded through the `Err` variant
    TypedGlobal::Assembly(assembly) => std::iter::empty().chain(vec![Err(assembly)]).collect(),
  }
}

fn statement(statement: TypedStatement) -> Vec<Result<Token, String>> {
  match statement {
    TypedStatement::ExpressionN0(expression) => std::iter::empty()
      .chain(codegen::n0_expression(expression, 0))
      .collect(),

    TypedStatement::Compound(statements) => statements
      .into_iter()
      .flat_map(|statement| codegen::statement(statement))
      .collect(),

    TypedStatement::IfN1(label, condition, if_body, else_body) => codegen::if_n1_statement(
      label,
      condition,
      *if_body,
      else_body.map(|else_body| *else_body),
    ),

    TypedStatement::WhileN1(label, condition, body) => {
      codegen::while_n1_statement(label, condition, *body)
    }

    TypedStatement::MacroReturnN0(parameters_size, locals_size, expression) => {
      match (parameters_size, locals_size, expression) {
        (parameters_size, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::n0_expression(expression, 0))
          .chain(vec![Ok(Token::Pop); parameters_size + locals_size])
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
        (parameters_size, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); parameters_size + locals_size])
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
      }
    }

    TypedStatement::MacroReturnN1(parameters_size, locals_size, expression)
    | TypedStatement::MacroReturnN8(parameters_size, locals_size, expression) => {
      match (parameters_size, locals_size, expression) {
        (0, 0, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
        (parameters_size, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![Ok(Token::StO(
            (parameters_size + locals_size - 1) as u8,
          ))])
          .chain(vec![Ok(Token::Pop); parameters_size + locals_size - 1])
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
        (0, 0, None) => std::iter::empty()
          .chain(vec![Ok(Token::XXX(0x00))])
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
        (parameters_size, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); parameters_size + locals_size - 1])
          .chain(vec![
            Ok(Token::LabelRef(codegen::ret_label!())),
            Ok(Token::MacroRef(link::jmp_macro!())),
          ])
          .collect(),
      }
    }

    TypedStatement::FunctionReturnN0(parameters_size, locals_size, expression) => {
      match (parameters_size, locals_size, expression) {
        (0, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (0, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::n0_expression(expression, 0))
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (parameters_size, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::StO((parameters_size - 1) as u8))])
          .chain(vec![Ok(Token::Pop); parameters_size - 1])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (parameters_size, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::n0_expression(expression, 0))
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::StO((parameters_size - 1) as u8))])
          .chain(vec![Ok(Token::Pop); parameters_size - 1])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
      }
    }

    TypedStatement::FunctionReturnN1(parameters_size, locals_size, expression)
    | TypedStatement::FunctionReturnN8(parameters_size, locals_size, expression) => {
      match (parameters_size, locals_size, expression) {
        (0, 0, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![Ok(Token::Swp)])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (0, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![Ok(Token::StO((locals_size - 1) as u8))])
          .chain(vec![Ok(Token::Pop); locals_size - 1])
          .chain(vec![Ok(Token::Swp)])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (0, 0, None) => std::iter::empty()
          .chain(vec![Ok(Token::XXX(0x00)), Ok(Token::Swp)])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (0, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); locals_size - 1])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (1, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (1, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![Ok(Token::StO((locals_size + 1) as u8))])
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (parameters_size, locals_size, None) => std::iter::empty()
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::StO((parameters_size - 2) as u8))])
          .chain(vec![Ok(Token::Pop); parameters_size - 2])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
        (parameters_size, locals_size, Some(expression)) => std::iter::empty()
          .chain(codegen::expression(expression, 0))
          .chain(vec![Ok(Token::StO((parameters_size + locals_size) as u8))])
          .chain(vec![Ok(Token::Pop); locals_size])
          .chain(vec![Ok(Token::StO((parameters_size - 2) as u8))])
          .chain(vec![Ok(Token::Pop); parameters_size - 2])
          .chain(vec![Ok(Token::MacroRef(link::ret_macro!()))])
          .collect(),
      }
    }

    TypedStatement::InitLocalN0(_expression) => todo!(),

    TypedStatement::InitLocalN1(_expression) => todo!(),

    TypedStatement::InitLocalN8(_expression) => todo!(),

    TypedStatement::Assembly(assembly) => std::iter::empty().chain(vec![Err(assembly)]).collect(),
  }
}

fn if_n1_statement(
  label: String,
  condition: TypedExpression,
  if_body: TypedStatement,
  else_body: Option<TypedStatement>,
) -> Vec<Result<Token, String>> {
  let (negated, condition) = match condition {
    TypedExpression::N1BitwiseComplement(expression) => (true, *expression),
    _ => (false, condition),
  };

  match condition {
    TypedExpression::N1BitwiseComplement(expression) if negated => {
      codegen::if_n1_statement(label, *expression, if_body, else_body)
    }

    TypedExpression::N1EqualToN8(expression, constant)
    | TypedExpression::N1EqualToN8(constant, expression)
      if *constant == TypedExpression::N8Constant(0x00) =>
    {
      std::iter::empty()
        .chain(codegen::n8_expression(*expression, 0))
        .chain(vec![
          Ok(Token::MacroRef(link::zr_macro!())),
          Ok(Token::LabelRef(codegen::else_label!(&label))),
          Ok(Token::MacroRef(match negated {
            true => link::bcs_macro!(),
            false => link::bcc_macro!(),
          })),
        ])
        .chain(codegen::statement(if_body))
        .chain(vec![
          Ok(Token::LabelRef(codegen::end_label!(&label))),
          Ok(Token::MacroRef(link::jmp_macro!())),
          Ok(Token::LabelDef(codegen::else_label!(&label))),
        ])
        .chain(else_body.map(codegen::statement).unwrap_or_else(Vec::new))
        .chain(vec![Ok(Token::LabelDef(codegen::end_label!(&label)))])
        .collect()
    }

    TypedExpression::N1EqualToN8(expression1, expression2) => std::iter::empty()
      .chain(codegen::n8_expression(*expression1, 0))
      .chain(codegen::n8_expression(*expression2, 1))
      .chain(vec![
        Ok(Token::MacroRef(link::eq_macro!())),
        Ok(Token::LabelRef(codegen::else_label!(&label))),
        Ok(Token::MacroRef(match negated {
          true => link::bcs_macro!(),
          false => link::bcc_macro!(),
        })),
      ])
      .chain(codegen::statement(if_body))
      .chain(vec![
        Ok(Token::LabelRef(codegen::end_label!(&label))),
        Ok(Token::MacroRef(link::jmp_macro!())),
        Ok(Token::LabelDef(codegen::else_label!(&label))),
      ])
      .chain(else_body.map(codegen::statement).unwrap_or_else(Vec::new))
      .chain(vec![Ok(Token::LabelDef(codegen::end_label!(&label)))])
      .collect(),

    TypedExpression::N1CastN8(expression) => std::iter::empty()
      .chain(codegen::n8_expression(*expression, 0))
      .chain(vec![
        Ok(Token::XXX(0x01)),
        Ok(Token::MacroRef(link::cl_macro!())),
        Ok(Token::LabelRef(codegen::else_label!(&label))),
        Ok(Token::MacroRef(match negated {
          true => link::bcc_macro!(),
          false => link::bcs_macro!(),
        })),
      ])
      .chain(codegen::statement(if_body))
      .chain(vec![
        Ok(Token::LabelRef(codegen::end_label!(&label))),
        Ok(Token::MacroRef(link::jmp_macro!())),
        Ok(Token::LabelDef(codegen::else_label!(&label))),
      ])
      .chain(else_body.map(codegen::statement).unwrap_or_else(Vec::new))
      .chain(vec![Ok(Token::LabelDef(codegen::end_label!(&label)))])
      .collect(),

    TypedExpression::N1Constant(constant) if constant == negated => std::iter::empty()
      .chain(else_body.map(codegen::statement).unwrap_or_else(Vec::new))
      .collect(),

    TypedExpression::N1Constant(constant) if constant != negated => std::iter::empty()
      .chain(codegen::statement(if_body))
      .collect(),

    _ => unreachable!(),
  }
}

fn while_n1_statement(
  label: String,
  condition: TypedExpression,
  body: TypedStatement,
) -> Vec<Result<Token, String>> {
  let (negated, condition) = match condition {
    TypedExpression::N1BitwiseComplement(expression) => (true, *expression),
    _ => (false, condition),
  };

  match condition {
    TypedExpression::N1BitwiseComplement(expression) if negated => {
      codegen::while_n1_statement(label, *expression, body)
    }

    TypedExpression::N1EqualToN8(expression, constant)
    | TypedExpression::N1EqualToN8(constant, expression)
      if *constant == TypedExpression::N8Constant(0x00) =>
    {
      std::iter::empty()
        .chain(vec![
          Ok(Token::LabelRef(codegen::cond_label!(&label))),
          Ok(Token::MacroRef(link::jmp_macro!())),
          Ok(Token::LabelDef(codegen::begin_label!(&label))),
        ])
        .chain(codegen::statement(body))
        .chain(vec![Ok(Token::LabelDef(codegen::cond_label!(&label)))])
        .chain(codegen::n8_expression(*expression, 0))
        .chain(vec![
          Ok(Token::MacroRef(link::zr_macro!())),
          Ok(Token::LabelRef(codegen::begin_label!(&label))),
          Ok(Token::MacroRef(match negated {
            true => link::bcc_macro!(),
            false => link::bcs_macro!(),
          })),
        ])
        .collect()
    }

    TypedExpression::N1EqualToN8(expression1, expression2) => std::iter::empty()
      .chain(vec![
        Ok(Token::LabelRef(codegen::cond_label!(&label))),
        Ok(Token::MacroRef(link::jmp_macro!())),
        Ok(Token::LabelDef(codegen::begin_label!(&label))),
      ])
      .chain(codegen::statement(body))
      .chain(vec![Ok(Token::LabelDef(codegen::cond_label!(&label)))])
      .chain(codegen::n8_expression(*expression1, 0))
      .chain(codegen::n8_expression(*expression2, 1))
      .chain(vec![
        Ok(Token::MacroRef(link::eq_macro!())),
        Ok(Token::LabelRef(codegen::begin_label!(&label))),
        Ok(Token::MacroRef(match negated {
          true => link::bcc_macro!(),
          false => link::bcs_macro!(),
        })),
      ])
      .collect(),

    TypedExpression::N1CastN8(expression) => std::iter::empty()
      .chain(vec![
        Ok(Token::LabelRef(codegen::cond_label!(&label))),
        Ok(Token::MacroRef(link::jmp_macro!())),
        Ok(Token::LabelDef(codegen::begin_label!(&label))),
      ])
      .chain(codegen::statement(body))
      .chain(vec![Ok(Token::LabelDef(codegen::cond_label!(&label)))])
      .chain(codegen::n8_expression(*expression, 0))
      .chain(vec![
        Ok(Token::XXX(0x01)),
        Ok(Token::MacroRef(link::cl_macro!())),
        Ok(Token::LabelRef(codegen::begin_label!(&label))),
        Ok(Token::MacroRef(match negated {
          true => link::bcs_macro!(),
          false => link::bcc_macro!(),
        })),
      ])
      .collect(),

    TypedExpression::N1Constant(constant) if constant == negated => std::iter::empty().collect(),

    TypedExpression::N1Constant(constant) if constant != negated => std::iter::empty()
      .chain(vec![Ok(Token::LabelDef(codegen::begin_label!(&label)))])
      .chain(codegen::statement(body))
      .chain(vec![
        Ok(Token::LabelRef(codegen::begin_label!(&label))),
        Ok(Token::MacroRef(link::jmp_macro!())),
      ])
      .collect(),

    _ => unreachable!(),
  }
}

fn expression(expression: TypedExpression, temporaries_size: usize) -> Vec<Result<Token, String>> {
  match expression {
    TypedExpression::N8Negation(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N1BitwiseComplement(_) => codegen::n1_expression(expression, temporaries_size),
    TypedExpression::N8BitwiseComplement(_) => codegen::n8_expression(expression, temporaries_size),

    TypedExpression::N8Addition(_, _) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N8Subtraction(_, _) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::U8Multiplication(_, _) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::U8Division(_, _) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::U8Modulo(_, _) => codegen::n8_expression(expression, temporaries_size),

    TypedExpression::N1EqualToN8(_, _) => codegen::n1_expression(expression, temporaries_size),

    TypedExpression::N0CastN1(_) => codegen::n0_expression(expression, temporaries_size),
    TypedExpression::N0CastN8(_) => codegen::n0_expression(expression, temporaries_size),
    TypedExpression::N1CastN8(_) => codegen::n1_expression(expression, temporaries_size),
    TypedExpression::N0Constant(_) => codegen::n0_expression(expression, temporaries_size),
    TypedExpression::N1Constant(_) => codegen::n1_expression(expression, temporaries_size),
    TypedExpression::N8Constant(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N8GetLocal(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N8AddrLocal(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N8GetGlobal(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N8AddrGlobal(_) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N0MacroCall(_, _) => codegen::n0_expression(expression, temporaries_size),
    TypedExpression::N1MacroCall(_, _) => codegen::n1_expression(expression, temporaries_size),
    TypedExpression::N8MacroCall(_, _) => codegen::n8_expression(expression, temporaries_size),
    TypedExpression::N0FunctionCall(_, _) => codegen::n0_expression(expression, temporaries_size),
    TypedExpression::N1FunctionCall(_, _) => codegen::n1_expression(expression, temporaries_size),
    TypedExpression::N8FunctionCall(_, _) => codegen::n8_expression(expression, temporaries_size),
  }
}

fn n0_expression(
  expression: TypedExpression,
  temporaries_size: usize,
) -> Vec<Result<Token, String>> {
  match expression {
    TypedExpression::N0CastN1(expression) => std::iter::empty()
      .chain(codegen::n1_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::Pop)])
      .collect(),

    TypedExpression::N0CastN8(expression) => std::iter::empty()
      .chain(codegen::n8_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::Pop)])
      .collect(),

    TypedExpression::N0Constant(_constant) => std::iter::empty().collect(),

    TypedExpression::N0MacroCall(label, arguments) => arguments
      .into_iter()
      .enumerate()
      // TODO assumes all expressions are one byte in size
      .flat_map(|(index, expression)| codegen::expression(expression, temporaries_size + index))
      .chain(vec![Ok(Token::MacroRef(link::global_macro!(&label)))])
      .collect(),

    TypedExpression::N0FunctionCall(designator, arguments) => {
      let arguments_size = arguments.len();

      arguments
        .into_iter()
        .enumerate()
        // TODO assumes all expressions are one byte in size
        .flat_map(|(index, expression)| codegen::expression(expression, temporaries_size + index))
        .chain(codegen::n8_expression(
          *designator,
          // TODO assumes all arguments are one byte in size
          temporaries_size + arguments_size,
        ))
        .chain(vec![Ok(Token::MacroRef(link::call_macro!()))])
        .collect()
    }

    _ => unreachable!(),
  }
}

fn n1_expression(
  expression: TypedExpression,
  temporaries_size: usize,
) -> Vec<Result<Token, String>> {
  match expression {
    TypedExpression::N1BitwiseComplement(expression) => std::iter::empty()
      .chain(codegen::n1_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::XXX(0x01)), Ok(Token::Xor)])
      .collect(),

    TypedExpression::N1CastN8(expression) => std::iter::empty()
      .chain(codegen::n8_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::XXX(0x01)), Ok(Token::And)])
      .collect(),

    TypedExpression::N1EqualToN8(expression, constant)
    | TypedExpression::N1EqualToN8(constant, expression)
      if *constant == TypedExpression::N8Constant(0x00) =>
    {
      std::iter::empty()
        .chain(codegen::n8_expression(*expression, temporaries_size))
        .chain(vec![
          Ok(Token::MacroRef(link::zr_macro!())),
          Ok(Token::XXX(0x00)),
          Ok(Token::Shl),
          Ok(Token::AtDyn),
        ])
        .collect()
    }

    TypedExpression::N1EqualToN8(expression1, expression2) => std::iter::empty()
      .chain(codegen::n8_expression(*expression1, temporaries_size))
      .chain(codegen::n8_expression(*expression2, temporaries_size + 1))
      .chain(vec![
        Ok(Token::MacroRef(link::eq_macro!())),
        Ok(Token::XXX(0x00)),
        Ok(Token::Shl),
        Ok(Token::AtDyn),
      ])
      .collect(),

    TypedExpression::N1Constant(constant) => match constant {
      true => vec![Ok(Token::XXX(0x01))],
      false => vec![Ok(Token::XXX(0x00))],
    },

    TypedExpression::N1MacroCall(_, _) => todo!(),

    TypedExpression::N1FunctionCall(_, _) => todo!(),

    _ => unreachable!(),
  }
}

fn n8_expression(
  expression: TypedExpression,
  temporaries_size: usize,
) -> Vec<Result<Token, String>> {
  match expression {
    TypedExpression::N8Negation(expression) => std::iter::empty()
      .chain(codegen::n8_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::Neg)])
      .collect(),

    TypedExpression::N8BitwiseComplement(expression) => std::iter::empty()
      .chain(codegen::n8_expression(*expression, temporaries_size))
      .chain(vec![Ok(Token::Not)])
      .collect(),

    TypedExpression::N8Addition(expression1, expression2) => match (*expression1, *expression2) {
      (expression, TypedExpression::N8Constant(0x01))
      | (TypedExpression::N8Constant(0x01), expression) => std::iter::empty()
        .chain(codegen::n8_expression(expression, temporaries_size))
        .chain(vec![Ok(Token::Inc)])
        .collect(),
      (expression, TypedExpression::N8Constant(0x00))
      | (TypedExpression::N8Constant(0x00), expression) => std::iter::empty()
        .chain(codegen::n8_expression(expression, temporaries_size))
        .collect(),
      (expression, TypedExpression::N8Constant(0xff))
      | (TypedExpression::N8Constant(0xff), expression) => std::iter::empty()
        .chain(codegen::n8_expression(expression, temporaries_size))
        .chain(vec![Ok(Token::Dec)])
        .collect(),
      (expression1, expression2) => std::iter::empty()
        .chain(codegen::n8_expression(expression1, temporaries_size))
        .chain(codegen::n8_expression(expression2, temporaries_size + 1))
        .chain(vec![Ok(Token::Clc), Ok(Token::Add)])
        .collect(),
    },

    TypedExpression::N8Subtraction(expression1, expression2) => {
      match (*expression1, *expression2) {
        (expression, TypedExpression::N8Constant(0x01))
        | (TypedExpression::N8Constant(0x01), expression) => std::iter::empty()
          .chain(codegen::n8_expression(expression, temporaries_size))
          .chain(vec![Ok(Token::Dec)])
          .collect(),
        (expression, TypedExpression::N8Constant(0x00))
        | (TypedExpression::N8Constant(0x00), expression) => std::iter::empty()
          .chain(codegen::n8_expression(expression, temporaries_size))
          .collect(),
        (expression, TypedExpression::N8Constant(0xff))
        | (TypedExpression::N8Constant(0xff), expression) => std::iter::empty()
          .chain(codegen::n8_expression(expression, temporaries_size))
          .chain(vec![Ok(Token::Inc)])
          .collect(),
        (expression1, expression2) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(codegen::n8_expression(expression2, temporaries_size + 1))
          .chain(vec![Ok(Token::Clc), Ok(Token::Sub)])
          .collect(),
      }
    }

    TypedExpression::U8Multiplication(expression1, expression2) => {
      match (*expression1, *expression2) {
        (expression, TypedExpression::N8Constant(0x02))
        | (TypedExpression::N8Constant(0x02), expression) => std::iter::empty()
          .chain(codegen::n8_expression(expression, temporaries_size))
          .chain(vec![Ok(Token::Clc), Ok(Token::Shl)])
          .collect(),
        (expression, TypedExpression::N8Constant(0x01))
        | (TypedExpression::N8Constant(0x01), expression) => std::iter::empty()
          .chain(codegen::n8_expression(expression, temporaries_size))
          .collect(),
        (_expression, TypedExpression::N8Constant(0x00))
        | (TypedExpression::N8Constant(0x00), _expression) => std::iter::empty()
          .chain(vec![Ok(Token::XXX(0x00))])
          .collect(),
        (expression1, expression2) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(codegen::n8_expression(expression2, temporaries_size + 1))
          .chain(vec![Ok(Token::MacroRef(link::mul_macro!()))])
          .collect(),
      }
    }

    TypedExpression::U8Division(expression1, expression2) => {
      match (*expression1, *expression2) {
        (expression1, TypedExpression::N8Constant(0x02)) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(vec![Ok(Token::Clc), Ok(Token::Shr)])
          .collect(),
        (expression1, TypedExpression::N8Constant(0x01)) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .collect(),
        (_expression1, TypedExpression::N8Constant(0x00)) => std::iter::empty().collect(), // behavior is undefined
        (TypedExpression::N8Constant(0x00), _expression) => std::iter::empty()
          .chain(vec![Ok(Token::XXX(0x00))])
          .collect(),
        (expression1, expression2) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(codegen::n8_expression(expression2, temporaries_size + 1))
          .chain(vec![Ok(Token::MacroRef(link::div_macro!()))])
          .collect(),
      }
    }

    TypedExpression::U8Modulo(expression1, expression2) => {
      match (*expression1, *expression2) {
        (expression1, TypedExpression::N8Constant(0x02)) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(vec![Ok(Token::XXX(0x01)), Ok(Token::And)])
          .collect(),
        (_expression1, TypedExpression::N8Constant(0x01)) => std::iter::empty()
          .chain(codegen::n8_expression(TypedExpression::N8Constant(0x00), 0))
          .collect(),
        (_expression1, TypedExpression::N8Constant(0x00)) => std::iter::empty().collect(), // behavior is undefined
        (TypedExpression::N8Constant(0x00), _expression) => std::iter::empty()
          .chain(vec![Ok(Token::XXX(0x00))])
          .collect(),
        (expression1, expression2) => std::iter::empty()
          .chain(codegen::n8_expression(expression1, temporaries_size))
          .chain(codegen::n8_expression(expression2, temporaries_size + 1))
          .chain(vec![Ok(Token::MacroRef(link::mod_macro!()))])
          .collect(),
      }
    }

    TypedExpression::N8Constant(constant) => vec![Ok(Token::XXX(constant))],

    // TODO assumes the stack contains no temporaries
    TypedExpression::N8GetLocal(offset) => std::iter::empty()
      .chain(vec![Ok(Token::LdO((offset + temporaries_size) as u8))])
      .collect(),

    TypedExpression::N8AddrLocal(_offset) => todo!(),

    TypedExpression::N8GetGlobal(label) => std::iter::empty()
      .chain(vec![Ok(Token::LabelRef(link::global_label!(&label)))])
      .chain(vec![Ok(Token::Lda)])
      .collect(),

    TypedExpression::N8AddrGlobal(label) => std::iter::empty()
      .chain(vec![Ok(Token::LabelRef(link::global_label!(&label)))])
      .collect(),

    TypedExpression::N8MacroCall(label, arguments) => arguments
      .into_iter()
      .enumerate()
      // TODO assumes all expressions are one byte in size
      .flat_map(|(index, expression)| codegen::expression(expression, temporaries_size + index))
      .chain(vec![Ok(Token::MacroRef(link::global_macro!(&label)))])
      .collect(),

    TypedExpression::N8FunctionCall(designator, arguments) => {
      let arguments_size = arguments.len();

      arguments
        .into_iter()
        .enumerate()
        // TODO assumes all expressions are one byte in size
        .flat_map(|(index, expression)| codegen::expression(expression, temporaries_size + index))
        .chain(codegen::n8_expression(
          *designator,
          // TODO assumes all arguments are one byte in size
          temporaries_size + arguments_size,
        ))
        .chain(vec![Ok(Token::MacroRef(link::call_macro!()))])
        .collect()
    }

    _ => unreachable!(),
  }
}
