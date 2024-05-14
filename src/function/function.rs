use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Function<'f> {
    Variable(&'f str),
    Constant(f64),
    S {
        f: Arc<Function<'f>>,
        m: f64,
        p: f64,
    },
    Add {
        vec: Vec<Arc<Function<'f>>>
    },
    Sub {
        lhs: Arc<Function<'f>>,
        rhs: Arc<Function<'f>>,
    },
    Mul {
        vec: Vec<Arc<Function<'f>>>
    },
    Div {
        num: Arc<Function<'f>>,
        den: Arc<Function<'f>>,
    }
}

// pub enum Simplify<'f> {
//     S {
//         f: Arc<Function<'f>>,
//         m: f64,
//         p: f64,
//     },
// }

// Define the way that Equality interacts for taking derivatives
impl<'f> PartialEq for Function<'f> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Variable(lhs), Function::Variable(rhs)) => lhs == rhs,
            (Function::Constant(lhs), Function::Constant(rhs)) => lhs == rhs,
            (
                Function::S {
                    f: lhs,
                    m: lhs_mul,
                    p: lhs_pow,
                },
                Function::S {
                    f: rhs,
                    m: rhs_mul,
                    p: rhs_pow,
                },
            ) => lhs == rhs && lhs_mul == rhs_mul && lhs_pow == rhs_pow,
            (
                Function::Add {
                    vec: lhs,
                },
                Function::Add {
                    vec: rhs,
                },
            )
            | (
                Function::Mul {
                    vec: lhs,
                },
                Function::Mul {
                    vec: rhs,
                },
            ) => {
                let mut lhs_sorted = lhs.clone();
                let mut rhs_sorted = rhs.clone();
                lhs_sorted.sort();
                rhs_sorted.sort();
                lhs.len() == rhs.len() && lhs_sorted.iter().zip(rhs_sorted.iter()).all(|(l, r)| l == r)
            },
            (
                Function::Sub {
                    lhs: lhs1,
                    rhs: rhs1,
                },
                Function::Sub {
                    lhs: lhs2,
                    rhs: rhs2,
                },
            ) => lhs1 == lhs2 && rhs1 == rhs2,
            _ => false
        }
    }
}

impl<'f> Eq for Function<'f> {}

impl<'f> PartialOrd for Function<'f> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'f> Ord for Function<'f> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Function::Variable(lhs_name), Function::Variable(rhs_name)) => lhs_name.cmp(rhs_name),
            (Function::Variable(_), _) => Ordering::Less,
            (_, Function::Variable(_)) => Ordering::Greater,
            (Function::Constant(lhs_val), Function::Constant(rhs_val)) => lhs_val.partial_cmp(rhs_val).unwrap(),
            (Function::Constant(_), _) => Ordering::Less,
            (_, Function::Constant(_)) => Ordering::Greater,
            (
                Function::S {
                    f: lhs,
                    m: lhs_mul,
                    p: lhs_pow,
                },
                Function::S {
                    f: rhs,
                    m: rhs_mul,
                    p: rhs_pow,
                },
            ) => {
                match lhs.cmp(rhs) {
                    Ordering::Equal => {
                        match lhs_mul.partial_cmp(rhs_mul).unwrap() {
                            Ordering::Equal => lhs_pow.partial_cmp(rhs_pow).unwrap(),
                            other => other
                        }
                    },
                    other => other
                }
            },
            (
                Function::Add {
                    vec: lhs,
                },
                Function::Add {
                    vec: rhs,
                },
            ) => lhs.len().cmp(&rhs.len()),
            (
                Function::Mul {
                    vec: lhs,
                },
                Function::Mul {
                    vec: rhs,
                },
            ) => lhs.len().cmp(&rhs.len()),
            _ => todo!(),
        }
    }
}