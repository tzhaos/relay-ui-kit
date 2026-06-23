//! GPUI-native reactive state runtime.
//!
//!
//! # Example
//!
//! ```rust,no_run
//! use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
//! use relay::{ReactiveAppExt, ReactiveContextExt, Signal, init};
//!
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Counter {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         init(cx);
//!         Self {
//!             count: cx.signal(0),
//!         }
//!     }
//! }
//!
//! impl Render for Counter {
//!     fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         cx.tracked(|cx| div().child(self.count.get(cx).to_string()))
//!     }
//! }
//! ```

mod binding;
pub mod composables;
mod context;
mod effect;
mod form;
mod hooks;
mod keyed;
mod memo;
mod resource;
mod runtime;
mod selected_item;
mod selector;
mod signal;
mod signal_vec;
pub mod view;
mod window_ext;

pub use binding::Binding;
pub use composables::{
    FocusState, FormModel, FormModelBuilder, MultiSelectionModel, Mutation, MutationState,
    OrderedSelectionModel, ProjectedTreeNode, Query, SelectionModel, SelectionReconcilePolicy,
    SourceQuery, TreeProjection, use_error_query, use_focus_state, use_form_model,
    use_multi_selection_model, use_mutation, use_ordered_selection_model,
    use_ordered_selection_model_with_navigation, use_query, use_query_from_source, use_ready_query,
    use_selection_model, use_tree_projection,
};
pub use context::{ContextHandle, provide_context, use_context};
pub use effect::{
    CleanupScope, Effect, effect, effect_in, effect_in_with_cleanup, effect_with_cleanup,
};
pub use form::Form;
pub use hooks::{ReactiveAppExt, ReactiveContextExt, install};
pub use keyed::{KeyedSubView, KeyedSubViews};
pub use memo::Memo;
pub use relay_macros::Reactive;
pub use resource::{Resource, ResourceState};
pub use runtime::{EffectId, ReactiveRuntime, SignalId, batch, init, is_installed, track, untrack};
pub use selected_item::SelectedItemExt;
pub use selector::{SelectionNavigation, Selector};
pub use signal::{ReadSignal, Signal, WriteSignal};
pub use signal_vec::SignalVecExt;
pub use view::{FormBuilder, ReactiveView, StateScope, SubView};
pub use window_ext::WindowSignalExt;

/// Common relay imports for GPUI views.
pub mod prelude {
    pub use crate::view;
    pub use crate::{
        Binding, CleanupScope, ContextHandle, Effect, Form, FormBuilder, KeyedSubView,
        KeyedSubViews, Memo, MultiSelectionModel, Mutation, MutationState, OrderedSelectionModel,
        ProjectedTreeNode, Query, Reactive, ReactiveAppExt, ReactiveContextExt, ReactiveView,
        Resource, ResourceState, SelectedItemExt, SelectionModel, SelectionNavigation,
        SelectionReconcilePolicy, Selector, Signal, SignalVecExt, SourceQuery, StateScope, SubView,
        TreeProjection, WindowSignalExt, batch, effect, effect_in, effect_in_with_cleanup,
        effect_with_cleanup, init, install, provide_context, track, untrack, use_context,
        use_error_query, use_focus_state, use_form_model, use_multi_selection_model, use_mutation,
        use_ordered_selection_model, use_ordered_selection_model_with_navigation, use_query,
        use_query_from_source, use_ready_query, use_selection_model, use_tree_projection,
    };
}
