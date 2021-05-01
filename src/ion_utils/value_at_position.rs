#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueAtPosition<T> {
    pos: usize,
    value: T,
}

impl<T> ValueAtPosition<T> {
    pub fn new(pos: usize, value: T) -> Self {
        Self { pos, value }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn take_value(self) -> T {
        self.value
    }

    pub fn map<Q>(self, map: impl FnOnce(T) -> Q) -> ValueAtPosition<Q> {
        ValueAtPosition {
            pos: self.pos,
            value: map(self.value),
        }
    }
}

impl<T> ValueAtPosition<Option<T>> {
    pub fn transpose(self) -> Option<ValueAtPosition<T>> {
        let Self { pos, value } = self;

        let value = value?;

        Some(ValueAtPosition { pos, value })
    }
}
