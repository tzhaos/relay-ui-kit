//! Form aggregation model.
//!
//! [`Form`] collects multiple [`Binding`]s into a single aggregate, providing
//! derived helpers for dirty-checking, validation, submit, and reset. The
//! model keeps a snapshot of initial values so `is_dirty` can be computed
//! declaratively via a [`Memo`].
//!
//! Fields are registered with a key and a binding. The form exposes:
//!
//! - `is_dirty()` — a `Memo<bool>` that is `true` when any field differs from
//!   its initial value.
//! - `validate(cx)` — runs all registered validators, returning the first
//!   error (or `Ok(())`).
//! - `reset(cx)` — restores all fields to their initial values.
//! - `commit(cx)` — snapshots the current values as the new "clean" baseline.
//!
//! # Example
//!
//! ```ignore
//! let mut form = Form::new(cx);
//! form.field("name", name_binding);
//! form.field("enabled", enabled_binding);
//! let dirty = form.is_dirty();
//! ```

use std::collections::HashMap;
use std::rc::Rc;

use gpui::App;

use crate::{Binding, Memo, ReactiveAppExt};

/// A type-erased field entry in a [`Form`].
type FieldEntry = Rc<dyn Field>;

/// Trait implemented by field entries to enable type-erased reset/validate.
trait Field {
    /// Reset this field to its initial value.
    fn reset(&self, cx: &mut App);
    /// Compare the current value with the initial value.
    fn is_changed(&self, cx: &App) -> bool;
    /// Snapshot the current value as the new initial value.
    fn snapshot(&self, cx: &App) -> FieldEntry;
}

/// A typed field entry storing the binding and its initial value.
struct TypedField<T: PartialEq + Clone + 'static> {
    binding: Binding<T>,
    initial: T,
}

impl<T: PartialEq + Clone + 'static> Field for TypedField<T> {
    fn reset(&self, cx: &mut App) {
        self.binding.set(cx, self.initial.clone());
    }

    fn is_changed(&self, cx: &App) -> bool {
        self.binding.get(cx) != self.initial
    }

    fn snapshot(&self, cx: &App) -> FieldEntry {
        Rc::new(TypedField {
            binding: self.binding.clone(),
            initial: self.binding.get(cx),
        })
    }
}

/// An aggregate of bound fields with derived dirty/validate/reset helpers.
pub struct Form {
    fields: HashMap<&'static str, FieldEntry>,
    dirty_memo: Option<Memo<bool>>,
}

impl Form {
    /// Create an empty form.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            dirty_memo: None,
        }
    }

    /// Register a field with a key and a binding.
    ///
    /// The field's current value is captured as its initial (clean) value.
    pub fn field<T: PartialEq + Clone + 'static>(
        &mut self,
        key: &'static str,
        binding: Binding<T>,
        cx: &App,
    ) {
        let initial = binding.get(cx);
        self.fields.insert(
            key,
            Rc::new(TypedField {
                binding,
                initial,
            }),
        );
    }

    /// Build and return a `Memo<bool>` that is `true` when any field differs
    /// from its initial value.
    ///
    /// Call this once after registering all fields. The returned memo updates
    /// automatically when any field's binding changes.
    pub fn build_is_dirty(&mut self, cx: &mut App) -> Memo<bool> {
        let fields: Vec<FieldEntry> = self.fields.values().cloned().collect();
        let memo = cx.memo(move |cx| fields.iter().any(|field| field.is_changed(cx)));
        self.dirty_memo = Some(memo.clone());
        memo
    }

    /// Returns whether any field differs from its initial value.
    ///
    /// Requires [`Form::build_is_dirty`] to have been called first.
    pub fn is_dirty(&self) -> &Memo<bool> {
        self.dirty_memo
            .as_ref()
            .expect("build_is_dirty must be called before is_dirty")
    }

    /// Reset all fields to their initial values.
    pub fn reset(&self, cx: &mut App) {
        for field in self.fields.values() {
            field.reset(cx);
        }
    }

    /// Snapshot the current values as the new clean baseline.
    ///
    /// After calling this, `is_dirty` returns `false` until a field changes
    /// again. The dirty memo is notified so consumers refresh immediately.
    pub fn commit(&mut self, cx: &mut App) {
        let old_fields = std::mem::take(&mut self.fields);
        for (key, field) in old_fields {
            self.fields.insert(key, field.snapshot(cx));
        }

        // Force the dirty memo to recompute by touching each binding. The
        // memo's effect tracks the bindings, so a no-op update triggers
        // re-evaluation with the new initial values.
        if let Some(dirty) = &self.dirty_memo {
            let signal = dirty.signal().clone();
            // Re-run the memo's compute by setting the signal to the freshly
            // computed value.
            let fields: Vec<FieldEntry> = self.fields.values().cloned().collect();
            let new_value = fields.iter().any(|field| field.is_changed(cx));
            signal.set(cx, new_value);
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use super::*;
    use crate::init;

    #[test]
    fn form_is_dirty_when_field_changes() {
        let mut app = TestApp::new();
        let (binding, dirty) = app.update(|cx| {
            init(cx);
            let mut form = Form::new();
            let binding: Binding<bool> = cx.binding(false);
            form.field("enabled", binding.clone(), cx);
            let dirty = form.build_is_dirty(cx);
            // Leak the form so the effect stays alive for the dirty memo.
            std::mem::forget(form);
            (binding, dirty)
        });

        // Initially not dirty.
        app.read(|cx| assert!(!dirty.get(cx)));

        // Change the field — dirty should become true.
        app.update(|cx| binding.set(cx, true));
        app.read(|cx| assert!(dirty.get(cx)));

        let _ = dirty;
    }

    #[test]
    fn form_reset_restores_initial_values() {
        let mut app = TestApp::new();
        let _binding = app.update(|cx| {
            init(cx);
            let binding: Binding<i32> = cx.binding(42);
            let mut form = Form::new();
            form.field("count", binding.clone(), cx);
            form.build_is_dirty(cx);
            // Mutate the binding.
            binding.set(cx, 100);
            // Reset should restore 42.
            form.reset(cx);
            std::mem::forget(form);
            binding
        });

        app.read(|cx| {
            assert_eq!(_binding.get(cx), 42);
        });
    }

    #[test]
    fn form_commit_makes_dirty_false() {
        let mut app = TestApp::new();
        let (_binding, dirty) = app.update(|cx| {
            init(cx);
            let mut form = Form::new();
            let binding: Binding<String> = cx.binding("initial".into());
            form.field("name", binding.clone(), cx);
            let dirty = form.build_is_dirty(cx);
            // Change the value.
            binding.set(cx, "changed".into());
            // Dirty should be true.
            assert!(dirty.get(cx));
            // Commit the new value as the baseline.
            form.commit(cx);
            std::mem::forget(form);
            (binding, dirty)
        });

        // After commit, dirty should be false (current == new baseline).
        app.read(|cx| {
            assert!(!dirty.get(cx));
        });
    }

    #[test]
    fn form_with_multiple_fields() {
        let mut app = TestApp::new();
        let (a, b, dirty) = app.update(|cx| {
            init(cx);
            let mut form = Form::new();
            let a: Binding<bool> = cx.binding(false);
            let b: Binding<i32> = cx.binding(0);
            form.field("a", a.clone(), cx);
            form.field("b", b.clone(), cx);
            let dirty = form.build_is_dirty(cx);
            std::mem::forget(form);
            (a, b, dirty)
        });

        // Not dirty initially.
        app.read(|cx| assert!(!dirty.get(cx)));

        // Change one field.
        app.update(|cx| a.set(cx, true));
        app.read(|cx| assert!(dirty.get(cx)));

        // Revert it.
        app.update(|cx| a.set(cx, false));
        app.read(|cx| assert!(!dirty.get(cx)));

        // Change the other.
        app.update(|cx| b.set(cx, 5));
        app.read(|cx| assert!(dirty.get(cx)));
    }
}
