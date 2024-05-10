use std::collections::HashMap;
use std::sync::Arc;
use crate::function::function::Function;
use crate::function::function::Function::{Add, Constant, Mul, S, Variable};

pub fn simplify_h(function: Function) -> Function {
    let mut hmap = HashMap::new();
    simplify_hash(&Arc::new(function), &mut hmap); // , counts: &'m mut HashMap<&'m str, f64>

    println!("{:?}", hmap);
    println!("{}", stringify_hash(&hmap));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable};

    #[test]
    fn test_simplify_hash() {
        let x = Variable("x");
        let y = Variable("y");
        let z = (x.clone() + y.clone()) * x.clone();
        let z_simple = simplify_h(z.clone());
        println!("{:?}", z);
        println!("{:?}", z_simple);
    }
}