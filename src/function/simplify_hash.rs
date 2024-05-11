use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use crate::function::function::Function;
use crate::function::function::Function::{Add, Constant, Mul, S, Variable};

pub fn simplify_h(function: Function) -> Function {
    let mut hmap = HashMap::new();
    // simplify_hash(&Arc::new(function), &mut hmap); // , counts: &'m mut HashMap<&'m str, f64>
    simplify_hash2(&Arc::new(function), &mut hmap);

    println!("{:?}", hmap);
    // println!("{}", stringify_hash(&hmap));
    println!("{}", stringify_hash2(&hmap));
    Constant(0.0)
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

fn simplify_hash2<'f: 's, 'm, 's>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<(*const Function<'f>, &'s str), Vec<*const Function<'f>>>) -> *const Function<'f> {
    let origin_ptr = Arc::into_raw(function.clone());
    if !hmap.contains_key(&(origin_ptr, "origin")) {
        match &**function {
            Variable(name) => {
                Arc::into_raw(Arc::new(S {
                    f: Arc::new(Variable(name)),
                    m: 1.0,
                    p: 1.0
                }))
            }
            Constant(_) | S {..} => Arc::into_raw(Arc::new(function.deref().clone())),
            Add { vec } => {
                vec.iter().for_each(|child| {
                    // simplify the child
                    let child_ptr = simplify_hash2(child, hmap);
                    let child_func = unsafe { &*child_ptr };

                    match child_func {
                        Constant(val) => {
                            // check if key (origin_ptr, other) where other is a constant exists
                            if let Some(cur_vec) = hmap.get_mut(&(origin_ptr, "C")) {
                                let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
                                let mut cur_func = unsafe { &*cur_ptr };
                                match cur_func {
                                    Constant(other_val) => {
                                        // update the value
                                        let new_func = Constant(val + other_val);
                                        cur_vec[0] = Arc::into_raw(Arc::new(new_func));
                                    }
                                    _ => todo!()
                                };
                            } else {
                                // create a new key (origin_ptr, other) where other is a constant
                                hmap.insert((origin_ptr, "C"), vec![child_ptr]);
                            }
                        }
                        S { f, m: mul, p: pow } => {
                            match f.deref().clone() {
                                Variable(name) => {
                                    if let Some(cur_vec) =  hmap.get_mut(&(origin_ptr, name)) {
                                        let mut cur_ptr = cur_vec[0]; // THERE SHOULD ONLY BE 1 ELEMENT IN THIS VECTOR
                                        let mut cur_func = unsafe { &*cur_ptr };
                                        match cur_func {
                                            S { f: other_f, m: other_mul, p: other_pow } => {
                                                if other_f == f && other_pow == pow {
                                                    let new_func = S {
                                                        f: f.clone(),
                                                        m: mul + other_mul,
                                                        p: *pow
                                                    };
                                                    cur_vec[0] = Arc::into_raw(Arc::new(new_func));
                                                }
                                            }
                                            _ => todo!()
                                        }
                                    } else {
                                        hmap.insert((origin_ptr, name), vec![child_ptr]);

                                    }
                                }
                                _ => todo!()
                            }
                        }
                        _ => todo!()
                    }
                });

                origin_ptr
            }
            _ => todo!()
        }
    } else {
        origin_ptr
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

pub fn stringify_hash2(map: &HashMap<(*const Function, &str), Vec<*const Function>>) -> String {
    let mut s = String::new();
    for ((key, name), value) in map {
        s.push_str(&format!("{:?}, {:?} ({:?}) -> [", key, name, unsafe { &(**key) }));
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
        // let z = (x.clone() + y.clone()) * x.clone();
        let z = Constant(2.0) + x.clone() + Constant(2.0) + x.clone();
        let z_simple = simplify_h(z.clone());
        println!("{:?}", z);
        println!("{:?}", z_simple);
    }
}