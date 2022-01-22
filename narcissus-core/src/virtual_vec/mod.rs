mod raw_virtual_vec;
mod virtual_deque;
mod virtual_vec;

pub use self::raw_virtual_vec::RawVirtualVec;
pub use self::virtual_deque::VirtualDeque;
pub use self::virtual_vec::VirtualVec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_vec() {
        let mut v = VirtualVec::new(2048);

        for i in 0..2048 {
            v.push(i);
        }

        for i in 0..2048 {
            assert_eq!(v[i], i);
        }

        for i in (0..2048).rev() {
            assert_eq!(v.pop(), Some(i))
        }

        assert_eq!(v.len(), 0);
    }

    #[test]
    #[should_panic]
    fn virtual_vec_too_large() {
        let mut v = VirtualVec::new(2048);
        for i in 0..2049 {
            v.push(i);
        }
    }

    #[test]
    fn virtual_deque() {
        let mut queue = VirtualDeque::new(2048);

        for _ in 0..2049 {
            for i in 0..2047 {
                queue.push_back(i);
            }

            for i in 0..2047 {
                assert!(queue.pop_front() == Some(i));
            }
        }

        assert_eq!(queue.len(), 0);
    }

    #[test]
    #[should_panic]
    fn virtual_deque_too_large() {
        let mut v = VirtualDeque::new(2048);
        for i in 0..2049 {
            v.push_back(i);
        }
    }
}
