// Copyright 2016 Ben Mather <bwhmather@bwhmather.com>
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A binary heap structure supporting fast in-place partial sorting.
//!
//! This is structure is the core of Dijkstra's Smoothsort algorithm.
#[cfg(test)]
extern crate rand;

mod leonardo;
mod subheap;
mod layout;

use std::fmt::Debug;

use subheap::SubHeapMut;


fn sift_down<T: Ord + Debug>(heap: &mut SubHeapMut<T>) {
    let (mut this_value, mut children) = heap.destructure_mut();

    loop {
        // No children.  We have reached the bottom of the heap.
        if children.is_none() {
            break;
        }

        let (fst_child, snd_child) = children.unwrap();

        // Find the largest child.  Prefer the furthest child if both children
        // are the same as doing so makes the array slightly more sorted.
        let mut next_heap = if fst_child.value() > snd_child.value() {
            fst_child
        } else {
            snd_child
        };

        // The heap property is satisfied.  No need to do anything else.
        if &*this_value >= next_heap.value() {
            break;
        }

        // Swap the value of the parent with the value of the largest child.
        std::mem::swap(this_value, next_heap.value_mut());

        // TODO there has to be a better pattern for unpacking to existing vars
        match next_heap.into_components() {
            (v, n) => {
                this_value = v;
                children = n;
            }
        }
    }
}


fn restring<T : Ord + Debug>(mut subheap_iter: layout::IterMut<T>) {
    if let Some(mut this_subheap) = subheap_iter.next() {
        for mut next_subheap in subheap_iter {
            if next_subheap.value() <= this_subheap.value() {
                break;
            }

            std::mem::swap(next_subheap.value_mut(), this_subheap.value_mut());

            sift_down(&mut next_subheap);

            this_subheap = next_subheap;
        }
    }
}


fn balance_after_push<T: Ord + Debug>(
    heap_data: &mut [T], layout: &layout::Layout,
) {
    assert_eq!(heap_data.len(), layout.len());

    sift_down(&mut layout.iter(heap_data).next().unwrap());
    restring(layout.iter(heap_data));
}


fn balance_after_pop<T: Ord + Debug>(
    heap_data: &mut [T], layout: &layout::Layout,
) {
    {
        let mut subheap_iter = layout.iter(heap_data);
        match (subheap_iter.next(), subheap_iter.next()) {
            (Some(fst), Some(snd)) => {
                if snd.order - fst.order != 1 {
                    return
                }
            }
            _ => {
                return;
            }
        }
    }

    {
        let mut subheaps_from_snd = layout.iter(heap_data);
        // Consume the first subheap.
        subheaps_from_snd.next();

        restring(subheaps_from_snd);
    }

    {
        let subheaps_from_fst = layout.iter(heap_data);
        restring(subheaps_from_fst);
    }
}


#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    heap_data: &'a mut [T],
    layout: layout::Layout,
}


impl<'a, T : Ord + Debug> Iterator for Iter<'a, T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.layout.pop();

        if self.heap_data.len() != 0 {
            // In order to avoid having more than one mutable reference to the
            // heap at any one time,we have to temporarily replace it in self
            // with a placeholder value.
            let heap_data = std::mem::replace(&mut self.heap_data, &mut []);

            let (result, rest_data) = heap_data.split_last_mut().unwrap();

            // Store what's left of the heap back in self.
            self.heap_data = rest_data;

            balance_after_pop(self.heap_data, &self.layout);

            Some(&*result)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.heap_data.len(), Some(self.heap_data.len()))
    }
}


impl<'a, T : Ord + Debug> ExactSizeIterator for Iter<'a, T> {}


#[derive(Debug)]
pub struct Drain<'a, T: 'a> {
    heap: &'a mut LeonardoHeap<T>,
}


impl<'a, T: Ord + Debug> Iterator for Drain<'a, T>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.heap.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.heap.len(), Some(self.heap.len()))
    }
}


impl<'a, T : Ord + Debug> ExactSizeIterator for Drain<'a, T> {}


#[derive(Debug)]
pub struct LeonardoHeap<T> {
    data: Vec<T>,
    layout: layout::Layout,
}


impl<T: Ord + Debug> LeonardoHeap<T> {
    /// Creates a new, empty `LeonardoHeap<T>`
    pub fn new() -> Self {
        LeonardoHeap {
            data: Vec::new(),
            layout: layout::Layout::new(),
        }
    }

    /// Creates a new `LeonardoHeap<T>` with space allocated for at least
    /// `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        LeonardoHeap {
            data: Vec::with_capacity(capacity),
            layout: layout::Layout::new(),
        }
    }

    /// Returns the number of elements for which space has been allocated.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve at least enough space for `additional` elements to be pushed
    /// on to the heap.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` elements to be
    /// pushed onto the heap.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional)
    }

    /// Shrinks the capacity of the underlying storage to free up as much space
    /// as possible.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    /// Removes all elements from the heap that do not match a predicate.
    pub fn retain<F>(&mut self, f: F)
        where F: FnMut(&T) -> bool
    {
        // TODO there is a much more interesting implementation
        self.data.retain(f);

        self.heapify();
    }

    /// Removes all elements from the heap.
    pub fn clear(&mut self) {
        self.data.clear();
        self.layout = layout::Layout::new();
    }

    /// Returns the number of elements in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the heap contains no elements, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes duplicate elements from the heap, preserving heap order.
    pub fn dedup(&mut self) {
        self.sort();
        self.data.dedup();
        self.heapify();
    }

    fn heapify(&mut self) {
        let mut layout = layout::Layout::new();

        // TODO harmless off-by-one error
        for i in 0..self.data.len() {
            balance_after_push(&mut self.data[0..i], &layout);
            layout.push();
        }
    }

    /// Forces sorting of the entire underlying array.  The sorted array is
    /// still a valid leonardo heap.
    pub fn sort(&mut self) {
        let mut layout = self.layout.clone();

        // TODO harmless off-by-one error
        for i in (0..self.data.len()).rev() {
            layout.pop();
            balance_after_pop(&mut self.data[0..i], &layout);
        }
    }

    /// Adds a new element to the heap.  The heap will be rebalanced to
    /// maintain the string and heap properties.
    ///
    /// Elements pushed more than once will not be deduplicated.
    pub fn push(&mut self, item: T) {
        self.data.push(item);
        self.layout.push();

        balance_after_push(self.data.as_mut_slice(), &self.layout);
    }

    /// Returns a reference to the largest element in the heap without removing
    /// it.
    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.data.len())
    }

    /// Removes and returns the largest element in the heap.  If the heap is
    /// empty, returns `None`.
    pub fn pop(&mut self) -> Option<T> {
        let result = self.data.pop();
        self.layout.pop();

        balance_after_pop(self.data.as_mut_slice(), &self.layout);

        result
    }

    /// Returns a *sorted* iterator over the elements in the heap.
    ///
    /// Will lazily sort the top elements of the heap in-place as it is
    /// consumed.
    pub fn iter(&mut self) -> Iter<T> {
        Iter {
            heap_data: self.data.as_mut_slice(),
            layout: self.layout.clone(),
        }
    }

    /// Returns an iterator that removes and returns elements from the top of
    /// the heap.
    pub fn drain(&mut self) -> Drain<T> {
        // TODO should drain clear the heap if not fully consumed
        Drain {
            heap: self,
        }
    }
}


#[cfg(test)]
mod tests {
    use rand;
    use rand::Rng;

    use layout;
    use subheap::SubHeapMut;
    use {LeonardoHeap, sift_down, balance_after_push, balance_after_pop};

    #[test]
    fn test_sift_down_zero() {
        let mut subheap_data = [1];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 0));
        assert_eq!(subheap_data, [1]);
    }

    #[test]
    fn test_sift_down_one() {
        let mut subheap_data = [1];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 1));
        assert_eq!(subheap_data, [1]);
    }

    #[test]
    fn test_sift_down_two() {
        let mut subheap_data = [3, 2, 1];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 2));
        assert_eq!(subheap_data, [1, 2, 3]);

        let mut subheap_data = [3, 5, 4];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 2));
        assert_eq!(subheap_data, [3, 4, 5]);

        let mut subheap_data = [6, 7, 8];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 2));
        assert_eq!(subheap_data, [6, 7, 8]);
    }

    #[test]
    fn test_sift_down_three() {
        let mut subheap_data = [1, 2, 3, 4, 5];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [1, 2, 3, 4, 5]);

        let mut subheap_data = [1, 2, 3, 5, 4];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [1, 2, 3, 4, 5]);

        let mut subheap_data = [1, 2, 5, 4, 3];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [1, 2, 3, 4, 5]);

        let mut subheap_data = [2, 3, 5, 4, 1];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [2, 1, 3, 4, 5]);

        let mut subheap_data = [3, 2, 5, 4, 1];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_sift_down_sorting() {
        let mut subheap_data = [5, 5, 4];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 2));
        assert_eq!(subheap_data, [4, 5, 5]);

        let mut subheap_data = [1, 2, 4, 4, 3];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 3));
        assert_eq!(subheap_data, [1, 2, 3, 4, 4]);
    }

    #[test]
    #[should_panic]
    fn test_sift_down_wrong_order() {
        let mut subheap_data : [i32; 0] = [];
        sift_down(&mut SubHeapMut::new(&mut subheap_data, 0));
    }

    #[test]
    fn test_balance_after_push_first() {
        let mut subheap_data = [1];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(1),
        );
        assert_eq!(subheap_data, [1]);
    }

    #[test]
    fn test_balance_after_push_second() {
        let mut subheap_data = [1, 2];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(2),
        );
        assert_eq!(subheap_data, [1, 2]);

        let mut subheap_data = [2, 1];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(2),
        );
        assert_eq!(subheap_data, [1, 2]);
    }

    #[test]
    fn test_balance_after_push_merge() {
        let mut subheap_data = [1, 2, 3];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(3),
        );
        assert_eq!(subheap_data, [1, 2, 3]);

        let mut subheap_data = [1, 3, 2];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(3),
        );
        assert_eq!(subheap_data, [1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_balance_after_push_mismatched_lengths() {
        let mut subheap_data = [1, 2, 3, 4];
        balance_after_push(
            &mut subheap_data, &layout::Layout::new_from_len(12),
        );
    }

    #[test]
    fn test_balance_after_pop_empty() {
        let mut subheap_data : [i32; 0]= [];
        balance_after_pop(&mut subheap_data, &layout::Layout::new_from_len(0));
        assert_eq!(subheap_data, []);
    }

    #[test]
    fn test_balance_after_pop_one() {
        let mut heap_data = [1];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(1));
        assert_eq!(heap_data, [1]);
    }

    #[test]
    fn test_balance_after_pop_two() {
        let mut heap_data = [1, 2];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(2));
        assert_eq!(heap_data, [1, 2]);

        let mut heap_data = [2, 1];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(2));
        assert_eq!(heap_data, [1, 2]);
    }

    #[test]
    fn test_balance_after_pop_split_heaps() {
        let mut heap_data = [1, 2, 3, 4, 5, 6, 7];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);

        let mut heap_data = [1, 2, 3, 4, 5, 7, 6];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);

        let mut heap_data = [1, 2, 3, 4, 6, 5, 7];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);

        let mut heap_data = [1, 2, 3, 4, 7, 5, 6];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);

        let mut heap_data = [1, 2, 3, 4, 6, 7, 5];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);

        let mut heap_data = [1, 2, 3, 4, 7, 6, 5];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(7));
        assert_eq!(heap_data, [1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_balance_after_pop_restring_after_sift() {
        let mut heap_data = [
            1, 2, 3, 4, 5, 6, 10, 11, 12,
            9, 7, 13,
            8
        ];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(13));
        assert_eq!(heap_data, [
            1, 2, 3, 4, 5, 6, 9, 10, 11,
            8, 7, 12,
            13,
        ]);
    }

    #[test]
    fn test_balance_after_pop_mutiple_layers() {
        let mut heap_data = [
            3, 0, 5, 1, 9, 2, 6, 7, 10,
            4,
            8,
        ];
        balance_after_pop(&mut heap_data, &layout::Layout::new_from_len(11));
        assert_eq!(heap_data, [
            3, 0, 4, 1, 5, 2, 6, 7, 8,
            9,
            10,
        ]);
    }

    #[test]
    #[should_panic]
    fn test_balance_after_pop_mismatched_lengths() {
        let mut subheap_data = [1, 2, 3, 4];
        balance_after_pop(
            &mut subheap_data, &layout::Layout::new_from_len(12),
        );
    }

    #[test]
    fn test_push_pop() {
        let mut heap = LeonardoHeap::new();
        heap.push(4);
        heap.push(1);
        heap.push(2);
        heap.push(3);

        assert_eq!(heap.pop(), Some(4));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(1));
    }

    #[test]
    fn test_random() {
        let mut rng = rand::thread_rng();

        let mut inputs : Vec<i32> = (0..200).collect();

        let mut expected = inputs.clone();
        expected.sort_by(|a, b| b.cmp(a));

        rng.shuffle(inputs.as_mut_slice());

        let mut heap = LeonardoHeap::new();
        for input in inputs {
            heap.push(input.clone());
        }

        let mut outputs: Vec<i32> = Vec::new();
        while let Some(output) = heap.pop() {
            outputs.push(output);
        }

        assert_eq!(outputs, expected);
    }

    #[test]
    fn test_sort_random() {
        let mut rng = rand::thread_rng();

        let mut inputs : Vec<i32> = (0..200).collect();

        let mut expected = inputs.clone();
        expected.sort();

        rng.shuffle(inputs.as_mut_slice());

        let mut heap = LeonardoHeap::new();
        for input in &inputs {
            heap.push(input.clone());
        }

        heap.sort();

        assert_eq!(heap.data, expected);
    }

    #[test]
    fn test_iter() {
        let mut heap = LeonardoHeap::new();
        heap.push(4);
        heap.push(1);
        heap.push(2);
        heap.push(3);

        let mut heap_iter = heap.iter();

        let mut var = 4;
        assert_eq!(heap_iter.next(), Some(&var));
        var = 3;
        assert_eq!(heap_iter.next(), Some(&var));
        var = 2;
        assert_eq!(heap_iter.next(), Some(&var));
        var = 1;
        assert_eq!(heap_iter.next(), Some(&var));
    }
}
