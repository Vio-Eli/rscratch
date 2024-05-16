use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::ops::Deref;

#[derive(Clone)]
enum Function {
    Constant(f64),
    Variable(String),
    Add(Box<Function>, Box<Function>),
    Sub(Box<Function>, Box<Function>),
    Mul(Box<Function>, Box<Function>),
    Div(Box<Function>, Box<Function>),
}

fn freeze(function: &Function) -> u64 {
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

        Function::Add(l, r) |
        Function::Mul(l, r) => {
            let mut class = hasher.finish();
            class ^= freeze(l.deref());
            class ^= freeze(r.deref());
            class
        }
        Function::Sub(l, r) |
            Function::Div(l, r) => {
            let left_hash = freeze(l.deref());
            let right_hash = freeze(r.deref());
            hasher.write_u64(left_hash);
            hasher.write_u64(right_hash);
            hasher.finish()
        }
    }
}

impl Eq for Function {}
impl PartialEq<Self> for Function {
    fn eq(&self, other: &Self) -> bool {
        freeze(self) == freeze(other)
    }
}


fn main() {
    let a = Function::Variable("hello".to_string());
    let b = Function::Variable("hello".to_string());

    let a_uuid = freeze(&a);
    let b_uuid = freeze(&b);

    println!("a: {} | b: {} | => {}", a_uuid, b_uuid, a_uuid == b_uuid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeze_simple() {
        let a = Function::Variable("x".to_string());
        let b = Function::Variable("y".to_string());
        let c = Function::Constant(3.0);

        let d = Function::Add(Box::new(a.clone()), Box::new(b.clone()));
        let e = Function::Sub(Box::new(d.clone()), Box::new(c.clone()));

        let a_uuid = freeze(&a);
        let b_uuid = freeze(&b);
        let c_uuid = freeze(&c);
        let d_uuid = freeze(&d);
        let e_uuid = freeze(&e);

        assert_ne!(a_uuid, b_uuid);
        assert_eq!(a_uuid, freeze(&a));
        assert_eq!(b_uuid, freeze(&b));
        assert_ne!(d_uuid, e_uuid);
    }

    #[test]
    fn test_freeze_large_tree() {
        let x = Function::Variable("x".to_string());
        let y = Function::Variable("y".to_string());
        let z = Function::Variable("z".to_string());
        let c1 = Function::Constant(5.0);
        let c2 = Function::Constant(2.0);

        let expr1 = Function::Add(
            Box::new(Function::Mul(Box::new(x.clone()), Box::new(y.clone()))),
            Box::new(Function::Sub(Box::new(z.clone()), Box::new(c1.clone()))),
        );

        let expr2 = Function::Div(
            Box::new(Function::Add(
                Box::new(Function::Mul(Box::new(y.clone()), Box::new(c2.clone()))),
                Box::new(Function::Sub(Box::new(z.clone()), Box::new(x.clone()))),
            )),
            Box::new(Function::Add(Box::new(c1.clone()), Box::new(c2.clone()))),
        );

        let expr1_uuid = freeze(&expr1);
        let expr2_uuid = freeze(&expr2);

        assert_ne!(expr1_uuid, expr2_uuid);
        assert_eq!(expr1_uuid, freeze(&expr1));
        assert_eq!(expr2_uuid, freeze(&expr2));
    }

    #[test]
    fn test_freeze_identical_trees() {
        let x = Function::Variable("x".to_string());
        let y = Function::Variable("y".to_string());
        let z = Function::Variable("z".to_string());
        let c1 = Function::Constant(5.0);

        let expr1 = Function::Add(
            Box::new(Function::Mul(Box::new(x.clone()), Box::new(y.clone()))),
            Box::new(Function::Sub(Box::new(z.clone()), Box::new(c1.clone()))),
        );

        let expr1_clone = Function::Add(
            Box::new(Function::Mul(Box::new(x.clone()), Box::new(y.clone()))),
            Box::new(Function::Sub(Box::new(z.clone()), Box::new(c1.clone()))),
        );

        let expr1_uuid = freeze(&expr1);
        let expr1_clone_uuid = freeze(&expr1_clone);

        assert_eq!(expr1_uuid, expr1_clone_uuid);
    }

    #[test]
    fn test_freeze_deep_nesting() {
        let x = Function::Variable("x".to_string());

        // Deeply nested expression
        let expr = (0..100).fold(x.clone(), |acc, _| Function::Add(Box::new(acc), Box::new(x.clone())));

        let expr_uuid = freeze(&expr);

        // Ensure the deep nested structure has a consistent hash
        assert_eq!(expr_uuid, freeze(&expr));
    }

    #[test]
    fn test_freeze_commutativity() {
        let x = Function::Variable("x".to_string());
        let y = Function::Variable("y".to_string());

        let expr1 = Function::Add(Box::new(x.clone()), Box::new(y.clone()));
        let expr2 = Function::Add(Box::new(y.clone()), Box::new(x.clone()));

        let expr1_uuid = freeze(&expr1);
        let expr2_uuid = freeze(&expr2);

        // Ensure commutativity is handled correctly for Add and Mul
        assert_eq!(expr1_uuid, expr2_uuid);

        let expr3 = Function::Mul(Box::new(x.clone()), Box::new(y.clone()));
        let expr4 = Function::Mul(Box::new(y.clone()), Box::new(x.clone()));

        let expr3_uuid = freeze(&expr3);
        let expr4_uuid = freeze(&expr4);

        assert_eq!(expr3_uuid, expr4_uuid);
    }

    #[test]
    fn test_freeze_associativity() {
        let x = Function::Variable("x".to_string());
        let y = Function::Variable("y".to_string());
        let z = Function::Variable("z".to_string());

        let expr1 = Function::Add(Box::new(Function::Add(Box::new(x.clone()), Box::new(y.clone()))), Box::new(z.clone()));
        let expr2 = Function::Add(Box::new(x.clone()), Box::new(Function::Add(Box::new(y.clone()), Box::new(z.clone()))));

        let expr1_uuid = freeze(&expr1);
        let expr2_uuid = freeze(&expr2);

        // Ensure associativity is handled correctly for Add and Mul
        assert_eq!(expr1_uuid, expr2_uuid);

        let expr3 = Function::Sub(Box::new(Function::Sub(Box::new(x.clone()), Box::new(y.clone()))), Box::new(z.clone()));
        let expr4 = Function::Sub(Box::new(x.clone()), Box::new(Function::Sub(Box::new(y.clone()), Box::new(z.clone()))));

        let expr3_uuid = freeze(&expr3);
        let expr4_uuid = freeze(&expr4);

        assert_ne!(expr3_uuid, expr4_uuid);
    }

    #[test]
    fn test_freeze_floating_point_edge_cases() {
        let c1 = Function::Constant(0.0);
        let c2 = Function::Constant(-0.0);
        let c3 = Function::Constant(f64::NAN);
        let c4 = Function::Constant(f64::INFINITY);
        let c5 = Function::Constant(f64::NEG_INFINITY);

        assert_ne!(freeze(&c1), freeze(&c2));
        assert_ne!(freeze(&c1), freeze(&c3));
        assert_ne!(freeze(&c1), freeze(&c4));
        assert_ne!(freeze(&c1), freeze(&c5));
        assert_ne!(freeze(&c2), freeze(&c3));
        assert_ne!(freeze(&c2), freeze(&c4));
        assert_ne!(freeze(&c2), freeze(&c5));
        assert_ne!(freeze(&c3), freeze(&c4));
        assert_ne!(freeze(&c3), freeze(&c5));
        assert_ne!(freeze(&c4), freeze(&c5));
    }
}
