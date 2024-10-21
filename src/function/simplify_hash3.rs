use std::collections::HashMap;
use std::ptr::hash;
use std::sync::Arc;
use crate::function::freeze::freeze;
use crate::function::function::{DiscriminantId, Function};
use crate::function::function::Function::{Variable, Constant, S, Add, Sub, Mul, Div, Pow};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct HashKeys {
    // pub ptr: &'f Arc<Function<'f>>,
    pub uuid: u64,
    pub hash: u64
}

#[derive(Clone, Debug)]
pub struct HashVals<'f> {
    pub ptr: Arc<Function<'f>>,
    pub idx: usize,
    pub hash: u64,
    pub mul_ptr: Option<Arc<Function<'f>>>,
    pub pow_ptr: Option<Arc<Function<'f>>>,
    // pub mul_ptr: Option<*const f64>,
    // pub pow_ptr: Option<*const f64>,
}

#[derive(Debug)]
pub struct SimplifyReturn<'f> {
    pub ptr: Arc<Function<'f>>,
    pub hash: u64,
    pub id: u8,
    pub mul: f64,
    pub pow: f64
}

fn simplify_hash3<'f, 'm>(function: &'f Arc<Function<'f>>, hmap: &'m mut std::collections::HashMap<HashKeys, HashVals<'f>>, id: u8) -> Vec<SimplifyReturn<'f>> {

    let uuid = Arc::as_ptr(function) as u64; // yeah, i need to change this lol

    match &**function {
        Variable(_) => {
            let hash = freeze(function);
            vec![SimplifyReturn { ptr: function.clone(), hash, id: function.id(), mul: 1.0, pow: 1.0 }]
        }
        Constant(val) => {
            let hash = freeze(function);
            vec![SimplifyReturn { ptr: function.clone(), hash, id: function.id(), mul: *val, pow: 1.0 }]
        },
        Add { vec } => {

            // for now we create a new vec that'll become the new contents of Add
            let mut new_add_vec = vec![];

            // iter through children of Add
            vec.iter().for_each(|child| {
                // Simplify child and iterate through grandchildren
                simplify_hash3(child, hmap, id).iter().for_each(|grandchild| {
                    // Hashing the power in for comparison
                    let grandchild_add_hash = freeze(&Pow { base: grandchild.ptr.clone(), raised: Constant(grandchild.pow).into() });
                    // Check if the grandchild is in the hashmap
                    if let Some(hashvals) = hmap.get_mut(&HashKeys { uuid, hash: grandchild_add_hash}) {

                        // If the Mul ptr exists, we can mutate the value
                        if let Some(gc_mul_ptr) = hashvals.mul_ptr.clone() {
                            if let Constant(val) = *gc_mul_ptr {
                                unsafe {
                                    let ptr = Arc::as_ptr(&gc_mul_ptr) as *mut Function;
                                    *ptr = Constant(val + grandchild.mul);
                                }
                            }
                        } else {
                            // Create a new Mul block and getting the pointer to the value inside it
                            let new_mul_const = Arc::new(Constant(grandchild.mul + 1.0)); // We add 1.0 because the previous value was 1.0
                            // if let Constant(ref mut val) = new_mul_const {
                            //     hashvals.mul_ptr = Some(val);
                            // }
                            hashvals.mul_ptr = Some(new_mul_const.clone());

                            // Splicing function tree and inserting new mul block
                            let new_gc = Arc::new(Mul { vec: vec![hashvals.ptr.clone(), new_mul_const] });

                            // Inserting new Mul block into the new Add vec
                            new_add_vec[hashvals.idx] = new_gc.clone();

                            // Updating the hashvals
                            hashvals.ptr = new_gc;
                        }
                    } else {
                        // If the grandchild is not in the hashmap, we add it
                        new_add_vec.push(grandchild.ptr.clone());

                        let (mul_ptr, pow_ptr) = match (grandchild.mul, grandchild.pow) {
                            (1.0, 1.0) => (None, None),
                            (mul, 1.0) => (Some(Arc::new(Constant(mul))), None),
                            (1.0, pow) => (None, Some(Arc::new(Constant(pow)))),
                            (mul, pow) => (Some(Arc::new(Constant(mul))), Some(Arc::new(Constant(pow))))
                        };

                        println!("Created Mul Ptr: {:?}", mul_ptr);
                        println!("Created Mul Val: {:?}", grandchild.mul);

                        hmap.insert(HashKeys { uuid, hash: grandchild_add_hash }, HashVals { ptr: grandchild.ptr.clone(), idx: new_add_vec.len() - 1, hash: grandchild_add_hash, mul_ptr, pow_ptr });
                    }
                })
            });

            let fnc = Add { vec: new_add_vec };
            let hash = freeze(&fnc);
            vec![SimplifyReturn { ptr: Arc::new(fnc), hash, id: function.id(), mul: 1.0, pow: 1.0 }]


            // vec![SimplifyReturn { ptr: function.clone(), hash: 0, id: 0, mul: 0.0, pow: 0.0 }]
        },
        _ => todo!()
    }



}


pub fn stringify_hashmap<'f>(hmap: HashMap<HashKeys, HashVals>) {
    let mut res = String::new();
    for (key, HashVals {ptr, idx, hash, mul_ptr, pow_ptr }) in hmap {
        println!("Key: {:?}, Val: {:?}, Val Ptr: {:?}, Idx: {:?}, Hash: {:?}, Mul Ptr: {:?}, Pow Ptr: {:?}", key, unsafe { &(*ptr) }, ptr, idx, hash, mul_ptr, pow_ptr);
    }
}







#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable, Add, Mul, S};

    #[test]
    fn test_simplify_hash3() {
        let x = Variable("x");
        let y = Variable("y");
        let z = Arc::new(x.clone() + x.clone() + x.clone() + x.clone());
        let z_id = z.id();

        let mut hmap = HashMap::new();
        let res = simplify_hash3(&z, &mut hmap, z_id);
        println!("{:?}", res);
        println!("{:?}", unsafe { &(*res[0].ptr) });
        stringify_hashmap(hmap);
    }
}