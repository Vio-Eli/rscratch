use std::collections::HashMap;
use std::ops::{Deref, Neg};
use std::sync::Arc;
use rust_decimal::Decimal;
use crate::function::function::Function;
use crate::function::function::Function::{Add, Sub, Constant, Mul, Div, S, Variable};
use crate::function::freeze::freeze;
use rust_decimal_macros::dec;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum FunctionType {
    Parent,
    Constant,
    SAdd(u64, Decimal), // when we simplify children in ADD, we care about the power
    SMul(u64), // when we simplify children in MUL, we don't care about the power
    Add,
    Mul,
    Other
}

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

fn simplify_hash2<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<(*const Function<'f>, FunctionType), Vec<*const Function<'f>>>) -> *const Function<'f> {
    // get a raw pointer to the parent function
    let parent_ptr = Arc::into_raw(function.clone());
    match &**function {
        Variable(name) => {
            // convert
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
            // type vec collection of all the existing function types existing in the parent
            // to keep track of the types we have encountered
            let mut type_vec = vec![];
            // visit each child node
            vec.iter().for_each(|child| {
                // simplify the function then return a raw pointer to it
                let child_ptr = simplify_hash2(child, hmap);
                // own the child
                let child_func = unsafe { &*child_ptr };
                // categorize the sub funciton type
                let child_func_type = match &child_func {
                    Constant(_) => FunctionType::Constant,
                    S { f, m, p } => {
                        FunctionType::SAdd(freeze(f), Decimal::from_f64_retain(*p).unwrap())
                    }
                    _ => FunctionType::Other
                };
                // if this exists in the "soup" hashmap
                if let Some(child_vec) = hmap.get_mut(&(parent_ptr, child_func_type.clone())) {
                    // grab the existing pointer
                    // look up the existing function and prep to mutate it in place
                    // the mutation will be based on the current child function and the existing function
                    let mut existing_child_ptr = child_vec[0]; // only one item in this vector. something went wrong if not true
                    let existing_child_func = unsafe { &*existing_child_ptr };
                    match existing_child_func {
                        Constant(val) => {
                            match child_func {
                                // if its constant then just add both
                                Constant(other_val) => {
                                    let new_func = Constant(val + other_val);
                                    child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                }
                                _ => unreachable!()
                            }
                        }
                        S { f: existing_f, m: existing_m, p: existing_p } => {
                            match child_func {
                                S { f: other_f, m: other_m, p: other_p } => {
                                    // because the types are the same we know the power and functions are the same
                                    // if existing_f == other_f && existing_p == other_p {
                                    if existing_m == &-other_m.clone() {
                                        let new_func = Constant(0.0);
                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                    } else {
                                        let new_func = S {
                                            f: other_f.clone(),
                                            m: existing_m + other_m,
                                            p: *other_p
                                        };
                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                    }
                                }
                                _ => unreachable!()
                            }
                        }
                        _ => unreachable!()
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
                        match &**f {
                            Add {..} => FunctionType::SAdd(freeze(&Add { vec: vec![] }), Decimal::from_f64_retain(*p).unwrap()),
                            Sub {..} => FunctionType::SAdd(freeze(&Add { vec: vec![] }), Decimal::from_f64_retain(*p).unwrap()),
                            _ => {
                                FunctionType::SAdd(freeze(f), Decimal::from_f64_retain(*p).unwrap())
                            }
                        }
                    }
                    _ => FunctionType::Other
                }
            }

            let lhs_type = get_func_type(lhs_func);
            let rhs_type = get_func_type(rhs_func);
            println!("Sub Types: {:?}, {:?}", lhs_type, rhs_type);

            if lhs_type == rhs_type {
                match (&lhs_func, &rhs_func) {
                    (Constant(lhs_val), Constant(rhs_val)) => {
                        Arc::into_raw(Arc::new(Constant(lhs_val - rhs_val)))
                    }
                    (S { f: lhs_s_func, m: lhs_s_mul, p: lhs_s_pow }, S { f: rhs_s_func, m: rhs_s_mul, p: rhs_s_pow }) => {
                        match (&**lhs_s_func, &**rhs_s_func) {
                            (Add { vec: lhs_vec }, Add { vec: rhs_vec }) if lhs_s_func != rhs_s_func => {
                                let mut type_vec = vec![];
                                lhs_vec.iter().for_each(|child| {
                                    let child_ptr = Arc::into_raw(child.clone());
                                    let child_type = get_func_type(&*child);
                                    type_vec.push(child_type.clone());
                                    hmap.insert((parent_ptr, child_type), vec![child_ptr]);
                                });

                                rhs_vec.iter().for_each(|child| {
                                    let child_type = get_func_type(&*child);
                                    if let Some(child_vec) = hmap.get_mut(&(parent_ptr, child_type.clone())) {
                                        let mut existing_child_ptr = child_vec[0];
                                        let existing_child_func = unsafe { &*existing_child_ptr };
                                        match existing_child_func {
                                            Constant(val) => {
                                                match &**child {
                                                    Constant(other_val) => {
                                                        let new_func = Constant(val - other_val);
                                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                    }
                                                    _ => unreachable!()
                                                }
                                            }
                                            S { f: existing_f, m: existing_m, p: existing_p } => {
                                                match &**child {
                                                    S { f: other_f, m: other_m, p: other_p } => {
                                                        if existing_m == other_m {
                                                            let new_func = Constant(0.0);
                                                            child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                        } else {
                                                            let new_func = S {
                                                                f: other_f.clone(),
                                                                m: existing_m - other_m,
                                                                p: *other_p
                                                            };
                                                            child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                        }
                                                    }
                                                    _ => unreachable!()
                                                }
                                            }
                                            _ => unreachable!()
                                        }
                                    } else {
                                        let child_ptr = Arc::into_raw(child.clone());
                                        type_vec.push(child_type.clone());
                                        hmap.insert((parent_ptr, child_type), vec![child_ptr]);
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
                            (Add { vec: lhs_add_vec }, Sub { lhs: rhs_sub_lhs, rhs: rhs_sub_rhs}) => {
                                println!("lhs_add_vec: {:?}, rhs_sub_lhs: {:?}, rhs_sub_rhs: {:?}", lhs_add_vec, rhs_sub_lhs, rhs_sub_rhs);
                                let mut type_vec = vec![];
                                lhs_add_vec.iter().for_each(|child| {
                                    let child_ptr = Arc::into_raw(child.clone());
                                    let child_type = get_func_type(&*child);
                                    type_vec.push(child_type.clone());
                                    hmap.insert((parent_ptr, child_type), vec![child_ptr]);
                                });
                                for (sub_itm, sign_mult) in vec![(rhs_sub_lhs, 1.0), (rhs_sub_rhs, -1.0)] {
                                    let sub_itm_type = get_func_type(&*sub_itm);
                                    if let Some(child_vec) = hmap.get_mut(&(parent_ptr, sub_itm_type.clone())) {
                                        let mut existing_child_ptr = child_vec[0];
                                        let existing_child_func = unsafe { &*existing_child_ptr };
                                        match &existing_child_func {
                                            Constant(val) => {
                                                match &**sub_itm {
                                                    Constant(other_val) => {
                                                        let new_func = Constant(val - (other_val * sign_mult));
                                                        child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                    }
                                                    _ => unreachable!()
                                                }
                                            }
                                            S { f: existing_f, m: existing_m, p: existing_p } => {
                                                match &**sub_itm {
                                                    S { f: other_f, m: other_m, p: other_p } => {
                                                        if existing_m == other_m {
                                                            let new_func = Constant(0.0);
                                                            child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                        } else {
                                                            let new_func = S {
                                                                f: other_f.clone(),
                                                                m: existing_m - (other_m * sign_mult),
                                                                p: *other_p
                                                            };
                                                            child_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                        }
                                                    }
                                                    _ => unreachable!()
                                                }
                                            }
                                            _ => unreachable!()
                                        }
                                    } else {
                                        let mut child_ptr = Arc::into_raw(sub_itm.clone());
                                        let child_func = unsafe { &*child_ptr };
                                        match &child_func {
                                            Constant(val) => {
                                                let new_child_func = Constant(val * sign_mult);
                                                child_ptr = Arc::into_raw(Arc::new(new_child_func));
                                            }
                                            S { f, m, p } => {
                                                let new_child_func = S {
                                                    f: f.clone(),
                                                    m: m * -sign_mult,
                                                    p: *p
                                                };
                                                child_ptr = Arc::into_raw(Arc::new(new_child_func));
                                            }
                                            _ => unreachable!()
                                        };
                                        type_vec.push(sub_itm_type.clone());
                                        hmap.insert((parent_ptr, sub_itm_type), vec![child_ptr]);
                                    }
                                }
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
                            _ => {

                                if lhs_s_mul == rhs_s_mul {
                                    Arc::into_raw(Arc::new(
                                        Constant(0.0)
                                    ))
                                } else {
                                    println!("SUB OTHER -> lhs: {:?}, rhs: {:?}", lhs_s_func, rhs_s_func);
                                    Arc::into_raw(Arc::new(
                                        S {
                                            f: lhs_s_func.clone(),
                                            m: lhs_s_mul - rhs_s_mul,
                                            p: *lhs_s_pow
                                        }
                                    ))
                                }
                            }
                        }
                    }
                    _ => unreachable!()
                }
            } else {
                println!("SUB NONEQUAL TYPES lhs: {:?}, rhs: {:?}", lhs_func, rhs_func);
                Arc::into_raw(Arc::new( S {
                    f: Arc::new(Sub {
                        // lhs: Arc::new(S {
                        //     f: Arc::new(lhs_func.clone()),
                        //     m: 1.0,
                        //     p: 1.0
                        // }),
                        // rhs: Arc::new(S {
                        //     f: Arc::new(rhs_func.clone()),
                        //     m: 1.0,
                        //     p: 1.0
                        // })
                        lhs: Arc::new(lhs_func.clone()),
                        rhs: Arc::new(rhs_func.clone())
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
                        FunctionType::SMul(freeze(f))
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
        Div {
            num,
            den
        } => {
            let num_ptr = simplify_hash2(num, hmap);
            let den_ptr = simplify_hash2(den, hmap);
            let num_func = unsafe { &*num_ptr };
            let den_func = unsafe { &*den_ptr };

            let num_type = match num_func {
                Constant(_) => FunctionType::Constant,
                S { f, m, p } => {
                    FunctionType::SAdd(freeze(f), Decimal::from_f64_retain(*p).unwrap())
                }
                _ => FunctionType::Other
            };
            let den_type = match den_func {
                Constant(_) => FunctionType::Constant,
                S { f, m, p } => {
                    FunctionType::SAdd(freeze(f), Decimal::from_f64_retain(*p).unwrap())
                }
                _ => FunctionType::Other
            };

            if num_type == den_type {
                match (num_func, den_func) {
                    (Constant(num_val), Constant(den_val)) => {
                        Arc::into_raw(Arc::new(Constant(num_val / den_val)))
                    }
                    (S { f: num_s_func, m: num_s_mul, p: num_s_pow }, S { f: den_s_func, m: den_s_mul, p: den_s_pow }) => {
                        Arc::into_raw(Arc::new(
                            S {
                                f: num_s_func.clone(),
                                m: num_s_mul / den_s_mul,
                                p: num_s_pow - den_s_pow
                            }
                        ))
                    }
                    _ => todo!()
                }
            } else {
                Arc::into_raw(Arc::new( S {
                    f: Arc::new(Div {
                        num: Arc::new(S {
                            f: Arc::new(num_func.clone()),
                            m: 1.0,
                            p: 1.0
                        }),
                        den: Arc::new(S {
                            f: Arc::new(den_func.clone()),
                            m: 1.0,
                            p: 1.0
                        })
                    }),
                    m: 1.0,
                    p: 1.0
                }))
            }
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
        let z = Variable("z");
        let a = Variable("a");

        let f = (z.clone() + a.clone()) - (x.clone() - y.clone());
        let f_simple = simplify_h(f.clone());
        println!("Input: {:?}", f);
        println!("Simpl: {:?}", f_simple);
    }
}