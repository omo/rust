// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// pretty-expanded FIXME #23616

trait A {
    fn g<T>(&self, x: T, y: T) -> (T, T) { (x, y) }
}

impl A for isize { }

fn f<T, V: A>(i: V, j: T, k: T) -> (T, T) {
    i.g(j, k)
}

pub fn main () {
    assert_eq!(f(0, 1, 2), (1, 2));
    assert_eq!(f(0, 1u8, 2u8), (1u8, 2u8));
}
