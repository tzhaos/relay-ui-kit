//! List, tree, and retained-row primitives.
//!
//! This family covers the foundation for dense desktop collections:
//!
//! - [`ListItem`] for focusable/selectable rows;
//! - [`SectionedList`] for grouped content with headers;
//! - [`TreeView`] and [`TreeNode`] for hierarchical content;
//! - [`ForEach`] for retained keyed rendering where row identity matters.
//!
//! The product bar here is higher than "can render a row":
//!
//! - row identity must remain stable across host mutation;
//! - selected, focused, hovered, and disabled states must not fight each other;
//! - keyboard activation and navigation must stay predictable;
//! - filtered, empty, and mutating collections must not leave stale focus or
//!   stale child state behind.
//!
//! Higher-level app rows such as session, task, and navigation rows build on
//! this family from [`crate::patterns`] and [`crate::row`].

mod for_each;
mod item;
mod sectioned_list;
mod tree_view;

pub use for_each::ForEach;
pub use item::{ListItem, ListItemSpacing};
pub use sectioned_list::{SectionedList, SectionedListGroup};
pub use tree_view::{TreeNode, TreeView};
