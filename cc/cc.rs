#[path = "../misc/common/common.rs"]
mod common;
use common::*;
use std::collections::HashMap;

mod codegen;
mod link;
mod parse;
mod preprocess;
mod typecheck;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 {
    println!("CC: Usage: cc <C source files> <assembly output file>");
    std::process::exit(1);
  }

  let mut errors: Vec<(Pos, Error)> = vec![];
  let c_source_files = args[1..args.len() - 1].to_vec();
  let assembly_output_file = &args[args.len() - 1];

  let preprocessed: Vec<String> = c_source_files
    .into_iter()
    .map(|c_source_file| File(c_source_file))
    .map(|c_source_file| {
      [
        format!("\nasm {{ # translation {} }}\n", c_source_file.clone()),
        preprocess::preprocess(c_source_file, &mut HashMap::new(), &mut errors, None),
      ]
    })
    .flatten()
    .collect();

  // println!("CC: Preprocessed: {:#?}", preprocessed);

  let parsed: Vec<Program> = preprocessed
    .into_iter()
    .map(|preprocessed| parse::parse(preprocessed, &mut errors))
    .collect();

  // println!("CC: Parsed: {:#?}", parsed);

  let typechecked: Vec<TypedProgram> = parsed
    .into_iter()
    .map(|program| typecheck::typecheck(program, &mut errors))
    .collect();

  // println!("CC: Typechecked: {:#?}", typechecked);

  let linked: Vec<Result<Token, String>> = std::iter::empty()
    .chain([Err(format!("# dependency graph"))])
    .chain(link::link(
      &TypedProgram(typechecked.iter().cloned().flat_map(|p| p.0).collect()),
      &mut errors,
    ))
    .collect();

  // println!("CC: Linked: {:#?}", linked);

  let codegened: Vec<Vec<Result<Token, String>>> = typechecked
    .into_iter()
    .map(|typed_program| codegen::codegen(typed_program, &mut errors))
    .collect();

  // println!("CC: Codegened: {:#?}", codegened);

  let tokens: Vec<Result<Token, String>> = codegened.into_iter().flatten().chain(linked).collect();

  let mnemonics: Vec<Result<Mnemonic, String>> = tokens
    .into_iter()
    .map(|token| token.map(common::token_to_mnemonic))
    .collect();

  let assembly: String = mnemonics
    .into_iter()
    .map(|mnemonic| match mnemonic {
      Ok(mnemonic) => format!("{} ", mnemonic),
      Err(assembly) => format!("{}\n", assembly),
    })
    .collect::<String>()
    .replace(" \n", "\n");

  let assembly = format!("# Generated by CC\n\n{}", assembly);

  // println!("CC: Assembly: {:#?}", assembly);

  match errors[..] {
    [] => std::fs::write(assembly_output_file, assembly).unwrap(),
    _ => {
      let errors = errors
        .iter()
        .map(|(pos, error)| format!("CC: Error: {}: {}", pos, error))
        .collect::<Vec<String>>()
        .join("\n");

      println!("{}", errors);
      std::process::exit(1);
    }
  }

  println!("CC: Done");
}

// abstract syntax tree

#[derive(Clone, PartialEq, Debug)]
pub struct Object(Type, String);

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
  Void,
  Bool,
  Char,
  SignedChar,
  UnsignedChar,
  Short,
  UnsignedShort,
  Int,
  UnsignedInt,
  Long,
  UnsignedLong,
  LongLong,
  UnsignedLongLong,
  Array(Box<Type>),
  Structure(Vec<Object>),
  Union(Vec<Object>),
  Enumeration(Vec<String>),
  Macro(Box<Type>, String, Vec<Type>, bool), // not using `Box<Object>` because pattern matching
  Function(Box<Type>, Vec<Type>, bool),
  Pointer(Box<Type>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Program(Vec<Global>);

#[derive(Clone, PartialEq, Debug)]
pub enum Global {
  FunctionDeclaration(bool, Object, Vec<Object>, bool),
  FunctionDefinition(bool, Object, Vec<Object>, bool, Statement),
  GlobalDeclaration(Object),
  GlobalDefinition(Object, Expression),
  GlobalAssembly(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
  Positive(Box<Expression>),
  Negation(Box<Expression>),
  LogicalNegation(Box<Expression>),
  BitwiseComplement(Box<Expression>),

  Addition(Box<Expression>, Box<Expression>),
  Subtraction(Box<Expression>, Box<Expression>),
  Multiplication(Box<Expression>, Box<Expression>),
  Division(Box<Expression>, Box<Expression>),
  Modulo(Box<Expression>, Box<Expression>),
  LogicalAnd(Box<Expression>, Box<Expression>),
  LogicalOr(Box<Expression>, Box<Expression>),
  BitwiseAnd(Box<Expression>, Box<Expression>),
  BitwiseExclusiveOr(Box<Expression>, Box<Expression>),
  BitwiseInclusiveOr(Box<Expression>, Box<Expression>),
  LeftShift(Box<Expression>, Box<Expression>),
  RightShift(Box<Expression>, Box<Expression>),

  EqualTo(Box<Expression>, Box<Expression>),
  NotEqualTo(Box<Expression>, Box<Expression>),
  LessThan(Box<Expression>, Box<Expression>),
  LessThanOrEqualTo(Box<Expression>, Box<Expression>),
  GreaterThan(Box<Expression>, Box<Expression>),
  GreaterThanOrEqualTo(Box<Expression>, Box<Expression>),

  Conditional(Box<Expression>, Box<Expression>, Box<Expression>),

  Cast(Type, Box<Expression>),
  IntegerConstant(u8),
  CharacterConstant(char),
  StringLiteral(String),
  Identifier(String),
  FunctionCall(Box<Expression>, Vec<Expression>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Statement {
  Expression(Expression),
  Compound(Vec<Statement>),
  If(Expression, Box<Statement>, Option<Box<Statement>>),
  While(Expression, Box<Statement>),
  Return(Option<Expression>),
  Assembly(String),
}

// typed intermediate representation

#[derive(Clone, PartialEq, Debug)]
pub struct TypedProgram(Vec<TypedGlobal>);

#[derive(Clone, PartialEq, Debug)]
pub enum TypedGlobal {
  Data(String, Vec<TypedExpression>),
  Macro(String, TypedStatement),
  Function(String, TypedStatement),
  Assembly(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum TypedExpression {
  N1BitwiseComplement(Box<TypedExpression>),
  N8BitwiseComplement(Box<TypedExpression>),

  N8Addition(Box<TypedExpression>, Box<TypedExpression>),
  N8Subtraction(Box<TypedExpression>, Box<TypedExpression>),
  U8Multiplication(Box<TypedExpression>, Box<TypedExpression>),
  U8Division(Box<TypedExpression>, Box<TypedExpression>),
  U8Modulo(Box<TypedExpression>, Box<TypedExpression>),

  N1EqualToN8(Box<TypedExpression>, Box<TypedExpression>),
  N1LessThanU8(Box<TypedExpression>, Box<TypedExpression>),

  N0CastN1(Box<TypedExpression>),
  N0CastN8(Box<TypedExpression>),
  N1CastN8(Box<TypedExpression>),
  N0Constant(()),
  N1Constant(bool),
  N8Constant(u8),
  N8GetLocal(usize), // TODO document: offset from last local variable
  N8AddrLocal(usize),
  N8GetGlobal(String),
  N8AddrGlobal(String),
  N0MacroCall(String, Vec<TypedExpression>),
  N1MacroCall(String, Vec<TypedExpression>),
  N8MacroCall(String, Vec<TypedExpression>),
  N0FunctionCall(Box<TypedExpression>, Vec<TypedExpression>),
  N1FunctionCall(Box<TypedExpression>, Vec<TypedExpression>),
  N8FunctionCall(Box<TypedExpression>, Vec<TypedExpression>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum TypedStatement {
  ExpressionN0(TypedExpression),
  Compound(Vec<TypedStatement>),
  IfN1(
    String,
    TypedExpression,
    Box<TypedStatement>,
    Option<Box<TypedStatement>>,
  ),
  WhileN1(String, TypedExpression, Box<TypedStatement>),
  MacroReturnN0(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  MacroReturnN1(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  MacroReturnN8(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  FunctionReturnN0(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  FunctionReturnN1(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  FunctionReturnN8(usize, usize, Option<TypedExpression>), // TODO document: param count, local count
  InitLocalN0(Option<TypedExpression>),
  InitLocalN1(Option<TypedExpression>),
  InitLocalN8(Option<TypedExpression>),
  Assembly(String),
}
