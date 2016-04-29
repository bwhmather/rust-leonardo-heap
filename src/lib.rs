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


#[derive(Clone, Debug)]
struct SubHeap<'a, T: 'a> {
    data: &'a [T],
    order: u32,
}


impl<'a, T: Ord + Debug> SubHeap<'a, T> {
    fn new(data: &[T], order: u32) -> SubHeap<T> {
        assert_eq!(data.len(), leonardo(order));

        SubHeap {
            data: data,
            order: order,
        }
    }

    fn destructure(&self) -> (&T, Option<(SubHeap<T>, SubHeap<T>)>) {
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

    #[inline]
    fn value(&self) -> &T {
        self.data.last().unwrap()
    }

    #[inline]
    fn children(&self) -> Option<(SubHeap<T>, SubHeap<T>)> {
        let (_, children) = self.destructure();
        return children
    }
}


#[derive(Debug)]
struct SubHeapMut<'a, T: 'a> {
    data: &'a mut [T],
    order: u32,
}


impl<'a, T: Ord + Debug> SubHeapMut<'a, T> {
    fn new(data: &mut [T], order: u32) -> SubHeapMut<T> {
        assert_eq!(data.len(), leonardo(order));

        SubHeapMut {
            data: data,
            order: order,
        }
    }

    fn destructure(&self) -> (&T, Option<(SubHeap<T>, SubHeap<T>)>) {
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

    fn destructure_mut(&mut self) -> (&mut T, Option<(SubHeapMut<T>, SubHeapMut<T>)>) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (mut value, mut body) = self.data.split_last_mut().unwrap();
            let (mut snd_data, mut fst_data) = body.split_at_mut(leonardo(snd_order));

            (value, Some((
                SubHeapMut::new(fst_data, fst_order),
                SubHeapMut::new(snd_data, snd_order),
            )))
        } else {
            (self.value_mut(), None)
        }
    }

    fn into_components(self) -> (&'a mut T, Option<(SubHeapMut<'a, T>, SubHeapMut<'a, T>)>) {
        if self.order > 1 {
            let fst_order = self.order - 2;
            let snd_order = self.order - 1;

            let (mut value, mut body) = self.data.split_last_mut().unwrap();
            let (mut snd_data, mut fst_data) = body.split_at_mut(leonardo(snd_order));

            (value, Some((
                SubHeapMut::new(fst_data, fst_order),
                SubHeapMut::new(snd_data, snd_order),
            )))
        } else {
            (self.into_value(), None)
        }
    }

    #[inline]
    fn value(&self) -> &T {
        self.data.last().unwrap()
    }

    #[inline]
    fn value_mut(&mut self) -> &mut T {
        self.data.last_mut().unwrap()
    }

    #[inline]
    fn into_value(self) -> &'a mut T {
        self.data.last_mut().unwrap()
    }

    #[inline]
    fn children(&self) -> Option<(SubHeap<T>, SubHeap<T>)> {
        let (_, children) = self.destructure();
        return children
    }

    #[inline]
    fn children_mut(&mut self) -> Option<(SubHeapMut<T>, SubHeapMut<T>)> {
        let (_, children) = self.destructure_mut();
        return children
    }

    #[inline]
    fn into_children(self) -> Option<(SubHeapMut<'a, T>, SubHeapMut<'a, T>)> {
        let (_, children) = self.into_components();
        return children
    }
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

    pub fn iter<'a, T : Ord + Debug>(&self, data : &'a mut [T]) -> SubHeapIterMut<'a, T> {
        assert_eq!(data.len(), self.size);
        SubHeapIterMut {
            heap: data,
            orders: _partition(self.size),
        }
    }
}


#[derive(Debug)]
struct SubHeapIterMut<'a, T: 'a> {
    heap: &'a mut [T],
    orders: u64,
}


impl<'a, T : Ord + Debug> Iterator for SubHeapIterMut<'a, T>
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
            let heap_len = self.heap.len();

            // In order to avoid having more than one mutable reference to the
            // heap at any one time,we have to temporarily replace it in self
            // with a placeholder value.
            let mut heap_data = std::mem::replace(&mut self.heap, &mut []);

            // Split the heap into the part belonging to this sub-heap and all
            // of the rest
            let (mut rest_data, mut subheap_data) = heap_data.split_at_mut(
                heap_len - leonardo(order)
            );

            // Store what's left of the heap back in self
            self.heap = rest_data;

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
        // TODO need to copy layout to keep the borrow checker happy.  Figure
        // out a way to avoid this.
        let layout = self.layout.clone();

        // TODO skip directly to subheap
        let new_root = self.data.len() - 1;
        let new_order = layout.lowest_order().unwrap();

        sift_down(&mut SubHeapMut::new(
            &mut self.data[
                (1 + new_root - leonardo(new_order))..(1 + new_root)
            ],
            new_order
        ));

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
