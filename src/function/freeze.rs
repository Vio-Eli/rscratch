use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::ops::Deref;
use std::sync::Arc;
use crate::function::function::Function;

pub fn freeze(function: &Function) -> u64 {
    let mut hasher = DefaultHasher::new();
    discriminant(function).hash(&mut hasher); // hash the discriminant
    match function {
        Function::Constant(c) => {
            let mut class = hasher.finish();
            class ^= c.to_bits();
            class
        }
        Function::Variable(v) => {
            v.hash(&mut hasher);
            hasher.finish()
        }
        Function::S { f, m, p } => {
            let mut class = hasher.finish();
            class ^= freeze(f.deref());
            class ^= m.to_bits();
            class ^= p.to_bits();
            class
        }
        Function::Add { vec } |
        Function::Mul { vec } => {
            let mut class = hasher.finish();
            for f in vec {
                class ^= freeze(f.deref());
            }
            class
        }
        Function::Sub { lhs: l, rhs: r} |
        Function::Div { den: l, num : r } |
        Function::Pow { base: l, raised: r } => {
            let left_hash = freeze(l.deref());
            let right_hash = freeze(r.deref());
            hasher.write_u64(left_hash);
            hasher.write_u64(right_hash);
            hasher.finish()
        }
        _ => todo!()
    }
}

// strategey:
// keep track of the nodes memory address.
// > mem address might need to be Pin<F>
// > but first try to just use the actuall Arc<F> fat ptr to compare since that will 100% not move
// we have a memaddr lookup table for the hashes to avoid haveing to recalc type
// we would have to do a hash table lookup for each node traversal
// we get the mem addr for each node and see if its in the hashtable
// if it is we use the lookup table [ltab] then we use the value
// > the ltab lookup will happen before we recurse down the tree in the "down travel" step
// the last step in the new freeze algo will be to add the cacluated freeze for the head node to the table
// u could do this every step but you prob dont need to.
// th optomization could be that you just do the one on the parent node that the new freeze was called on
// not everynode (when its called recurivly)

// pub fn propagate_freeze(function: &Function, freeze_table: HashMap<Arc<Function>, u64>) -> u64 {
//
// }

pub fn freeze_2(function: &Function, hashes: Vec<u64>) -> u64 {
    let mut hasher = DefaultHasher::new();
    discriminant(function).hash(&mut hasher); // hash the discriminant

    todo!()
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::function::Function::{Constant, Variable, Add, Mul, S};

    #[test]
    fn test_freeze() {
        let x = Variable("x");
        let y = Variable("y");
        let z = x.clone() * y.clone();
        let z2 = y.clone() * x.clone();
        let z_simple = freeze(&z);
        let z2_simple = freeze(&z2);
        println!("{:?}", z);
        println!("{:?}", z_simple);
        assert_eq!(z_simple, z2_simple);
    }
}
