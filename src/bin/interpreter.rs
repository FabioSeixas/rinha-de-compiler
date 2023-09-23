use clap::Parser;
use miette::IntoDiagnostic;
use rinha::{ast, parser, Command};
use std::{collections, fs, time::Instant};

fn main() {
    let command = Command::parse();
    let file = fs::read_to_string(&command.main).into_diagnostic().unwrap();
    let ast: ast::File = serde_json::from_str(&file).unwrap();
    let time = Instant::now();
    let mut interpreter = Interpreter::new();

    let mut global_scope = collections::HashMap::new();
    interpreter.interpret(ast.expression, &mut global_scope);
    // println!("{}", time.elapsed().as_secs_f32());
}

#[derive(Debug, Clone)]
enum Primitive {
    Str(String),
    Int(i32),
    Bool(bool),
    Var((String, Box<Primitive>)),
    Function {
        name: String,
        parameters: Vec<String>,
        value: ast::Term,
        outside_scope_var_names: Vec<String>,
    },
    Tuple([Box<Primitive>; 2]),
    None,
}

type Scope = collections::HashMap<String, Primitive>;

struct Interpreter {
    memo: Scope,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            memo: collections::HashMap::new(),
        }
    }
    fn interpret(&mut self, ast: ast::Term, scope: &mut Scope) {
        self.visit(ast, scope);
    }
    fn visit(&mut self, term: ast::Term, scope: &mut Scope) -> Primitive {
        match term {
            ast::Term::Int(v) => self.visit_int(v, scope),
            ast::Term::Str(v) => self.visit_str(v, scope),
            ast::Term::Bool(v) => self.visit_bool(v, scope),
            ast::Term::Binary(v) => self.visit_bin_op(v, scope),
            ast::Term::Let(v) => self.visit_let(v, scope),
            ast::Term::Var(v) => self.visit_var(v, scope),
            ast::Term::Print(v) => self.visit_print(v, scope),
            ast::Term::Function(v) => self.visit_function(v, scope),
            ast::Term::Call(v) => self.visit_call(v, scope),
            ast::Term::If(v) => self.visit_conditional(v, scope),
            ast::Term::Tuple(v) => self.visit_tuple(v, scope),
            _ => Primitive::None,
        }
    }
    fn visit_bin_op(&mut self, binary: ast::Binary, scope: &mut Scope) -> Primitive {
        let left = self.visit(*binary.lhs, scope);
        let right = self.visit(*binary.rhs, scope);
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
        }
    }
    fn visit_let(&mut self, let_param: ast::Let, scope: &mut Scope) -> Primitive {
        let raw_var_value = self.visit(*let_param.value, scope);
        match raw_var_value {
            Primitive::Function {
                name: _,
                parameters,
                value,
                outside_scope_var_names,
            } => {
                let current_vars_in_scope: Vec<String> = scope.keys().cloned().collect();
                let function_value = Primitive::Function {
                    parameters,
                    value,
                    outside_scope_var_names: current_vars_in_scope,
                    name: let_param.name.text.clone(),
                };
                scope.insert(let_param.name.text, function_value);
            }
            other_primitive_value => {
                scope.insert(let_param.name.text, other_primitive_value);
            }
        }
        self.visit(*let_param.next, scope)
    }
    fn visit_var(&mut self, var: parser::Var, scope: &Scope) -> Primitive {
        // println!("visit var: {:?}", var);
        let var_stored_opt = scope.get(&var.text);
        if let Some(var_stored) = var_stored_opt {
            var_stored.clone()
        } else {
            panic!(
                "{}",
                format!("Variable \"{}\" not found in the scope", &var.text)
            );
        }
    }
    fn visit_function(&mut self, func: ast::Function, scope: &Scope) -> Primitive {
        // println!("visit function: {:?}", func);
        let mut parameters: Vec<String> = Vec::new();
        for param in func.parameters {
            parameters.push(param.text);
        }

        let func_value = *func.value;

        Primitive::Function {
            name: String::from(""),
            value: func_value,
            outside_scope_var_names: vec![],
            parameters,
        }
    }
    fn visit_call(&mut self, call: ast::Call, scope: &mut Scope) -> Primitive {
        let function = self.visit(*call.callee, scope);
        if let Primitive::Function {
            name,
            parameters,
            value,
            outside_scope_var_names,
        } = function
        {
            // println!("{:?}", outside_scope_var_names);
            let mut func_call_key = String::from(name);
            let mut local_scope: Scope = collections::HashMap::new();

            local_scope.insert(func_call_key.clone(), Primitive::Function {
                value: value.clone(),
                outside_scope_var_names: outside_scope_var_names.clone(),
                name: func_call_key.clone(),
                parameters: parameters.clone()
            });

            for var_name in outside_scope_var_names {
                match scope.get(&var_name) {
                    Some(value) => {
                        local_scope.insert(var_name, value.clone());
                    }
                    None => {}
                }
            }
            for (name, param_value) in parameters.into_iter().zip(call.arguments) {
                let evaluated_param_value = self.visit(param_value, scope);
                local_scope.insert(name.clone(), evaluated_param_value.clone());

                match evaluated_param_value {
                    Primitive::Str(value) => {
                        func_call_key.push_str(&format!(",{}:{}", &name, value));
                    }
                    Primitive::Int(value) => {
                        func_call_key.push_str(&format!(",{}:{}", &name, value));
                    }
                    Primitive::Bool(value) => {
                        func_call_key.push_str(&format!(",{}:{}", &name, value));
                    }
                    _ => {}
                }
            }
            let existing_memo_item = self.memo.get(&func_call_key);
            if let Some(memoization) = existing_memo_item {
                return memoization.clone();
            } else {
                let function_result = self.visit(value, &mut local_scope);
                self.memo.insert(func_call_key, function_result.clone());
                return function_result;
            }
        }
        Primitive::None
    }
    fn visit_conditional(&mut self, conditional: ast::If, scope: &mut Scope) -> Primitive {
        if let Primitive::Bool(condition_result) = self.visit(*conditional.condition, scope) {
            if condition_result {
                return self.visit(*conditional.then, scope);
            } else {
                return self.visit(*conditional.otherwise, scope);
            }
        }
        Primitive::None
    }
    fn visit_int(&self, int: ast::Int, scope: &Scope) -> Primitive {
        Primitive::Int(int.value)
    }
    fn visit_bool(&self, bool: ast::Bool, scope: &Scope) -> Primitive {
        Primitive::Bool(bool.value)
    }
    fn visit_str(&self, str: ast::Str, scope: &Scope) -> Primitive {
        Primitive::Str(str.value)
    }
    fn visit_tuple(&mut self, tuple: ast::Tuple, scope: &mut Scope) -> Primitive {
        let first = self.visit(*tuple.first, scope);
        let second = self.visit(*tuple.second, scope);
        Primitive::Tuple([Box::new(first), Box::new(second)])
    }
    fn visit_print(&mut self, print: ast::Print, scope: &mut Scope) -> Primitive {
        let result = self.visit(*print.value, scope);
        match result {
            Primitive::Str(v) => println!("{v}"),
            Primitive::Int(v) => println!("{v}"),
            Primitive::Bool(v) => println!("{v}"),
            Primitive::Function {
                name,
                parameters,
                value,
                outside_scope_var_names,
            } => println!("<#closure>"),
            Primitive::Tuple(original_tuple) => {
                let print_tuple = get_tuple_string(original_tuple);

                println!("{print_tuple}")
            }
            _ => {}
        }
        Primitive::None
    }
}

fn get_tuple_string(original_tuple: [Box<Primitive>; 2]) -> String {
    let mut print_tuple = String::from("(");

    for (index, value) in original_tuple.into_iter().enumerate() {
        match *value {
            Primitive::Str(v) => print_tuple.push_str(&v),
            Primitive::Int(v) => print_tuple.push_str(&v.to_string()),
            Primitive::Bool(v) => print_tuple.push_str(&v.to_string()),
            Primitive::Function {
                name,
                parameters,
                value,
                outside_scope_var_names,
            } => print_tuple.push_str("<#closure>"),
            Primitive::Tuple(v) => {
                let inner_print_tuple = get_tuple_string(v);
                print_tuple.push_str(&inner_print_tuple);
            }
            _ => {}
        }

        if index < 1 {
            print_tuple.push_str(", ");
        }
    }

    print_tuple.push(')');

    print_tuple
}

fn add_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int + p2_int),
            Primitive::Str(p2_str) => {
                let mut result = String::from(p1_int.to_string());
                result.push_str(&p2_str);
                Primitive::Str(result)
            }
            Primitive::Var(p2_var) => add_two_primitives(p1, *p2_var.1),
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
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int - p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn mul_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int * p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn div_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int / p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn rem_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int % p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn eq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int == p2_int),
            _ => Primitive::None,
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str == p2_str),
            _ => Primitive::None,
        },
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool == p2_bool),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn neq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int != p2_int),
            _ => Primitive::None,
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str != p2_str),
            _ => Primitive::None,
        },
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool != p2_bool),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn lt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int < p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn gt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int > p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn lte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int <= p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn gte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int >= p2_int),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn and_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool && p2_bool),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}

fn or_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool || p2_bool),
            _ => Primitive::None,
        },
        _ => Primitive::None,
    }
}
