use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, Neg};
use std::sync::Arc;
use rust_decimal::Decimal;
use crate::function::function::{DiscriminantId, Function};
use crate::function::function::Function::{Add, Sub, Constant, Mul, Div, S, Variable, Pow};
use crate::function::freeze::freeze;
use std::mem::Discriminant;

//
// /************************************************************
//  ** PSEUDOCODE **
//  *
//  * 3 Pieces to Use:
//  * -> Raw Pointer to Function
//  * -> Buildable Hash for Function (changes as function gets simplified)
//  * -> Function with Simplified Parts
//
//  *
//  * N-key Hashmap to use as a lookup table
//  * * Key1: All original hashes of functions
//  * * Key2: The simplified function hash
//  * * Value: key2, the simplified function, mul, pow
//
//  Simplify Function:
//  * The function can be represented as a binary tree
//    a * x^b
//     -> Mul (a, Pow (x, b))
//     -> Mul (ptr to a, ptr to Pow (ptr to x, ptr to b))
//
//
//
//
//  ************************************************************/
//

#[derive(Clone, Debug)]
pub struct HashValue<'f> { // bad name will change this later
    pub hash: u64,
    pub ptr: *const Function<'f>,
    pub mul: f64,
    pub pow: f64,
    pub mul_ptr: Option<*const Function<'f>>,
    pub pow_ptr: Option<*const Function<'f>>,
    pub func_type: Discriminant<Function<'f>>,
    pub idx: usize,
}

// hashmap is:
// Key: (Parent Function ptr, simplified hash)
// Val: (simplified hash, vec of child hashes & their types, mul, pow, function type, index in array)
fn simplify_hash2<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<(* const Function<'f>, u64), (*const Function<'f>, u64, f64, f64, Vec<(*const Function<'f>, u64)>)>) -> Vec<(*const Function<'f>, u64, u8, f64, f64)> {

    let ptr = Arc::as_ptr(function);

    match &**function {
        Variable(_) => {
            let hash = freeze(function);
            vec![(ptr, hash, function.id(), 1.0, 1.0)]
        }
        Constant(val) => {
            let hash = freeze(function);
            vec![(ptr, hash, function.id(), *val, 1.0)]
        }
        Add { vec } => {
            // creating a primary lookup.
            // TODO: axe this with the ptr node method (this ur idea patrick)
            hmap.insert((ptr, 0), (ptr, 0, 0.0, 0.0, vec![]));

            vec.iter().for_each(|child| {
                // simplifying and iterating through grandchildren
                simplify_hash2(child, hmap).iter().for_each(|(gc_ptr, gc_hash, gc_id, gc_mul, gc_pow)| {
                    // hashing the power into the hash for comparison
                    let gc_add_hash = freeze(&Pow{ base: unsafe { Arc::from_raw(*gc_ptr) }, raised: Constant(*gc_pow).into() });
                    // check if in hashmap by hash
                    if let Some((_, _, mul, _, _)) = hmap.get_mut(&(ptr, gc_add_hash)) {
                        // if it is, add the mul to the current mul
                        *mul += gc_mul;
                    } else {
                        // else insert it as new
                        hmap.insert((ptr, gc_add_hash), (*gc_ptr, *gc_hash, *gc_mul, *gc_pow, vec![]));
                        // also insert it into the primary lookup
                        hmap.get_mut(&(ptr, 0)).unwrap().4.push((*gc_ptr, gc_add_hash));
                    }
                })
            });

            // packaging everything up
            let add_vec = hmap.get(&(ptr, 0)).unwrap().4.iter().map(|child| {
                // getting the child data
                let (gc_ptr, _, gc_mul, gc_pow, _) = hmap.get(&(ptr, child.1)).unwrap();

                match (gc_mul, gc_pow) {
                    (1.0, 1.0) => {
                        unsafe { Arc::from_raw(*gc_ptr) }
                    }
                    (_, 0.0) => {
                        Arc::new(Constant(1.0))
                    }
                    (0.0, _) => {
                        Constant(0.0).into()
                    }
                    (mul, pow) => {
                        if mul == &1.0 {
                            Pow { base: unsafe { Arc::from_raw(*gc_ptr) }, raised: Constant(*pow).into() }.into()
                        } else if pow == &1.0 {
                            Mul { vec: vec![Constant(*mul).into(), unsafe { Arc::from_raw(*gc_ptr) }] }.into()
                        } else {
                           Mul { vec: vec![Constant(*mul).into(), Pow { base: unsafe { Arc::from_raw(*gc_ptr) }, raised: Constant(*pow).into() }.into()] }.into()
                        }
                    }
                }

            }).collect();

            let fnctn = Add { vec: add_vec };

            let hash = freeze(&fnctn);
            let new_ptr = Arc::into_raw(Arc::new(fnctn));

            vec![(new_ptr, hash, function.id(), 1.0, 1.0)]
            // vec![(ptr, 0, 0, 1.0, 1.0)]
        }
        _ => todo!()
    }
}

pub fn stringify_hashmap<'f>(hmap: HashMap<(* const Function<'f>, u64), (*const Function<'f>, u64, f64, f64, Vec<(*const Function<'f>, u64)>)>) {
    let mut res = String::new();
    for (key, (ptr, hash, mul, pow, vec)) in hmap {
        println!("Key: {:?}, Val: {:?}, Hash: {:?}, Mul: {:?}, Pow: {:?} Vec: {:?}", key, unsafe { &(*ptr) }, hash, mul, pow, vec);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable, Add, Mul, S};

    #[test]
    fn test_simplify_hash2() {
        let x = Variable("x");
        let y = Variable("y");
        let z = x.clone() + x.clone();

        let mut hmap = HashMap::new();
        let res = simplify_hash2(&z.into(), &mut hmap);
        println!("{:?}", res);
        stringify_hashmap(hmap);
    }
}