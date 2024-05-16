use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::ops::Deref;
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
        Function::Div { den: l, num: r } => {
            let left_hash = freeze(l.deref());
            let right_hash = freeze(r.deref());
            hasher.write_u64(left_hash);
            hasher.write_u64(right_hash);
            hasher.finish()
        }
        _ => todo!()
    }
}