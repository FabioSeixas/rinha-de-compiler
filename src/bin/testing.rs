use clap::Parser;
use miette::IntoDiagnostic;
use rinha::{ast::*, Command};
use std::fs;

fn main2() {
    let command = Command::parse();
    let file = fs::read_to_string(&command.main).into_diagnostic().unwrap();
    let ast: File = serde_json::from_str(&file).unwrap();
    // println!("{:?}", ast.expression);

    let interpreter = Interpreter::new(ast.expression);

    interpreter.interpret();

    // evaluate(ast.expression);
}

struct Interpreter {
    ast: Term,
}

impl Interpreter {
    fn new(ast: Term) -> Interpreter {
        Interpreter { ast }
    }
    fn interpret(&self) {
        evaluate(&self.ast);
    }
}

trait MyTraits {
    fn print(&self);
    fn logic_operation<T>(&self, second: T, op: BinaryOp) -> bool;
}

impl MyTraits for Primitive {
    fn print(&self) {
        println!("{self}");
    }
    fn logic_operation<T>(&self, second: T, op: BinaryOp) -> bool {
        match op {
            BinaryOp::Eq => second.eq(self),
            BinaryOp::Neq => *self != second,
            BinaryOp::Lt => *self < second,
            BinaryOp::Gt => *self > second,
            BinaryOp::Lte => *self <= second,
            BinaryOp::Gte => *self >= second,
            _ => false
        }
    }
}

impl MyTraits for String {
    fn print(&self) {
        println!("{self}");
    }
}

enum Primitive {
    Str(String),
    Int(i32),
    Bool(bool),
    None,
}

impl Primitive {
    fn extract_str(&self) -> String {
        match self {
            Primitive::Str(v) => v.clone(),
            _ => panic!("attempt to extract a string for a non-string Primitive"),
        }
    }
    fn extract_int(&self) -> i32 {
        match self {
            Primitive::Int(v) => *v,
            _ => panic!("attempt to extract an int for a non-int Primitive"),
        }
    }
    fn extract_bool(&self) -> bool {
        match self {
            Primitive::Bool(v) => *v,
            _ => panic!("attempt to extract a bool for a non-boolean Primitive"),
        }
    }
}

fn evaluate(t: &Term) -> Primitive {
    match t {
        // Term::Let(v) => {
        //     println!("Um LET");
        //     println!("value: {:?}", v.value);
        //     println!("location: {:?}", v.location);
        //     evaluate(*v.value)
        // }
        Term::Print(v) => {
            // println!("Um PRINT");
            // println!("value: {:?}", v.value);
            // println!("location: {:?}", v.location);
            let value_to_print = evaluate(&v.value);
            match value_to_print {
                Primitive::Int(v) => {
                    v.print();
                }
                Primitive::Str(v) => {
                    v.print();
                }
                _ => {}
            }
            Primitive::None
        }
        Term::Binary(v) => {
            // println!("Um BINARY");
            // println!("left: {:?}", v.lhs);
            // println!("right: {:?}", v.rhs);
            // println!("operation: {:?}", v.op);
            // println!("location: {:?}", v.location);
            let left = evaluate(&v.lhs).extract_int();
            let right = evaluate(&v.rhs).extract_int();
            match v.op {
                BinaryOp::Add => {
                    // println!("adding {} and {}", left, right);
                    Primitive::Int(left + right)
                }
                BinaryOp::Mul => {
                    // println!("multiplying {} by {}", left, right);
                    Primitive::Int(left * right)
                }
                BinaryOp::Div => {
                    // println!("dividing {} by {}", left, right);
                    Primitive::Int(left / right)
                }
                BinaryOp::Sub => {
                    // println!("subtracting {} by {}", left, right);
                    Primitive::Int(left - right)
                }
                BinaryOp::Rem => Primitive::Int(left % right),
                BinaryOp::Eq => Primitive::Bool(left == right),
                BinaryOp::Neq => Primitive::Bool(left != right),
                BinaryOp::Lt => Primitive::Bool(left < right),
                BinaryOp::Gt => Primitive::Bool(left > right),
                BinaryOp::Lte => Primitive::Bool(left <= right),
                BinaryOp::Gte => Primitive::Bool(left >= right),
                BinaryOp::And => Primitive::Bool(&&right),
                _ => return Primitive::None,
            }
        }
        Term::Str(v) => {
            // println!("Um STR");
            // println!("value: {:?}", v.value);
            // println!("location: {:?}", v.location);
            Primitive::Str(v.value.clone())
        }
        Term::Int(v) => {
            // println!("Um INT");
            // println!("value: {:?}", v.value);
            // println!("location: {:?}", v.location);
            Primitive::Int(v.value.clone())
        }
        Term::Bool(v) => {
            // println!("Um INT");
            // println!("value: {:?}", v.value);
            // println!("location: {:?}", v.location);
            Primitive::Bool(v.value.clone())
        }
        v => {
            // println!("other");
            // println!("{:?}", v);
            Primitive::None
            // return String::from("other")
        }
    }
}
