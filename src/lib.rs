extern crate rand;
use rand::Rng;
use std::fmt::Debug;


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


fn leonardo(order: u32) -> usize {
    LEONARDO_NUMBERS[order as usize] as usize
}


fn _partition(len: usize) -> u64 {
    let mut orders = 0;
    let mut remaining = len;

    for order in (0..63).rev() {
        if leonardo(order) <= remaining {
            remaining -= leonardo(order);
            orders |= 1 << order;
        }
    }

    return orders;
}

#[derive(Clone, Debug)]
struct Layout {
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
        Layout {
            orders: _partition(size),
            size: size,
        }
    }

    pub fn push(&mut self) {
        self.size += 1;
        // TODO update incrementally rather than recalculating
        self.orders = _partition(self.size);
    }

    pub fn pop(&mut self) {
        self.size -= 1;
        // TODO update incrementally rather than recalculating
        self.orders = _partition(self.size);
        // TODO possibly return two exposed subheaps
    }

    #[inline]
    pub fn lowest_order(&self) -> Option<u32> {
        match self.orders.trailing_zeros() {
            64 => None,
            n => Some(n),
        }
    }

    pub fn iter(&self) -> SubheapIterator {
        SubheapIterator {
            orders: _partition(self.size),
            root: self.size - 1,
        }
    }
}

#[derive(Clone)]
struct SubheapIterator {
    orders: u64,
    root: usize,
}

impl Iterator for SubheapIterator {
    type Item = (usize, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.orders.count_ones() != 0 {
            let order = self.orders.trailing_zeros();
            let root = self.root;

            self.orders ^= 1 << order;
            if self.orders != 0 {
                self.root -= leonardo(order);
            }

            Some((root, order))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ones = self.orders.count_ones() as usize;
        (ones, Some(ones))
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

    fn sift_down(&mut self, root: usize, order: u32) {
        let mut pos = root;
        let mut order = order;

        while order > 1 {
            let fst_child_pos = pos - 1;
            let snd_child_pos = pos - 1 - leonardo(order - 2);

            // If the root is greater than both children we can stop
            if (self.data[pos] >= self.data[fst_child_pos]) &&
               (self.data[pos] >= self.data[snd_child_pos]) {
                break;
            }

            // Swap the parent with the largest child.  Prefer the furthest
            // child if both children are the same as doing so makes the array
            // slightly more sorted.
            if self.data[fst_child_pos] > self.data[snd_child_pos] {
                self.data.swap(pos, fst_child_pos);

                pos = fst_child_pos;
                order -= 2;
            } else {
                self.data.swap(pos, snd_child_pos);

                pos = snd_child_pos;
                order -= 1;
            }
        }
    }

    fn restring(&mut self, mut subheap_iter: SubheapIterator) {
        let (mut this_root, _) = subheap_iter.next().unwrap();

        for (next_root, next_order) in subheap_iter {
            if self.data[next_root] <= self.data[this_root] {
                break;
            }

            self.data.swap(next_root, this_root);

            self.sift_down(next_root, next_order);

            this_root = next_root;
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
        self.layout.push();
        // TODO need to copy layout to keep the borrow checker happy.  Figure
        // out a way to avoid this.
        let layout = self.layout.clone();

        // TODO pull from layout
        let new_root = self.data.len() - 1;

        self.sift_down(new_root, layout.lowest_order().unwrap());
        self.restring(layout.iter());
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

                let subheaps_from_fst = self.layout.iter();

                let mut subheaps_from_snd = self.layout.iter();
                // consume the first subheap
                subheaps_from_snd.next();

                self.restring(subheaps_from_snd);
                self.restring(subheaps_from_fst);

                return result;
            }
            None => {
                None
            }
        }
    }
}


#[test]
fn test_sift_down() {
    let mut heap = LeonardoHeap {
        data: vec![3, 2, 1],
        layout: Layout::new_from_len(3),
    };

    heap.sift_down(2, 2);
    assert_eq!(heap.data, vec![1, 2, 3]);

    let mut heap = LeonardoHeap {
        data: vec![3, 5, 4],
        layout: Layout::new_from_len(3),
    };

    heap.sift_down(2, 2);
    assert_eq!(heap.data, vec![3, 4, 5]);
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
