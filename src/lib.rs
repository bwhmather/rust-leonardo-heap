
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


fn restring<'a, T : Ord + Debug>(mut subheap_iter: SubHeapIterMut<'a, T>) {
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

        sift_down(&mut self.iter_subheaps().next().unwrap());
        restring(self.iter_subheaps());
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.data.len())
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.layout.lowest_order() {
            Some(0) | Some(1) => {
                self.layout.pop();

                // TODO should always return Some(...) but might be worth
                // checking explicitly
                self.data.pop()
            }
            Some(order) => {
                self.layout.pop();

                // TODO should always return Some(...) but might be worth
                // checking explicitly
                let result = self.data.pop();

                if self.layout.lowest_order() == None {
                    return None; // TODO
                }

                {
                    let mut subheaps_from_snd = self.iter_subheaps();
                    // consume the first subheap
                    subheaps_from_snd.next();

                    restring(subheaps_from_snd);
                }

                {
                    let subheaps_from_fst = self.iter_subheaps();
                    restring(subheaps_from_fst);
                }

                return result;
            }
            None => {
                None
            }
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::Rng;

    use subheap::SubHeapMut;
    use {LeonardoHeap, restring, sift_down};

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
}
