use clap::Parser;
use miette::IntoDiagnostic;
use rinha::{ast, Command, parser};
use std::{collections, fs};

fn main() {
    let command = Command::parse();
    let file = fs::read_to_string(&command.main).into_diagnostic().unwrap();
    let ast: ast::File = serde_json::from_str(&file).unwrap();
    // println!("{:?}", ast.expression);

    let mut interpreter = Interpreter::new();

    interpreter.visit(ast.expression);
}
#[derive(Debug, Clone)]
enum Primitive {
    Str(String),
    Int(i32),
    Bool(bool),
    Var((String, Box<Primitive>)),
    None,
}

struct Interpreter {
    terms_map: collections::HashMap<String, Primitive>,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter { terms_map: collections::HashMap::new() }
    }
    fn visit(&mut self, term: ast::Term) -> Primitive {
        match term {
            ast::Term::Int(v) => self.visit_int(v),
            ast::Term::Str(v) => self.visit_str(v),
            ast::Term::Bool(v) => self.visit_bool(v),
            ast::Term::Binary(v) => self.visit_bin_op(v),
            ast::Term::Let(v) => self.visit_let(v),
            ast::Term::Var(v) => self.visit_var(v),
            ast::Term::Print(v) => self.visit_print(v),
            _ => Primitive::None,
        }
    }
    fn visit_bin_op(&mut self, binary: ast::Binary) -> Primitive {
        let left = self.visit(*binary.lhs);
        let right = self.visit(*binary.rhs);
        match binary.op {
            ast::BinaryOp::Add => add_two_primitives(left, right),
            ast::BinaryOp::Sub => sub_two_primitives(left, right),
            ast::BinaryOp::Mul => mul_two_primitives(left, right),
            ast::BinaryOp::Div => div_two_primitives(left, right),
            ast::BinaryOp::Rem => rem_two_primitives(left, right),
            ast::BinaryOp::Eq => eq_two_primitives(left, right),
            ast::BinaryOp::Neq => neq_two_primitives(left, right),
            ast::BinaryOp::Lt => lt_two_primitives(left, right),
            ast::BinaryOp::Gt => gt_two_primitives(left, right),
            ast::BinaryOp::Lte => lte_two_primitives(left, right),
            ast::BinaryOp::Gte => gte_two_primitives(left, right),
            ast::BinaryOp::And => and_two_primitives(left, right),
            ast::BinaryOp::Or => or_two_primitives(left, right),
            _ => Primitive::None,
        }
    }
    fn visit_let(&mut self, let_param: ast::Let) -> Primitive {
        let var_value = self.visit(*let_param.value);
        println!("var value: {:?}", var_value);
        self.terms_map.insert(let_param.name.text, var_value);
        println!("hash map: {:?}", self.terms_map);
        self.visit(*let_param.next)
    }
    fn visit_var(&mut self, var: parser::Var) -> Primitive {
        let var_stored_opt = self.terms_map.get(&var.text);
        if let Some(var_stored) = var_stored_opt {
            var_stored.clone()
        } else {
            Primitive::None
        }
    }
    fn visit_int(&self, int: ast::Int) -> Primitive {
        Primitive::Int(int.value)
    }
    fn visit_bool(&self, bool: ast::Bool) -> Primitive {
        Primitive::Bool(bool.value)
    }
    fn visit_str(&self, str: ast::Str) -> Primitive {
        Primitive::Str(str.value)
    }
    fn visit_print(&mut self, print: ast::Print) -> Primitive {
        let result = self.visit(*print.value);
        match result {
            Primitive::Str(v) => println!("{v}"),
            Primitive::Int(v) => println!("{v}"),
            Primitive::Bool(v) => println!("{v}"),
            _ => {}
        }
        Primitive::None
    }
}

fn add_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int + p2_int),
            Primitive::Str(p2_str) => {
                let mut result = String::from(p1_int.to_string());
                result.push_str(&p2_str);
                Primitive::Str(result)
            }
            Primitive::Var(p2_var) => {
                println!("p2_var: {:?}", p2_var);
                add_two_primitives(p1, *p2_var.1)
            }
           _ => Primitive::None,
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Int(p2_int) => {
                let mut result = String::from(p1_str);
                result.push_str(&p2_int.to_string());
                Primitive::Str(result)
            }
            Primitive::Str(p2_str) => {
                let mut result = String::from(p1_str);
                result.push_str(&p2_str);
                Primitive::Str(result)
            }
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn sub_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int - p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn mul_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int * p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn div_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int / p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn rem_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int % p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn eq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int == p2_int),
            _ => Primitive::None
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str == p2_str),
            _ => Primitive::None
        }
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool == p2_bool),
            _ => Primitive::None
        }
        _ => Primitive::None,
    }
}

fn neq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int != p2_int),
            _ => Primitive::None
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str != p2_str),
            _ => Primitive::None
        }
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool != p2_bool),
            _ => Primitive::None
        }
        _ => Primitive::None,
    }
}

fn lt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int < p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn gt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int > p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn lte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int <= p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn gte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int >= p2_int),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn and_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool && p2_bool),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

fn or_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    println!("p1: {:?}", p1);
    println!("p2: {:?}", p2);
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool || p2_bool),
            _ => Primitive::None
        },
        _ => Primitive::None,
    }
}

