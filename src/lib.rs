use std::ops::Deref;
use crate::function::function::Function::{Variable, Constant, S, Add, Sub, Mul, Div};

mod function;

#[cfg(test)]
mod tests {
    use super::*;
    use function::function::Function::{Constant, Variable};
    use function::simplify_nohash::simplify;

    #[test]
    fn hi() {
        let x = Variable("x");
        let y = Variable("y");
        // let z = (((x.clone() + y.clone()) * (x.clone() + y.clone())) * (x.clone() + y.clone())) * ((x.clone() * x.clone()) - (y.clone() * y.clone()));
        // let z = (x.clone() + y.clone()) * (x.clone() + y.clone() + x.clone()) * (x.clone() + y.clone());
        let z = (x.clone() + y.clone()) * x.clone();
        let z_simple = simplify(z.clone());
        println!("{:?}", z);
        println!("{:?}", z_simple);
    }

}
