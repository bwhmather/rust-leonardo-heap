// Copyright 2016 Ben Mather <bwhmather@bwhmather.com>
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::mem;

use leonardo::leonardo;
use subheap::SubHeapMut;


#[derive(Clone, Debug)]
pub struct Layout {
    orders: u64,
    size: usize,
}


impl Layout {
    pub fn new() -> Self {
        Layout {
            orders: 0,
            size: 0,
        }
    }

    pub fn new_from_len(size: usize) -> Self {
        let mut orders = 0;
        let mut remaining = size;

        for order in (0..63).rev() {
            if leonardo(order) <= remaining {
                remaining -= leonardo(order);
                orders |= 1 << order;
            }
        }

        Layout {
            orders: orders,
            size: size,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn push(&mut self) {
        self.size += 1;

        match self.lowest_order() {
            Some(lowest_order) => {
                let mergeable_mask : u64 = 3 << lowest_order;

                if (mergeable_mask & self.orders) == mergeable_mask {
                    // lowest two sub-heaps are adjacent and car be merged
                    // Clear the two lowest orders
                    self.orders &= !mergeable_mask;

                    // Replace them with the next order up
                    self.orders |= 1 << (lowest_order + 2);
                } else if lowest_order == 1 {
                    self.orders |= 1;
                } else {
                    self.orders |= 2;
                }
            }
            None => {
                self.orders |= 2;
            }
        }
    }

    pub fn pop(&mut self) {
        if self.size == 0 {
            return;
        }

        self.size -= 1;

        match self.lowest_order() {
            Some(lowest_order) => {
                // Clear the order
                let mask : u64 = 1 << lowest_order;
                self.orders &= !mask;

                // If the order is not zero or one (the single element orders)
                // then we need to split it into two heaps of size n-1 and n-2
                if lowest_order != 0 && lowest_order != 1 {
                    let mask : u64 = 3 << lowest_order - 2;
                    self.orders |= mask;
                }
            }
            None => {}
        }
    }

    #[inline]
    pub fn lowest_order(&self) -> Option<u32> {
        match self.orders.trailing_zeros() {
            64 => None,
            n => Some(n),
        }
    }

    pub fn iter<'a, T : Ord + Debug>(&self, data : &'a mut [T]) -> IterMut<'a, T> {
        assert_eq!(data.len(), self.size);

        IterMut {
            heap_data: data,
            orders: self.orders,
        }
    }
}


#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    heap_data: &'a mut [T],
    orders: u64,
}


impl<'a, T : Ord + Debug> Iterator for IterMut<'a, T>
{
    type Item = SubHeapMut<'a, T>;

    fn next(&mut self) -> Option<SubHeapMut<'a, T>> {
        if self.orders != 0 {
            // Records and remove the first order from the font of the bitset
            // This is the order of the sub-heap at the start of the heap
            let order = self.orders.trailing_zeros();
            self.orders ^= 1 << order;

            // We need to pre-calculate the length to get around the fact that
            // the borrow checker can't yet handle borrowing in for only as
            // long as is needed to calculate the argument to a function
            let heap_len = self.heap_data.len();

            // In order to avoid having more than one mutable reference to the
            // heap at any one time,we have to temporarily replace it in self
            // with a placeholder value.
            let mut heap_data = mem::replace(&mut self.heap_data, &mut []);

            // Split the heap into the part belonging to this sub-heap and all
            // of the rest
            let (mut rest_data, mut subheap_data) = heap_data.split_at_mut(
                heap_len - leonardo(order)
            );

            // Store what's left of the heap back in self
            self.heap_data = rest_data;

            Some(SubHeapMut::new(subheap_data, order))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones = self.orders.count_ones() as usize;
        (ones, Some(ones))
    }
}
