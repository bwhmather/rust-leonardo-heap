// Copyright 2016 Ben Mather <bwhmather@bwhmather.com>
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

const LEONARDO_NUMBERS: [u64; 64] = [
    1, 1, 3, 5, 9, 15, 25, 41, 67, 109, 177, 287, 465, 753, 1219, 1973, 3193,
    5167, 8361, 13529, 21891, 35421, 57313, 92735, 150049, 242785, 392835,
    635621, 1028457, 1664079, 2692537, 4356617, 7049155, 11405773, 18454929,
    29860703, 48315633, 78176337, 126491971, 204668309, 331160281, 535828591,
    866988873, 1402817465, 2269806339, 3672623805, 5942430145, 9615053951,
    15557484097, 25172538049, 40730022147, 65902560197, 106632582345,
    172535142543, 279167724889, 451702867433, 730870592323, 1182573459757,
    1913444052081, 3096017511839, 5009461563921, 8105479075761, 13114940639683,
    21220419715445,
];

/// Lookup table based implementation of function for determining the nth
/// leonardo number.
#[inline]
fn leonardo_lookup(order: u32) -> usize {
    LEONARDO_NUMBERS[order as usize] as usize
}

/// Closed form implementation of function for determining the nth leonardo
/// number.
#[inline]
fn leonardo_closed(order: u32) -> usize {
    // TODO this starts to diverge due to precision issues at higher orders.
    // Need to figure out how far it is accurate, and raise an assertion error.
    return (
        2.0 * (
            ((1.0 + 5.0f64.sqrt()) / 2.0).powf(order as f64 + 1.0) -
            ((1.0 - 5.0f64.sqrt()) / 2.0).powf(order as f64 + 1.0)
        ) / 5.0f64.sqrt()
    ).floor() as usize - 1;
}

/// Iterative function for determining the nth leonardo number.
#[inline]
fn leonardo_naive(order: u32) -> usize {
    if order < 2 {
        return 1;
    }

    let mut n_minus_2 = 1;
    let mut n_minus_1 = 1;

    let mut i = 2;
    loop {
        let n = n_minus_2 + n_minus_1 + 1;
        if i == order {
            return n;
        }
        i += 1;
        n_minus_2 = n_minus_1;
        n_minus_1 = n;
    }
}

/// Returns the nth leonardo number.
/// Only defined for order less than 64.
pub fn leonardo(order: u32) -> usize {
    return leonardo_lookup(order);
}

#[cfg(test)]
mod tests {
    use leonardo::{leonardo_lookup, leonardo_closed, leonardo_naive};

    #[test]
    fn test_leonardo_lookup_matches() {
        for order in 0..64 {
            assert_eq!(leonardo_lookup(order), leonardo_naive(order));
        }
    }

    #[test]
    fn test_leonardo_closed_matches() {
        for order in 0..70 {
            assert_eq!(leonardo_closed(order), leonardo_naive(order));
        }
    }
}
