use std::sync::Arc;
use std::ops::Deref;
use crate::function::function::Function;
use crate::function::function::Function::{Add, Constant, Mul, S, Variable};

pub(crate) fn simplify(function: Function) -> Function {
    // let mut counts = HashMap::new();
    simplify_helper(function) // , counts: &'m mut HashMap<&'m str, f64>
}

fn simplify_helper(function: Function) -> Function {
    match function {
        Variable(name) => {
            S {
                f: Arc::new(Variable(name)),
                m: 1.0,
                p: 1.0
            }
        }
        Constant(_) | S {..} => function,
        Add { vec } => {
            let mut new_vec = vec![];
            for f in vec {
                let simplified = simplify_helper(f.deref().clone());
                match simplified {
                    Constant(val) => {
                        let mut found = false;
                        for f in new_vec.iter_mut() {
                            match f {
                                Constant(other_val) => {
                                    *other_val += val;
                                    found = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        if !found {
                            new_vec.push(Constant(val));
                        }
                    }
                    S { f: function, m: mul, p: pow } => {
                        let mut found = false;
                        for f in new_vec.iter_mut() {
                            match f {
                                S { f: other_function, m: other_mul, p: other_pow } => {
                                    if *other_function.deref().clone() == *function.deref() && other_pow.clone() == pow.clone() {
                                        *other_mul += mul;
                                        found = true;
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !found {
                            new_vec.push(S {
                                f: function,
                                m: mul,
                                p: pow
                            });
                        }
                    }
                    _ => {
                        println!("Not Implemented in Add: {:?}", simplified);
                        todo!()
                    }
                }
            }
            S {
                f: Arc::new(Add { vec: new_vec.into_iter().map(|f| f.into()).collect() }),
                m: 1.0,
                p: 1.0
            }
            // Add { vec: new_vec.into_iter().map(|f| f.into()).collect() }
        }
        Mul { vec } => {
            let mut new_vec = vec![];
            for f in vec {
                let simplified = simplify_helper(f.deref().clone());
                match simplified {
                    Constant(val) => {
                        let mut found = false;
                        for f in new_vec.iter_mut() {
                            match f {
                                Constant(other_val) => {
                                    *other_val *= val;
                                    found = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        if !found {
                            new_vec.push(Constant(val));
                        }
                    }
                    S { f: function, m: mul, p: pow } => {
                        let mut found = false;
                        for f in new_vec.iter_mut() {
                            match f {
                                S { f: other_function, m: other_mul, p: other_pow } => {
                                    if *other_function.deref().clone() == *function.deref() {
                                        *other_mul *= mul;
                                        *other_pow += pow;
                                        found = true;
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !found {
                            new_vec.push(S {
                                f: function,
                                m: mul,
                                p: pow
                            });
                        }
                    }
                    _ => {
                        println!("Not Implemented in Mul: {:?}", simplified);
                        todo!()
                    }
                }
            }
            S {
                f: Arc::new(Mul { vec: new_vec.into_iter().map(|f| f.into()).collect() }),
                m: 1.0,
                p: 1.0
            }
            // Mul { vec: new_vec.into_iter().map(|f| f.into()).collect() }
        }
        _ => {
            println!("Not Implemented in Root: {:?}", function);
            todo!()
        }
    }
}