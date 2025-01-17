use clap::Parser;
use miette::IntoDiagnostic;
use rinha::{ast, parser, Command};
use std::{collections, fs, time::Instant};

fn main() {
    let command = Command::parse();
    let file = fs::read_to_string(&command.main).into_diagnostic().unwrap();
    let ast: ast::File = serde_json::from_str(&file).unwrap();
    // let time = Instant::now();
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
        env: Scope,
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
            ast::Term::First(v) => self.visit_first(v, scope),
            ast::Term::Second(v) => self.visit_second(v, scope),
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
                env,
            } => {
                let mut new_scope = scope.clone();
                for (key, value) in env {
                    new_scope.insert(key, value);
                }
                let function_value = Primitive::Function {
                    parameters,
                    value,
                    env: new_scope,
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
        let mut parameters: Vec<String> = Vec::new();
        for param in func.parameters {
            parameters.push(param.text);
        }

        Primitive::Function {
            name: String::from(""),
            value: *func.value,
            env: scope.clone(),
            parameters,
        }
    }
    fn visit_call(&mut self, call: ast::Call, scope: &mut Scope) -> Primitive {
        let function = self.visit(*call.callee, scope);
        if let Primitive::Function {
            name,
            parameters,
            value,
            env,
        } = function
        {
            if call.arguments.len() != parameters.len() {
                panic!(
                    "Function \"{}\" expect \"{}\" parameters.",
                    name,
                    parameters.len()
                )
            }

            let mut func_call_key = String::from(&name);

            let mut local_scope = env.clone();

            local_scope.insert(
                func_call_key.clone(),
                Primitive::Function {
                    value: value.clone(),
                    env: local_scope.clone(),
                    name: func_call_key.clone(),
                    parameters: parameters.clone(),
                },
            );

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
        panic!("The condition inside 'if' must evaluate to Bool")
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
    fn visit_first(&mut self, first: ast::First, scope: &mut Scope) -> Primitive {
        match *first.value {
            ast::Term::Tuple(v) => self.visit(*v.first, scope),
            _ => {
                panic!("\"First\" keyword must be used on Tuples")
            }
        }
    }
    fn visit_second(&mut self, second: ast::Second, scope: &mut Scope) -> Primitive {
        match *second.value {
            ast::Term::Tuple(v) => self.visit(*v.second, scope),
            _ => {
                panic!("\"First\" keyword must be used on Tuples")
            }
        }
    }
    fn visit_print(&mut self, print: ast::Print, scope: &mut Scope) -> Primitive {
        let result = self.visit(*print.value, scope);
        match &result {
            Primitive::Str(v) => print!("{v}\n"),
            Primitive::Int(v) => print!("{v}\n"),
            Primitive::Bool(v) => print!("{v}\n"),
            Primitive::Function {
                name,
                parameters,
                value,
                env,
            } => print!("<#closure>\n"),
            Primitive::Tuple(original_tuple) => {
                let print_tuple = get_tuple_string(original_tuple.clone());

                print!("{print_tuple}\n")
            }
            _ => {}
        }
        result
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
                env,
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
            _ => panic!("Int can only be sum with Int and Str"),
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
            _ => panic!("Str can only be sum with Int and Str"),
        },
        _ => panic!("Sum operation can only be done between Int and Str"),
    }
}

fn sub_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int - p2_int),
            _ => panic!("You can only subtract Int by another Int"),
        },
        _ => panic!("Subtract operation can only be done between two Int"),
    }
}

fn mul_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int * p2_int),
            _ => panic!("You can only multiply Int by another Int"),
        },
        _ => panic!("Multiplication operation can only be done between two Int"),
    }
}

fn div_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int / p2_int),
            _ => panic!("You can only divide Int by another Int"),
        },
        _ => panic!("Divide operation can only be done between two Int"),
    }
}

fn rem_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Int(p1_int % p2_int),
            _ => panic!("You can only remainder Int by another Int"),
        },
        _ => panic!("Remainder operation can only be done between two Int"),
    }
}

fn eq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int == p2_int),
            _ => panic!("You can only test equality of Int by another Int"),
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str == p2_str),
            _ => panic!("You can only test equality of Str by another Str"),
        },
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool == p2_bool),
            _ => panic!("You can only test equality of Bool by another Bool"),
        },
        _ => panic!("Equality operation can only be done between Int, Str and Bool"),
    }
}

fn neq_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int != p2_int),
            _ => panic!("You can only test inequality of Int by another Int"),
        },
        Primitive::Str(p1_str) => match p2 {
            Primitive::Str(p2_str) => Primitive::Bool(p1_str != p2_str),
            _ => panic!("You can only test inequality of Str by another Str"),
        },
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool != p2_bool),
            _ => panic!("You can only test inequality of Bool by another Bool"),
        },
        _ => panic!("Inequality operation can only be done between Int, Str and Bool"),
    }
}

fn lt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int < p2_int),
            _ => panic!("You can only test 'lower than' of Int by another Int"),
        },
        _ => panic!("'Lower than' test operator can only be done with Int"),
    }
}

fn gt_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int > p2_int),
            _ => panic!("You can only test 'greater than' of Int by another Int"),
        },
        _ => panic!("'Greater than' test operator can only be done with Int"),
    }
}

fn lte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int <= p2_int),
            _ => panic!("You can only test 'lower than or equal' of Int by another Int"),
        },
        _ => panic!("'Lower than or equal' test operator can only be done with Int"),
    }
}

fn gte_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Int(p1_int) => match p2 {
            Primitive::Int(p2_int) => Primitive::Bool(p1_int >= p2_int),
            _ => panic!("You can only test 'greater than or equal' of Int by another Int"),
        },
        _ => panic!("'Greater than or equal' test operator can only be done with Int"),
    }
}

fn and_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool && p2_bool),
            _ => panic!("You can only use 'and' operator between Bool"),
        },
        _ => panic!("You can only use 'and' operator between Bool"),
    }
}

fn or_two_primitives(p1: Primitive, p2: Primitive) -> Primitive {
    match p1 {
        Primitive::Bool(p1_bool) => match p2 {
            Primitive::Bool(p2_bool) => Primitive::Bool(p1_bool || p2_bool),
            _ => panic!("You can only use 'or' operator between Bool"),
        },
        _ => panic!("You can only use 'or' operator between Bool"),
    }
}
