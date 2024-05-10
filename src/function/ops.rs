use crate::function::function::Function;
use std::ops::{Add, Div, Mul, Sub};


impl<'f> Add for Function<'f> {
    type Output = Function<'f>;
    #[inline]
    fn add(self, rhs: Function<'f>) -> Self::Output {
        let lhs: Function = self.into();
        let rhs: Function = rhs.into();
        println!("lhs: {:?}, rhs: {:?}", lhs, rhs);
        match (&lhs, &rhs) {
            (&Function::Constant(lhs), &Function::Constant(rhs)) => Function::Constant(lhs + rhs),
            (&Function::Add{vec: ref lhs}, &Function::Add{vec: ref rhs}) => {
                let mut vec = lhs.clone();
                vec.extend_from_slice(&rhs);
                Function::Add{ vec }
            },
            (&Function::Add{vec: ref lhs}, _) => {
                let mut vec = lhs.clone();
                vec.push(rhs.into());
                Function::Add{ vec }
            },
            (_, &Function::Add{vec: ref rhs}) => {
                let mut vec = rhs.clone();
                vec.push(lhs.into()); // order doesn't matter as it's addition
                Function::Add{ vec }
            },
            _ => Function::Add {
                vec: vec![lhs.into(), rhs.into()]
            }
        }
    }
}

impl<'f> Sub for Function<'f> {
    type Output = Function<'f>;
    #[inline]
    fn sub(self, rhs: Function<'f>) -> Self::Output {
        let lhs: Function = self.into();
        let rhs: Function = rhs.into();
        match (&lhs, &rhs) {
            (&Function::Constant(lhs), &Function::Constant(rhs)) => Function::Constant(lhs - rhs),
            _ => Function::Sub {
                lhs: lhs.into(),
                rhs: rhs.into()
            }
        }
    }
}

impl<'f> Mul for Function<'f> {
    type Output = Function<'f>;
    #[inline]
    fn mul(self, rhs: Function<'f>) -> Self::Output {
        let lhs: Function = self.into();
        let rhs: Function = rhs.into();
        match (&lhs, &rhs) {
            (&Function::Constant(lhs), &Function::Constant(rhs)) => Function::Constant(lhs * rhs),
            (&Function::Mul{vec: ref lhs}, &Function::Mul{vec: ref rhs}) => {
                let mut vec = lhs.clone();
                vec.extend_from_slice(&rhs);
                Function::Mul{ vec }
            },
            (&Function::Mul{vec: ref lhs}, _) => {
                let mut vec = lhs.clone();
                vec.push(rhs.into());
                Function::Mul{ vec }
            },
            (_, &Function::Mul{vec: ref rhs}) => {
                let mut vec = rhs.clone();
                vec.push(lhs.into()); // order doesn't matter as it's multiplication
                Function::Mul{ vec }
            },
            _ => Function::Mul {
                vec: vec![lhs.into(), rhs.into()]
            }
        }
    }
}

impl<'f> Div for Function<'f> {
    type Output = Function<'f>;
    #[inline]
    fn div(self, rhs: Function<'f>) -> Self::Output {
        let lhs: Function = self.into();
        let rhs: Function = rhs.into();
        match (&lhs, &rhs) {
            (&Function::Constant(lhs), &Function::Constant(rhs)) => Function::Constant(lhs / rhs),
            _ => Function::Div {
                num: lhs.into(),
                den: rhs.into()
            }
        }
    }
}