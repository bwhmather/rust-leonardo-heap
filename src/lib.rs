
mod leonardo;
mod subheap;
mod layout;

use std::fmt::Debug;

use leonardo::leonardo;
use subheap::SubHeapMut;
use layout::{SubHeapIterMut, Layout};


fn sift_down<T: Ord + Debug>(heap: &mut SubHeapMut<T>) {
    let (mut this_value, mut children) = heap.destructure_mut();

    loop {
        // No children.  We have reached the bottom of the heap
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

        // The heap property is satisfied.  No need to do anything else
        if &*this_value >= next_heap.value() {
            break;
        }

        // Seap the value of the parent with the value of the largest child.
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


fn restring<T : Ord + Debug>(mut subheap_iter: SubHeapIterMut<T>) {
    let mut this_subheap = subheap_iter.next().unwrap();

    for mut next_subheap in subheap_iter {
        if next_subheap.value() <= this_subheap.value() {
            break;
        }

        std::mem::swap(next_subheap.value_mut(), this_subheap.value_mut());

        sift_down(&mut next_subheap);

        this_subheap = next_subheap;
    }
}


fn balance_after_push<T: Ord + Debug>(heap_data: &mut [T], layout: &Layout) {
    sift_down(&mut layout.iter(heap_data).next().unwrap());
    restring(layout.iter(heap_data));
}


fn balance_after_pop<T: Ord + Debug>(heap_data: &mut [T], layout: &Layout) {
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
        // consume the first subheap
        subheaps_from_snd.next();

        restring(subheaps_from_snd);
    }

    {
        let subheaps_from_fst = layout.iter(heap_data);
        restring(subheaps_from_fst);
    }
}


#[derive(Debug)]
pub struct HeapIterMut<'a, T: 'a> {
    heap_data: &'a mut [T],
    layout: Layout,
}


impl<'a, T : Ord + Debug> Iterator for HeapIterMut<'a, T>
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        self.layout.pop();

        if self.heap_data.len() != 0 {
            // In order to avoid having more than one mutable reference to the
            // heap at any one time,we have to temporarily replace it in self
            // with a placeholder value.
            let mut heap_data = std::mem::replace(&mut self.heap_data, &mut []);

            let (result, rest_data) = heap_data.split_last_mut().unwrap();

            // Store what's left of the heap back in self
            self.heap_data = rest_data;

            balance_after_pop(self.heap_data, &self.layout);

            Some(result)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.heap_data.len(), Some(self.heap_data.len()))
    }
}


#[derive(Debug)]
pub struct LeonardoHeap<T> {
    data: Vec<T>,
    layout: Layout,
}


impl<T: Ord + Debug> LeonardoHeap<T> {
    pub fn new() -> Self {
        LeonardoHeap {
            data: vec![],
            layout: Layout::new(),
        }
    }

    fn iter_subheaps(&mut self) -> SubHeapIterMut<T> {
       self.layout.iter(&mut self.data)
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
        self.layout.push();

        balance_after_push(self.data.as_mut_slice(), &self.layout);
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.data.len())
    }

    pub fn pop(&mut self) -> Option<T> {
        let result = self.data.pop();
        self.layout.pop();

        balance_after_pop(self.data.as_mut_slice(), &self.layout);

        result
    }

    pub fn iter_mut(&mut self) -> HeapIterMut<T> {
        HeapIterMut {
            heap_data: self.data.as_mut_slice(),
            layout: self.layout.clone(),
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::Rng;

    use subheap::SubHeapMut;
    use {LeonardoHeap, HeapIterMut, restring, sift_down};

    #[test]
    fn test_sift_down() {
        let mut heap = vec![3, 2, 1];
        {
            let mut subheap = SubHeapMut::new(heap.as_mut_slice(), 2);
            sift_down(&mut subheap);
        }
        assert_eq!(heap, vec![1, 2, 3]);

        let mut heap = vec![3, 5, 4];
        {
            let mut subheap = SubHeapMut::new(heap.as_mut_slice(), 2);
            sift_down(&mut subheap);
        }
        assert_eq!(heap, vec![3, 4, 5]);
    }


    #[test]
    fn test_restring() {
        //let mut heap = LeonardoHeap {
        //    data: vec![4, 3],
        //};

        //heap.restring(1, BitSet::from_bytes(&[0b11000000]));

        //assert_eq!(heap.data, vec![3, 4]);
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
    fn test_sort_random() {
        let mut rng = rand::thread_rng();

        let mut inputs: Vec<i32> = Vec::new();
        for _ in 0..200 {
            inputs.push(rng.gen());
        }

        let mut heap = LeonardoHeap::new();
        for input in &inputs {
            heap.push(input.clone());
        }

        let mut outputs: Vec<i32> = Vec::new();
        loop {
            match heap.pop() {
                Some(output) => {
                    outputs.push(output);
                }
                None => {
                    break;
                }
            }
        }

        inputs.sort_by(|a, b| b.cmp(a));

        assert_eq!(outputs, inputs);
    }

    #[test]
    fn test_iter() {
        let mut heap = LeonardoHeap::new();
        heap.push(4);
        heap.push(1);
        heap.push(2);
        heap.push(3);

        let mut heap_iter = heap.iter_mut();

        let mut var = 4;
        assert_eq!(heap_iter.next(), Some(&mut var));
        var = 3;
        assert_eq!(heap_iter.next(), Some(&mut var));
        var = 2;
        assert_eq!(heap_iter.next(), Some(&mut var));
        var = 1;
        assert_eq!(heap_iter.next(), Some(&mut var));
    }
}
