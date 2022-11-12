use std::collections::VecDeque;

pub struct DelayQueue<T> {
    delay: usize,
    counter: usize,
    values: VecDeque<(usize, T)>,
}

impl<T> DelayQueue<T> {
    pub fn new(delay: usize) -> Self {
        Self {
            delay,
            counter: 0,
            values: VecDeque::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.values.push_back((self.counter + self.delay, value))
    }

    pub fn expire<F: FnMut(T)>(&mut self, mut f: F) {
        self.counter += 1;

        let to_remove = self
            .values
            .iter()
            .take_while(|(expiry, _)| *expiry == self.counter)
            .count();

        for _ in 0..to_remove {
            f(self.values.pop_front().unwrap().1);
        }
    }

    pub fn drain<R>(&mut self, range: R) -> std::collections::vec_deque::Drain<'_, (usize, T)>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.values.drain(range)
    }
}
