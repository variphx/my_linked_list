use std::{marker::PhantomData, ptr::NonNull};

struct Node<T> {
    key: T,
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    fn new(key: T) -> Node<T> {
        Node {
            key,
            prev: None,
            next: None,
        }
    }
}

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

pub struct Iter<'a, T> {
    head: Option<NonNull<Node<T>>>,
    _tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let node = unsafe { self.head.unwrap_unchecked() };

        self.head = unsafe { node.as_ref() }.next;
        self.len -= 1;

        Some(&unsafe { node.as_ref() }.key)
    }
}

pub struct IterMut<'a, T> {
    head: Option<NonNull<Node<T>>>,
    _tail: Option<NonNull<Node<T>>>,
    len: usize,
    marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let mut node = unsafe { self.head.unwrap_unchecked() };

        self.head = unsafe { node.as_ref() }.next;
        self.len -= 1;

        Some(&mut unsafe { node.as_mut() }.key)
    }
}

impl<T> LinkedList<T> {
    pub const fn new() -> LinkedList<T> {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.head,
            _tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            head: self.head,
            _tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    pub fn push_front(&mut self, key: T) {
        if self.head.is_none() {
            let node = NonNull::new(Box::into_raw(Box::new(Node::new(key))));
            self.head = node;
            self.tail = node;
            self.len = 1;
            return;
        }

        let node = NonNull::new(Box::into_raw(Box::new(Node {
            key,
            prev: None,
            next: self.head,
        })));
        unsafe { self.head.unwrap_unchecked().as_mut() }.prev = node;
        self.head = node;
        self.len += 1;
    }

    pub fn push_back(&mut self, key: T) {
        if self.tail.is_none() {
            let node = NonNull::new(Box::into_raw(Box::new(Node::new(key))));
            self.head = node;
            self.tail = node;
            self.len = 1;
            return;
        }

        let node = NonNull::new(Box::into_raw(Box::new(Node {
            key,
            prev: self.tail,
            next: None,
        })));
        unsafe { self.tail.unwrap_unchecked().as_mut() }.next = node;
        self.tail = node;
        self.len += 1;
    }

    pub fn push_at(&mut self, at: usize, key: T) {
        assert!(
            at <= self.len,
            "Index out of bound: len is `{}` but index is `{}`",
            self.len,
            at
        );

        if at == 0 {
            return self.push_front(key);
        }

        if at == self.len() {
            return self.push_back(key);
        }

        let mut prev_node = unsafe { self.head.unwrap_unchecked() };
        let mut post_node = unsafe { prev_node.as_ref().next.unwrap_unchecked() };

        for _ in 1..at {
            prev_node = post_node;
            post_node = unsafe { post_node.as_ref().next.unwrap_unchecked() };
        }

        let node = NonNull::new(Box::into_raw(Box::new(Node {
            key,
            prev: Some(prev_node),
            next: Some(post_node),
        })));

        unsafe { prev_node.as_mut() }.next = node;
        unsafe { post_node.as_mut() }.prev = node;

        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.tail.is_none() {
            return None;
        }

        let tail = unsafe { self.tail.unwrap_unchecked() };

        let node = unsafe { Box::from_raw(tail.as_ptr()) };

        if self.len == 1 {
            self.head = None;
            self.tail = None;
            self.len = 0;
        } else {
            self.tail = unsafe { tail.as_ref() }.prev;
            self.tail = self.tail.map(|mut tail| {
                unsafe { tail.as_mut() }.next = None;
                tail
            });
            self.len -= 1;
        }

        Some(node.key)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        let head = unsafe { self.head.unwrap_unchecked() };

        let node = unsafe { Box::from_raw(head.as_ptr()) };

        if self.len == 1 {
            self.head = None;
            self.tail = None;
            self.len = 0;
        } else {
            self.head = unsafe { head.as_ref() }.next;
            self.len -= 1;
        }

        Some(node.key)
    }

    pub fn pop_at(&mut self, at: usize) -> Option<T> {
        assert!(
            at < self.len,
            "Index out of bound: len is `{}` but index is `{}`",
            self.len,
            at
        );

        if at == 0 {
            return self.pop_front();
        }

        if at == self.len - 1 {
            return self.pop_back();
        }

        let mut prev_node = unsafe { self.head.unwrap_unchecked() };
        let mut post_node = unsafe {
            prev_node
                .as_ref()
                .next
                .unwrap_unchecked()
                .as_ref()
                .next
                .unwrap_unchecked()
        };

        for _ in 1..at {
            prev_node = unsafe { prev_node.as_ref().next.unwrap_unchecked() };
            post_node = unsafe { post_node.as_ref().next.unwrap_unchecked() };
        }

        let node = unsafe { prev_node.as_ref().next.unwrap_unchecked() };
        let node = unsafe { Box::from_raw(node.as_ptr()) };

        unsafe { prev_node.as_mut() }.next = Some(post_node);
        unsafe { post_node.as_mut() }.prev = Some(prev_node);

        self.len -= 1;

        Some(node.key)
    }

    pub fn contains(&self, key: &T) -> bool
    where
        T: PartialEq<T>,
    {
        for x in self.iter() {
            if *x == *key {
                return true;
            }
        }

        false
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        if self.head.is_none() {
            return;
        }

        while !self.is_empty() {
            let to_free = unsafe { self.head.unwrap_unchecked() };
            self.head = unsafe { to_free.as_ref() }.next;
            self.len -= 1;
            let _ = unsafe { Box::from_raw(to_free.as_ptr()) };
        }
    }
}
