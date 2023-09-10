macro_rules! impl_op {
    ($trait:ty, $fn:ident, $op:tt, $rhs:ty) => {
        impl $trait for IndexView {
            type Output = $rhs;

            fn $fn(self, rhs: $rhs) -> Self::Output {
                self.index $op rhs
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($trait:ty, $fn:ident, $op:tt, $rhs:ty) => {
        impl $trait for IndexView {
            fn $fn(&mut self, rhs: $rhs) {
                self.index $op rhs;
                self.lsb = lsb(self.index);
            }
        }
    };
}

macro_rules! safe_tree_index {
    // wrap_ret for VirtualTreeWalker
    (virtual($self:tt, $index:expr)) => {
        require!(
            $index > 0 && $index < $self.length(),
            FenwickTreeError::OutOfBounds{index: $index, length: $self.length()}
        );

        return Ok($index);
    };
    // wrap_ret for StatefulTreeWalkers
    (stateful($self:tt, $index:expr $(, $mut:tt)?)) => {
        require!(
            $index > 0 && $index < $self.collection.length(),
            FenwickTreeError::OutOfBounds{index: $index, length: $self.collection.length()}
        );

        return Ok(& $($mut)? $self.collection[$index]);
    };
}

/*################################
            Tree Walker
################################*/

macro_rules! impl_walker {
    (@up($self:ident)) => {
        ($self.curr | $self.curr.lsb << 1) ^ $self.curr.lsb
    };
    (@down($self:ident, $side:ident)) => {
        // Transition downward to next 'lsb namespace'
        match $side {
            NodeSide::Left => ($self.curr | $self.curr.lsb >> 1) ^ $self.curr.lsb,
            NodeSide::Right => ($self.curr | $self.curr.lsb) ^ $self.curr.lsb >> 1
        }
    };
    (@left($self:ident, $op:tt)) => {
        $self.curr.index $op ($self.curr.lsb << 1).min($self.curr.index)
    };
    (@right($self:ident, $op:tt)) => {
        $self.curr $op ($self.curr.lsb << 1)
    };
    (@peek($fn:ident, $($mut:ident,)? $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w $($mut)? self, direction: Direction) -> Result<$output, FenwickTreeError> {
            let index: usize = match direction {
                Direction::Up => {
                    impl_walker!{@up(self)}
                },
                Direction::Down(side) => {
                    impl_walker!{@down(self, side)}
                },
                Direction::Left => {
                    impl_walker!{@left(self, -)}
                },
                Direction::Right => {
                    impl_walker!{@right(self, +)}
                }
            };

            interpolate!{ret => {index}, $($wrap_ret)+}
        }
    };
    (@probe($fn:ident, $($mut:ident,)? $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w $($mut)? self, path: Self::Path) -> Result<$output, FenwickTreeError> {
            interpolate!(ret => {path}, $($wrap_ret)+);
        }
    };
    (@traverse($fn:ident, $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w mut self, direction: Direction) -> Result<$output, FenwickTreeError> {
            match direction {
                Direction::Up => {
                    self.curr.update(impl_walker!{@up(self)});
                },
                Direction::Down(side) => {
                    self.curr.update(impl_walker!{@down(self, side)});
                },
                Direction::Left => {
                    impl_walker!(@left(self, -=));
                },
                Direction::Right => {
                    impl_walker!(@right(self, +=));
                }
            };

            interpolate!(ret => {self.curr.index}, $($wrap_ret)+);
        }
    };
    (@seek($fn:ident, $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w mut self, path: Self::Path) -> Result<$output, FenwickTreeError> {
            require!(path > 0, FenwickTreeError::OutOfBounds{ index: 0, length: self.length() });

            self.curr.update(path);
            interpolate!(ret => {self.curr.index}, $($wrap_ret)+);
        }
    };
    (@current($fn:ident, $($mut:ident,)? $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w $($mut)? self) -> Result<$output, FenwickTreeError> {
            interpolate!(ret => {self.curr.index}, $($wrap_ret)+);
        }
    };
    (@sibling($fn:ident, $($mut:ident,)? $output:ty, $($wrap_ret:tt)+)) => {
        fn $fn(&'w $($mut)? self) -> Result<$output, FenwickTreeError> {
            let sibling: usize = self.curr.index ^ self.curr.lsb << 1;
            interpolate!(ret => {sibling}, $($wrap_ret)+);
        }
    };
    (@trait_body(output = $output:ty, return_wrapper = $($wrap_ret:tt)+)) => {
        type Path = usize;
        type Output = $output;
        type Error = FenwickTreeError;

        impl_walker!{@peek(peek, $output, $($wrap_ret)+)}
        impl_walker!{@probe(probe, $output, $($wrap_ret)+)}
        impl_walker!{@traverse(traverse, $output, $($wrap_ret)+)}
        impl_walker!{@seek(seek, $output, $($wrap_ret)+)}
        impl_walker!{@current(current, $output, $($wrap_ret)+)}
        impl_walker!{@sibling(sibling, $output, $($wrap_ret)+)}

        fn reset(&mut self) {
            self.curr.index = self.length();
        }

        fn type_(&self) -> NodeType {
            NodeType::from(&self.curr)
        }

        fn side(&self) -> NodeSide {
            NodeSide::from(&self.curr)
        }
    };
    (new(type = $target_type:ident $(: $mut:tt)?)) => {
        impl<'a, C> $target_type<'a, C> where
            C: ?Sized + IndexedCollection,
            C::Output: Sized
        {
            pub fn new(collection: &'a $($mut)? C, index: usize) -> Result<Self, FenwickTreeError> {
                require!(index > 0, FenwickTreeError::OutOfBounds{ index: 0, length: collection.length() });
        
                Ok(Self {
                    collection,
                    curr: IndexView::new(index)
                })
            }
        }
    };
    (trait(type = VirtualTreeView, output = $output:ty, return_wrapper = $($wrap_ret:tt)+)) => {
        impl<'w> TreeWalker<'w> for VirtualTreeView {
            impl_walker!{@trait_body(output = $output, return_wrapper = $($wrap_ret)+)}
        }
    };
    (trait(type = $target_type:ident, output = $output:ty, return_wrapper = $($wrap_ret:tt)+)) => {
        impl<'t, 'w, C> TreeWalker<'w> for $target_type<'t, C> where
            C: ?Sized + IndexedCollection,
            C::Output: Sized,
            't: 'w
        {
            impl_walker!{@trait_body(output = $output, return_wrapper = $($wrap_ret)+)}
        }
    };
    (trait_mut(type = $target_type:ident, output = $output:ty, return_wrapper = $($wrap_ret:tt)+)) => {
        impl<'t, 'w, C> TreeWalkerMut<'w> for $target_type<'t, C> where
            C: ?Sized + IndexedCollectionMut,
            C::Output: Sized,
            't: 'w
        {
            type OutputMut = $output;

            impl_walker!{@peek(peek_mut, mut, $output, $($wrap_ret)+)}
            impl_walker!{@probe(probe_mut, mut, $output, $($wrap_ret)+)}
            impl_walker!{@traverse(traverse_mut, $output, $($wrap_ret)+)}
            impl_walker!{@seek(seek_mut, $output, $($wrap_ret)+)}
            impl_walker!{@current(current_mut, mut, $output, $($wrap_ret)+)}
            impl_walker!{@sibling(sibling_mut, mut, $output, $($wrap_ret)+)}
        }
    };
}