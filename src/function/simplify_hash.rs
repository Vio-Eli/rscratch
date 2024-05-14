use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use rust_decimal::Decimal;
use crate::function::function::Function;
use crate::function::function::Function::{Add, Sub, Constant, Mul, Div, S, Variable};
use rust_decimal_macros::dec;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum FunctionType {
    Parent,
    Constant,
    SAdd(String, Decimal),
    SMul(String),
    Add,
    Mul,
    Other
}

// fn functiontotype<'t>(function: &Function<'t>) -> FunctionType<'t> {
//     match function {
//         Constant(_) => FunctionType::Constant,
//         Add {..} => FunctionType::Add,
//         Mul {..} => FunctionType::Mul,
//         _ => FunctionType::Other
//     }
// }

// #[derive(Hash, Eq, PartialEq, Debug)]
// struct FunctionKey<'t> {
//     function_type: FunctionType<'t>,
// }

pub fn simplify_h(function: Function) -> Function {
    let mut hmap = HashMap::new();
    let fun = simplify_hash2(&Arc::new(function), &mut hmap); // , counts: &'m mut HashMap<&'m str, f64>
    // simplify_hash2(&Arc::new(function), &mut hmap);

    println!("{:?}", hmap);
    // println!("{}", stringify_hash(&hmap));
    println!("{}", stringify_hash2(&hmap));
    unsafe { (*fun).clone() }
    // Constant(0.0)
}

fn simplify_hash<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<*const Function<'f>, Vec<*const Function<'f>>>) {
    let ptr = Arc::into_raw(function.clone());
    if !hmap.contains_key(&ptr) {
        let children = match &**function {
            Variable(_) | Constant(_) => vec![],
            Add { vec } | Mul { vec } => {
                vec.iter().map(|child| Arc::into_raw(child.clone())).collect()
            },
            _ => {
                println!("Not implemented: {:?}", function);
                vec![]
            }
        };
        hmap.insert(ptr, children);
        match &**function {
            Add { vec } | Mul { vec } => {
                for child in vec {
                    simplify_hash(child, hmap);
                }
            },
            _ => {}
        };
    }
}

// C(2) + C(1) -> (Add, C(2)) ->
// S(x, 95, 2) + S(x, 2, 2) -> S(x, 97, 2)
// matches!(func, S("x", _, 1))

// fn simplify_hash2<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<(*const Function<'f>, FunctionType<'f>), Vec<*const Function<'f>>>) -> *const Function<'f> {
//     let origin_ptr = Arc::into_raw(function.clone());
//     let origin_type = functiontotype(&**function);
//     if !hmap.contains_key(&(origin_ptr, origin_type)) {
//         match &**function {
//             Variable(name) => {
//                 Arc::into_raw(Arc::new(S {
//                     f: Arc::new(Variable(name)),
//                     m: 1.0,
//                     p: 1.0
//                 }))
//             }
//             Constant(_) | S {..} => Arc::into_raw(Arc::new(function.deref().clone())),
//             Add { vec } => {
//                 vec.iter().for_each(|child| {
//                     // simplify the child
//                     let child_ptr = simplify_hash2(child, hmap);
//                     let child_func = unsafe { &*child_ptr };
//
//                     match &child_func {
//                         Constant(val) => {
//                             // check if key (origin_ptr, other) where other is a constant exists
//                             if let Some(cur_vec) = hmap.get_mut(&(origin_ptr, FunctionType::Constant)) {
//                                 let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
//                                 let mut cur_func = unsafe { &*cur_ptr };
//                                 match cur_func {
//                                     Constant(other_val) => {
//                                         // update the value
//                                         let new_func = Constant(val + other_val);
//                                         cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                     }
//                                     _ => todo!()
//                                 };
//                             } else {
//                                 // create a new key (origin_ptr, other) where other is a constant
//                                 hmap.insert((origin_ptr, FunctionType::Constant), vec![child_ptr]);
//                             }
//                         }
//                         S { f, m: mul, p: pow } => {
//                             match f.deref().clone() {
//                                 Variable(name) => {
//                                     if let Some(cur_vec) =  hmap.get_mut(&(origin_ptr, FunctionType::SAdd(name, Decimal::from_f64_retain(*pow).unwrap()))) {
//                                         let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
//                                         let mut cur_func = unsafe { &*cur_ptr };
//                                         match cur_func {
//                                             S { f: other_f, m: other_mul, p: other_pow } => {
//                                                 if other_f == f && other_pow == pow {
//                                                     let new_func = S {
//                                                         f: f.clone(),
//                                                         m: mul + other_mul,
//                                                         p: *pow
//                                                     };
//                                                     cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                                 }
//                                             }
//                                             _ => todo!()
//                                         }
//                                     } else {
//                                         hmap.insert((origin_ptr, FunctionType::SAdd(name, Decimal::from_f64_retain(*pow).unwrap())), vec![child_ptr]);
//                                     }
//                                 }
//                                 Mul { vec: mul_vec } => {
//                                     if let Some(cur_vec) = hmap.get_mut(&(origin_ptr, FunctionType::Mul)) {
//                                         let mut cur_ptr = cur_vec[0];
//                                         let mut cur_func = unsafe { &*cur_ptr };
//                                         match cur_func {
//                                             S { f: other_f, m: other_mul, p: other_pow } => {
//                                                 if other_f == f && other_pow == pow {
//                                                     let new_func = S {
//                                                         f: f.clone(),
//                                                         m: mul + other_mul,
//                                                         p: *pow
//                                                     };
//                                                     cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                                 }
//                                             }
//                                             _ => todo!()
//                                         }
//                                     } else {
//                                         hmap.insert((origin_ptr, FunctionType::Mul), vec![child_ptr]);
//                                     }
//                                 }
//                                 _ => todo!()
//                             }
//                         }
//                         _ => todo!()
//                     }
//                 });
//
//                 origin_ptr
//             }
//             Mul { vec } => {
//                 vec.iter().for_each(|child| {
//                     let child_ptr = simplify_hash2(child, hmap);
//                     let child_func = unsafe { &*child_ptr };
//
//                     match &child_func {
//                         Constant(val) => {
//                             // check if key (origin_ptr, other) where other is a constant exists
//                             if let Some(cur_vec) = hmap.get_mut(&(origin_ptr, FunctionType::Constant)) {
//                                 let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
//                                 let mut cur_func = unsafe { &*cur_ptr };
//                                 match cur_func {
//                                     Constant(other_val) => {
//                                         // update the value
//                                         let new_func = Constant(val * other_val);
//                                         cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                     }
//                                     _ => todo!()
//                                 };
//                             } else {
//                                 // create a new key (origin_ptr, other) where other is a constant
//                                 hmap.insert((origin_ptr, FunctionType::Constant), vec![child_ptr]);
//                             }
//                         }
//                         S { f, m: mul, p: pow } => {
//                             match f.deref().clone() {
//                                 Variable(name) => {
//                                     if let Some(cur_vec) =  hmap.get_mut(&(origin_ptr, FunctionType::SAdd(name, Decimal::from_f64_retain(*pow).unwrap()))) {
//                                         let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
//                                         let mut cur_func = unsafe { &*cur_ptr };
//                                         match cur_func {
//                                             S { f: other_f, m: other_mul, p: other_pow } => {
//                                                 if other_f == f {
//                                                     let new_func = S {
//                                                         f: f.clone(),
//                                                         m: mul * other_mul,
//                                                         p: pow + other_pow
//                                                     };
//                                                     cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                                 }
//                                             }
//                                             _ => todo!()
//                                         }
//                                     } else {
//                                         hmap.insert((origin_ptr, FunctionType::SAdd(name, Decimal::from_f64_retain(*pow).unwrap())), vec![child_ptr]);
//                                     }
//                                 }
//                                 Add { vec: mul_vec } => {
//                                     if let Some(cur_vec) = hmap.get_mut(&(origin_ptr, FunctionType::Add)) {
//                                         let mut cur_ptr = cur_vec[0];
//                                         let mut cur_func = unsafe { &*cur_ptr };
//                                         match cur_func {
//                                             S { f: other_f, m: other_mul, p: other_pow } => {
//                                                 if other_f == f && other_pow == pow {
//                                                     let new_func = S {
//                                                         f: f.clone(),
//                                                         m: mul + other_mul,
//                                                         p: pow + other_pow
//                                                     };
//                                                     cur_vec[0] = Arc::into_raw(Arc::new(new_func));
//                                                 }
//                                             }
//                                             _ => todo!()
//                                         }
//                                     } else {
//                                         hmap.insert((origin_ptr, FunctionType::Add), vec![child_ptr]);
//                                     }
//                                 }
//                                 _ => todo!()
//                             }
//                         }
//                         _ => todo!()
//                     }
//
//                 });
//
//                 origin_ptr
//             }
//             _ => todo!()
//         }
//     } else {
//         origin_ptr
//     }
// }
fn simplify_hash2<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<(*const Function<'f>, FunctionType), Vec<*const Function<'f>>>) -> *const Function<'f> {
    let parent_ptr = Arc::into_raw(function.clone());
    match &**function {
        Variable(name) => {
            Arc::into_raw(Arc::new(
                S {
                    f: Arc::new(Variable(name)),
                    m: 1.0,
                    p: 1.0
                }
            ))
        }
        Constant(_) | S {..} => parent_ptr,
        Add { vec } => {
            let mut type_vec = vec![];
            vec.iter().for_each(|child| {
                let child_ptr = simplify_hash2(child, hmap);
                let child_func = unsafe { &*child_ptr };
                let child_func_type = match &child_func {
                    Constant(_) => FunctionType::Constant,
                    S { f, m, p } => {
                        let child_str = format!("{:?}", f);
                        FunctionType::SAdd(child_str, Decimal::from_f64_retain(*p).unwrap())
                    }
                    _ => FunctionType::Other
                };
                if let Some(child_vec) = hmap.get_mut(&(parent_ptr, child_func_type.clone())) {
                    let mut existing_child_ptr = child_vec[0];
                    let existing_child_func = unsafe { &*existing_child_ptr };
                    match existing_child_func {
                        Constant(val) => {
                            match child_func {
                                Constant(other_val) => {
                                    let new_func = Constant(val + other_val);
                                    child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                }
                                _ => todo!()
                            }
                        }
                        S { f: existing_f, m: existing_m, p: existing_p } => {
                            match child_func {
                                S { f: other_f, m: other_m, p: other_p } => {
                                    if existing_f == other_f && existing_p == other_p {
                                        let new_func = S {
                                            f: other_f.clone(),
                                            m: existing_m + other_m,
                                            p: *other_p
                                        };
                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                    }
                                }
                                _ => todo!()
                            }
                        }
                        _ => todo!()
                    }
                } else {
                    type_vec.push(child_func_type.clone());
                    hmap.insert((parent_ptr, child_func_type), vec![child_ptr]);
                }
            });

            let new_vec = type_vec.iter().map(|t| {
                let simplified_ptr = hmap.get(&(parent_ptr, t.clone())).unwrap()[0];
                let func = unsafe { &*simplified_ptr };
                Arc::new(func.clone())
            }).collect();

            Arc::into_raw(Arc::new(
                S {
                    f: Arc::new(Add { vec: new_vec }),
                    m: 1.0,
                    p: 1.0
                }
            ))
        }
        Sub { lhs, rhs } => {
            let lhs_ptr = simplify_hash2(lhs, hmap);
            let rhs_ptr = simplify_hash2(rhs, hmap);
            let lhs_func = unsafe { &*lhs_ptr };
            let rhs_func = unsafe { &*rhs_ptr };
            fn get_func_type(func: &Function) -> FunctionType {
                match func {
                    Constant(_) => FunctionType::Constant,
                    S { f, m, p } => {
                        let child_str = format!("{:?}", f);
                        FunctionType::SAdd(child_str, Decimal::from_f64_retain(*p).unwrap())
                    }
                    _ => FunctionType::Other
                }
            }

            let lhs_type = get_func_type(lhs_func);
            let rhs_type = get_func_type(rhs_func);

            if lhs_type == rhs_type {
                match (lhs_func, rhs_func) {
                    (Constant(lhs_val), Constant(rhs_val)) => {
                        Arc::into_raw(Arc::new(Constant(lhs_val - rhs_val)))
                    }
                    (S { f: lhs_s_func, m: lhs_s_mul, p: lhs_s_pow }, S { f: rhs_s_func, m: rhs_s_mul, p: rhs_s_pow }) => {
                        Arc::into_raw(Arc::new(
                            S {
                                f: lhs_s_func.clone(),
                                m: lhs_s_mul - rhs_s_mul,
                                p: *lhs_s_pow
                            }
                        ))
                    }
                    _ => todo!()
                }
            } else {
                println!("lhs: {:?}, rhs: {:?}", lhs_func, rhs_func);
                Arc::into_raw(Arc::new( S {
                    f: Arc::new(Sub {
                        lhs: Arc::new(S {
                            f: Arc::new(lhs_func.clone()),
                            m: 1.0,
                            p: 1.0
                        }),
                        rhs: Arc::new(S {
                            f: Arc::new(rhs_func.clone()),
                            m: 1.0,
                            p: 1.0
                        })
                    }),
                    m: 1.0,
                    p: 1.0
                }))
            }
        }
        Mul { vec } => {
            let mut type_vec = vec![];
            vec.iter().for_each(|child| {
                let child_ptr = simplify_hash2(child, hmap);
                let child_func = unsafe { &*child_ptr };
                let child_func_type = match &child_func {
                    Constant(_) => FunctionType::Constant,
                    S { f, m, p } => {
                        let child_str = format!("{:?}", f);
                        FunctionType::SMul(child_str)
                    }
                    _ => FunctionType::Other
                };
                if let Some(child_vec) = hmap.get_mut(&(parent_ptr, child_func_type.clone())) {
                    let mut existing_child_ptr = child_vec[0];
                    let existing_child_func = unsafe { &*existing_child_ptr };
                    match existing_child_func {
                        Constant(val) => {
                            match child_func {
                                Constant(other_val) => {
                                    let new_func = Constant(val * other_val);
                                    child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                }
                                _ => todo!()
                            }
                        }
                        S { f: existing_f, m: existing_m, p: existing_p } => {
                            match child_func {
                                S { f: other_f, m: other_m, p: other_p } => {
                                    if existing_f == other_f && existing_p == other_p {
                                        let new_func = S {
                                            f: other_f.clone(),
                                            m: existing_m * other_m,
                                            p: existing_p + other_p
                                        };
                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                    }
                                }
                                _ => todo!()
                            }
                        }
                        _ => todo!()
                    }
                } else {
                    type_vec.push(child_func_type.clone());
                    hmap.insert((parent_ptr, child_func_type), vec![child_ptr]);
                }
            });

            let new_vec = type_vec.iter().map(|t| {
                let simplified_ptr = hmap.get(&(parent_ptr, t.clone())).unwrap()[0];
                let func = unsafe { &*simplified_ptr };
                Arc::new(func.clone())
            }).collect();

            Arc::into_raw(Arc::new(
                S {
                    f: Arc::new(Mul { vec: new_vec }),
                    m: 1.0,
                    p: 1.0
                }
            ))
        }
        _ => todo!()
    }

}



pub fn stringify_hash(map: &HashMap<*const Function, Vec<*const Function>>) -> String {
    let mut s = String::new();
    for (key, value) in map {
        s.push_str(&format!("{:?} ({:?}) -> [", key, unsafe { &(**key) }));
        for v in value {
            s.push_str(&format!("{:?}, ", v));
        }
        if value.is_empty() {
            let key_str = unsafe { &(**key) };
            s.push_str(&format!("{:?}", key_str));
        }
        s.push_str("]\n");
    }
    s
}

pub fn stringify_hash2(map: &HashMap<(*const Function, FunctionType), Vec<*const Function>>) -> String {
    let mut s = String::new();
    for ((key, name), value) in map {
        s.push_str(&format!("{:?} ({:?}), {:?} -> [", key, unsafe { &(**key) }, name));
        for v in value {
            s.push_str(&format!("{:?}: {:?}, ", v, unsafe { &(**v) }));
        }
        s.push_str("]\n");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable};

    #[test]
    fn test_simplify_hash() {
        let x = Variable("x");
        let y = Variable("y");
        // let z = (x.clone() - y.clone()) * x.clone();
        // let z = x.clone() * y.clone() + x.clone() * y.clone();
        let z = (x.clone() - y.clone()) * (x.clone() + y.clone() + x.clone()) * (x.clone() - y.clone());
        let z_simple = simplify_h(z.clone());
        println!("Input: {:?}", z);
        println!("Simpl: {:?}", z_simple);
    }
}