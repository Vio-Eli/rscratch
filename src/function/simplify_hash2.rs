use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Neg};
use std::sync::{Arc, Mutex};
use rust_decimal::Decimal;
use crate::function::function::{DiscriminantId, Function};
use crate::function::function::Function::{Add, Sub, Constant, Mul, Div, S, Variable, Pow};
use crate::function::freeze::freeze;
use std::mem::Discriminant;
use std::rc::{Rc, Weak};
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


#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct HashKeys<'f> {
    pub ptr: *const Function<'f>,
    pub hash: u64
}

#[derive(Clone, Debug)]
pub struct HashVals<'f> {
    pub ptr: *const Function<'f>,
    pub idx: usize,
    pub hash: u64,
    pub mul: f64,
    pub pow: f64,
    pub mul_ptr: Option<*const f64>,
    pub pow_ptr: Option<*const f64>,
    pub children: Vec<(*const Function<'f>, u64)>
}

#[derive(Debug)]
pub struct SimplifyReturn<'f> {
    pub ptr: *const Function<'f>,
    pub hash: u64,
    pub id: u8,
    pub mul: f64,
    pub pow: f64
}

// hashmap is:
// Key: (Parent Function ptr, simplified hash)
// Val: (simplified hash, vec of child hashes & their types, mul, pow, function type, index in array)
fn simplify_hash2<'f, 'm>(function: &Arc<Function<'f>>, hmap: &'m mut HashMap<HashKeys<'f>, HashVals<'f>>, id: u8) -> Vec<SimplifyReturn<'f>> {

    let ptr = Arc::as_ptr(function);

    match &**function {
        Variable(_) => {
            let hash = freeze(function);
            vec![SimplifyReturn { ptr, hash, id: function.id(), mul: 1.0, pow: 1.0 }]
        }
        Constant(val) => {
            let hash = freeze(function);
            vec![SimplifyReturn { ptr, hash, id: function.id(), mul: *val, pow: 1.0 }]
        }
        Add { vec } => {
            // creating a primary lookup.
            // TODO: axe this with the ptr node method (this ur idea patrick)
            hmap.insert(HashKeys { ptr, hash: 0 }, HashVals { ptr, idx: 0, hash: 0, mul: 0.0, pow: 0.0, mul_ptr: None, pow_ptr: None, children: vec![] });

            // Creating a new Vec that'll become the contents of Add
            let mut add_vec = vec![];

            vec.iter().for_each(|child| {
                // simplifying and iterating through grandchildren
                simplify_hash2(child, hmap, id).iter().for_each(|SimplifyReturn { ptr: gc_ptr, hash: gc_hash, id: gc_id, mul: gc_mul, pow: gc_pow }| {
                    // hashing the power into the hash for comparison
                    let gc_add_hash = freeze(&Pow{ base: unsafe { Arc::from_raw(*gc_ptr) }, raised: Constant(*gc_pow).into() });
                    // check if in hashmap by hash
                    if let Some(HashVals {ptr: gc_hmap_ptr, idx: gc_hmap_idx, mul, mul_ptr, .. }) = hmap.get_mut(&HashKeys { ptr, hash: gc_add_hash }) {
                        // if it is, add the mul to the current mul
                        *mul += gc_mul; // LEGACY

                        if let Some(gc_mul_ptr) = mul_ptr {
                            // Editing the Mul value in place
                            // evil triple pointer 
                            unsafe { *(gc_mul_ptr as *mut *const f64 as *mut f64) += gc_mul; } // wot is this cast - harry potter wizard spell
                        } else {
                            // Creating the Mul block and getting the pointer to the val inside
                            let mut mul_const = Constant(*gc_mul);
                            if let Constant(ref mut val) = &mut mul_const {
                                *mul_ptr = Some(val as *const f64); // Setting ptr in hmap
                                *val += 1.0;
                            }

                            // Replacing the current node with the Mul block that contains current node,
                            //  essentially breaking the tree and reforming it.
                            // We use std::ptr::read to avoid recursion
                            // This might be able to be replaced with Weak<T>
                            // unsafe { *(*gc_hmap_ptr as *mut Function<'f>) = Mul { vec: vec![std::ptr::read(*gc_hmap_ptr).into(), mul_const.into()] } };
                            let new_gc = Arc::new(Mul { vec: vec![unsafe { Arc::from_raw(*gc_hmap_ptr) }, Arc::new(mul_const)] });
                            
                            // Inserting the new Mul block into the add_vec
                            add_vec[*gc_hmap_idx] = unsafe { new_gc.clone() };
                            *gc_hmap_ptr = Arc::as_ptr(&new_gc);
                            
                        }
                    } else {
                        // Getting function to push into add_vec and it's ptr
                        // Arc::from_raw requires ownership so old ptr will not work anymore
                        let gc_new_func = unsafe { Arc::from_raw(*gc_ptr) };
                        let gc_new_ptr = Arc::as_ptr(&gc_new_func);
                        add_vec.push(gc_new_func);
                        
                        // If gc_mul and gc_pow are 1.0, then their respective ptrs should be None
                        let (gc_mul_ptr, gc_pow_ptr) = match (gc_mul, gc_pow) {
                            (1.0, 1.0) => (None, None),
                            (_, 1.0) => (Some(gc_mul as *const f64), None),
                            (1.0, _) => (None, Some(gc_pow as *const f64)),
                            (_, _) => (Some(gc_mul as *const f64), Some(gc_pow as *const f64))
                        };
                        // Inserting values into hashmap for later retrieval and comparison
                        hmap.insert(HashKeys { ptr, hash: gc_add_hash }, HashVals { ptr: gc_new_ptr, idx: add_vec.len() - 1, hash: *gc_hash, mul: *gc_mul, pow: *gc_pow, mul_ptr: gc_mul_ptr, pow_ptr: gc_pow_ptr, children: vec![] });
                        // Also insert it into the primary lookup for record keeping
                        hmap.get_mut(&HashKeys { ptr, hash: 0 }).unwrap().children.push((gc_new_ptr, gc_add_hash));
                    }
                })
            });

            // Compiling the Function
            let fnctn = Add { vec: add_vec };
            let hash = freeze(&fnctn);
            let new_ptr = Arc::into_raw(Arc::new(fnctn));

            vec![SimplifyReturn { ptr: new_ptr, hash, id: function.id(), mul: 1.0, pow: 1.0 }]
            // vec![(ptr, 0, 0, 1.0, 1.0)]
        }
        _ => todo!()
    }
}

pub fn stringify_hashmap<'f>(hmap: HashMap<HashKeys, HashVals>) {
    let mut res = String::new();
    for (key, HashVals {ptr, idx, hash, mul, pow, mul_ptr, pow_ptr, children }) in hmap {
        println!("Key: {:?}, Val: {:?}, Val Ptr: {:?}, Idx: {:?}, Hash: {:?}, Mul: {:?}, Pow: {:?}, Mul Ptr: {:?}, Pow Ptr: {:?}, Vec: {:?}, Vec Expds: {:?}", key, unsafe { &(*ptr) }, ptr, idx, hash, mul, pow, mul_ptr, pow_ptr, children, children.iter().map(|(ptr, hash)| unsafe { &(**ptr) }).collect::<Vec<_>>());
    }
}

// TODO: plan for version 3 or 4
// instead of doing the splicing immediately return a function that will execute the splice as a vec
// the signature might look like something as follows
// fn(.. /*options tbd*/) -> Vec<fn(/*location*/ *const Function) -> Option<()> /*success/failure*/>
// the function would be a factory that returns a function that that can be executed to preform the slice
// returning the factory is better than returning the constructed function because less memory would be allocated
// it would be very efficient because it would be very efficient because a fn(..) is just a pointer to a location in asm
// when the recursive part of the algo is done (currently the entire algo is recursive)
// we would determine if any factories can be "canceled out" then we will execute the generated funcs
// there should be an option to either provide an async runtime like tokio if the user is using one
// or provide a thread-pool to possibly execute on
// the thread-pool option would just be us spawning threads manually



// fn a() -> Vec<fn(/*location*/ *const Function) -> Option<()> /*success/failure*/> {
//     // let v: Vec<dyn FnOnce> = vec![];
// } // sry i commented this so i can test stuff


#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable, Add, Mul, S};

    #[test]
    fn test_simplify_hash2() {
        let x = Variable("x");
        let y = Variable("y");
        let z = x.clone() + x.clone();
        let z_id = z.id();

        let mut hmap = HashMap::new();
        let res = simplify_hash2(&z.into(), &mut hmap, z_id);
        println!("{:?}", res);
        println!("{:?}", unsafe { &(*res[0].ptr) });
        stringify_hashmap(hmap);
    }
}