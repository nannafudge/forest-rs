pub mod errors;
pub mod traits;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

use core::ops::{
    BitOr, BitOrAssign,
    BitXor, BitXorAssign,
    BitAnd, BitAndAssign,
    AddAssign, SubAssign,
    Add, Sub
};
use crate::{
    NodeSide, NodeType, Direction,
    TreeWalker, TreeWalkerMut,
    require
};
use arborist_proc::{
    Length, interpolate, length_method
};

pub use self::{errors::*, traits::*};

/*################################
            Functions
################################*/

#[cfg(any(feature = "no_float", feature = "proptest"))]
pub(crate) mod compat {
    const USIZE_MIDPOINT: usize = (usize::BITS >> 1) as usize;

    #[inline(always)]
    pub(crate) fn log2_bin(length: &usize) -> usize {
        let mut mid: usize = USIZE_MIDPOINT;
        let mut cur: usize = USIZE_MIDPOINT;
    
        while mid > 1 {
            match length >> cur {
                1 => break,
                0 => cur -= { mid >>= 1; mid },
                _ => cur += { mid >>= 1; mid },
            }
        }
    
        cur
    }
}

#[inline(always)]
pub fn lsb(i: usize) -> usize {
    let _i: isize = i as isize;
    (_i & -_i) as usize
}

/*################################
            Index View
################################*/

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Length)]
#[length_method(self.lsb)]
pub struct IndexView {
    pub(crate) index: usize,
    pub(crate) lsb: usize
}

impl IndexView {
    fn new(index: usize) -> Self {
        IndexView {
            index: index,
            lsb: lsb(index)
        }
    }

    fn update(&mut self, new: usize) -> &mut Self {
        self.index = new;
        self.lsb = lsb(new);

        self
    }
}

impl PartialEq<usize> for IndexView {
    fn eq(&self, other: &usize) -> bool {
        &self.index == other
    }
}

/*################################
           Tree Walkers
################################*/

#[derive(Debug, Clone, PartialEq, Length)]
#[length_method(self.length)]
pub struct VirtualTreeView {
    length: usize,
    pub curr: IndexView
}

#[derive(Debug, Clone, PartialEq, Length)]
#[length_method(self.collection.length())]
pub struct StatefulTreeView<'a, C: ?Sized + Length> {
    collection: &'a C,
    pub curr: IndexView
}

#[derive(Debug, PartialEq, Length)]
#[length_method(self.collection.length())]
pub struct StatefulTreeViewMut<'a, C: ?Sized + Length> {
    collection: &'a mut C,
    pub curr: IndexView
}

impl VirtualTreeView {
    pub fn new(collection: &impl Length, index: usize) -> Result<Self, FenwickTreeError> {
        require!(index > 0, FenwickTreeError::OutOfBounds { index: 0, length: collection.length() });

        Ok(Self {
            length: collection.length(),
            curr: IndexView::new(index)
        })
    }
}

/*################################
            Node Impl
################################*/

impl From<usize> for NodeSide {
    fn from(index: usize) -> Self {
        let index_lsb: usize = lsb(index);
        match index >> index_lsb & 1 {
            0 => NodeSide::Left,
            1 => NodeSide::Right,
            _ => panic!("Invariant")
        }
    }
}

impl From<&IndexView> for NodeSide {
    fn from(view: &IndexView) -> Self {
        match view.index >> view.lsb & 1 {
            0 => NodeSide::Left,
            1 => NodeSide::Right,
            _ => panic!("Invariant")
        }
    }
}

impl From<usize> for NodeType {
    fn from(index: usize) -> Self {
        match index & 1 {
            0 => NodeType::Node,
            1 => NodeType::Leaf,
            _ => panic!("Invariant")
        }
    }
}

impl From<&IndexView> for NodeType {
    fn from(view: &IndexView) -> Self {
        match view.index & 1 {
            0 => NodeType::Node,
            1 => NodeType::Leaf,
            _ => panic!("Invariant")
        }
    }
}

/*################################
           Macro Impls
################################*/

impl_walker!{new(type = StatefulTreeView)}
impl_walker!{new(type = StatefulTreeViewMut: mut)}

impl_walker!{
    trait(
        type = VirtualTreeView,
        output = usize,
        return_wrapper = safe_tree_index!(virtual(self, #[ret]));
    )
}

impl_walker!{
    trait(
        type = StatefulTreeView,
        output = &'w C::Output,
        return_wrapper = safe_tree_index!(stateful(self, #[ret]));
    )
}

impl_walker!{
    trait(
        type = StatefulTreeViewMut,
        output = &'w C::Output,
        return_wrapper = safe_tree_index!(stateful(self, #[ret]));
    )
}

impl_walker!{
    trait_mut(
        type = StatefulTreeViewMut,
        output = &'w mut C::Output,
        return_wrapper = safe_tree_index!(stateful(self, #[ret], mut));
    )
}

impl_op!{BitOr<usize>, bitor, |, usize}
impl_op!{BitXor<usize>, bitxor, ^, usize}
impl_op!{BitAnd<usize>, bitand, &, usize}
impl_op!{Add<usize>, add, +, usize}
impl_op!{Sub<usize>, sub, -, usize}
impl_op_assign!{BitOrAssign<usize>, bitor_assign, |=, usize}
impl_op_assign!{BitXorAssign<usize>, bitxor_assign, ^=, usize}
impl_op_assign!{BitAndAssign<usize>, bitand_assign, &=, usize}
impl_op_assign!{AddAssign<usize>, add_assign, +=, usize}
impl_op_assign!{SubAssign<usize>, sub_assign, -=, usize}