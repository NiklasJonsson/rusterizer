use std::mem::MaybeUninit;

pub struct FixedVec<T, const N: usize> {
    contents: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub fn new() -> Self {
        Self {
            // # Safety
            // From stdlib example:
            // The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.
            contents: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, val: T) {
        if self.len < N {
            let idx = self.len;
            self.contents[idx].write(val);
            self.len += 1;
        } else {
            panic!("This vector is full");
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            // Safety
            // This is safe as we know that this data as has been initialized in push()
            Some(unsafe { self.contents[idx].assume_init_ref() })
        } else {
            None
        }
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_ops() {
        let mut v = FixedVec::<i32, 3>::new();
        assert_eq!(v.len(), 0);
        v.push(10);
        assert_eq!(v.len(), 1);
        v.push(11);
        assert_eq!(v.len(), 2);
        v.push(12);
        assert_eq!(v.len(), 3);
        assert_eq!(*v.get(0).unwrap(), 10);
        assert_eq!(*v.get(1).unwrap(), 11);
        assert_eq!(*v.get(2).unwrap(), 12);
    }
}
