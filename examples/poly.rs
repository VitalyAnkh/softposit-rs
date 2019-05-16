use softposit::{Polynom, P32};

fn p(x: f64) -> P32 {
    x.into()
}

fn main() {
    // x = 1.1000000014901161
    // 5.199999988079071×x^5−12.100000023841858×x^4−3.2999999970197678×x^3+0.6000000014901161×x^2+15 = 1.99274189651072
    let c = [p(15.), p(0.), p(0.6), p(-3.3), p(-12.1), p(5.2)];
    let x = p(1.1);
    println!("Polynom = {}", x.poly5(&c));
}
