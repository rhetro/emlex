use emlex::prelude::*;

fn main() {
    // Extra arguments not expected by the TT-muncher
    let _ = eml!((eml x y z));
}
