pub struct NonDet<T>(Vec<T>);

impl<T> NonDet<T> {
    pub fn new(xs: Vec<T>) -> Self {
        Self(xs)
    }

    pub fn singleton(x: T) -> Self {
        Self(vec![x])
    }

    pub fn and_then<F, U>(self, f: F) -> NonDet<U>
    where
        F: Fn(T) -> NonDet<U>,
    {
        let mut ys = Vec::new();
        for x in self.0 {
            for y in f(x).0 {
                ys.push(y);
            }
        }
        NonDet(ys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton() {
        let nd = NonDet::singleton(42);
        assert_eq!(nd.0, vec![42]);
    }

    #[test]
    fn test_new() {
        let nd = NonDet::new(vec![1, 2, 3]);
        assert_eq!(nd.0, vec![1, 2, 3]);
    }

    #[test]
    fn test_and_then() {
        let nd = NonDet::new(vec![1, 2, 3]);
        let nd2 = nd.and_then(|x| NonDet::new(vec![x * 2, x * 3]));
        assert_eq!(nd2.0, vec![2, 3, 4, 6, 6, 9]);
    }
}
