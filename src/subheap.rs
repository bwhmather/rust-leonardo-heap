// Copyright 2016 Ben Mather <bwhmather@bwhmather.com>
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;

use leonardo::leonardo;

#[derive(Clone, Debug)]
pub struct SubHeap<'a, T: 'a> {
    data: &'a [T],
    pub order: u32,
}


impl<'a, T: Ord + Debug> SubHeap<'a, T> {
    pub fn new(data: &[T], order: u32) -> SubHeap<T> {
        assert_eq!(data.len(), leonardo(order));

        SubHeap {
            data: data,
            order: order,
        }
    }

    /// Unpacks the head and, if they exist, the two smaller subheaps that make
    /// up the rest of this subheap.
    pub fn destructure(&self) -> (&T, Option<(SubHeap<T>, SubHeap<T>)>) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (value, body) = self.data.split_last().unwrap();
            let (snd_data, fst_data) = body.split_at(leonardo(snd_order));

            (value, Some((
                SubHeap::new(fst_data, fst_order),
                SubHeap::new(snd_data, snd_order)))
            )
        } else {
            (self.value(), None)
        }
    }

    /// Returns a reference to the value at the head of the subheap.
    #[inline]
    pub fn value(&self) -> &T {
        self.data.last().unwrap()
    }

    /// If the subheap is of third order or greater returns references to the
    /// two child subheaps containing all values below the head.
    #[inline]
    pub fn children(&self) -> Option<(SubHeap<T>, SubHeap<T>)> {
        let (_, children) = self.destructure();
        children
    }
}


#[derive(Debug)]
pub struct SubHeapMut<'a, T: 'a> {
    data: &'a mut [T],
    pub order: u32,
}


impl<'a, T: Ord + Debug> SubHeapMut<'a, T> {
    pub fn new(data: &mut [T], order: u32) -> SubHeapMut<T> {
        assert_eq!(data.len(), leonardo(order));

        SubHeapMut {
            data: data,
            order: order,
        }
    }

    /// Returns references to the head and, if they exist, the two smaller
    /// subheaps that make up the rest of this subheap allowing them to be read
    /// simultaneously.
    pub fn destructure(&self) -> (&T, Option<(SubHeap<T>, SubHeap<T>)>) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (value, body) = self.data.split_last().unwrap();
            let (snd_data, fst_data) = body.split_at(leonardo(snd_order));

            (value, Some((
                SubHeap::new(fst_data, fst_order),
                SubHeap::new(snd_data, snd_order)))
            )
        } else {
            (self.value(), None)
        }
    }

    /// Returns mutable references to the head and, if this subheap is of third
    /// order or greater, the two smaller subheaps that make up the rest
    /// allowing them to be read and modified simultaneously.
    pub fn destructure_mut(&mut self) -> (&mut T, Option<(SubHeapMut<T>, SubHeapMut<T>)>) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (value, body) = self.data.split_last_mut().unwrap();
            let (snd_data, fst_data) = body.split_at_mut(
                leonardo(snd_order),
            );

            (value, Some((
                SubHeapMut::new(fst_data, fst_order),
                SubHeapMut::new(snd_data, snd_order),
            )))
        } else {
            (self.value_mut(), None)
        }
    }

    /// Breaks the subheap up into its constituent parts: the head and for
    /// subheaps of third order or greater the two child subheaps.
    pub fn into_components(self) -> (
        &'a mut T, Option<(SubHeapMut<'a, T>, SubHeapMut<'a, T>)>,
    ) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (value, body) = self.data.split_last_mut().unwrap();
            let (snd_data, fst_data) = body.split_at_mut(
                leonardo(snd_order),
            );

            (value, Some((
                SubHeapMut::new(fst_data, fst_order),
                SubHeapMut::new(snd_data, snd_order),
            )))
        } else {
            (self.into_value(), None)
        }
    }

    /// Returns a reference to the value at the head of the subheap.
    #[inline]
    pub fn value(&self) -> &T {
        self.data.last().unwrap()
    }

    /// Returns a mutable reference to the value at the head of the subheap.
    #[inline]
    pub fn value_mut(&mut self) -> &mut T {
        self.data.last_mut().unwrap()
    }

    /// Consumes a subheap and returns a mutable reference to the value at the
    /// head of the data that it points to.
    #[inline]
    pub fn into_value(self) -> &'a mut T {
        self.data.last_mut().unwrap()
    }

    #[inline]
    fn children(&self) -> Option<(SubHeap<T>, SubHeap<T>)> {
        let (_, children) = self.destructure();
        children
    }

    #[inline]
    fn children_mut(&mut self) -> Option<(SubHeapMut<T>, SubHeapMut<T>)> {
        let (_, children) = self.destructure_mut();
        children
    }

    #[inline]
    fn into_children(self) -> Option<(SubHeapMut<'a, T>, SubHeapMut<'a, T>)> {
        let (_, children) = self.into_components();
        children
    }
}
