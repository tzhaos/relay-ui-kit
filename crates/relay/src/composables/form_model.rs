use gpui::App;

use crate::{Binding, Form, Memo, Signal};

/// A builder for [`FormModel`].
pub struct FormModelBuilder {
    form: Form,
}

/// A higher-level form model that owns the underlying [`Form`] and common
/// derived UI state.
pub struct FormModel {
    form: Form,
    dirty: Memo<bool>,
    submitted: Signal<bool>,
}

/// Create a form model by registering all fields up front.
pub fn use_form_model(cx: &mut App, build: impl FnOnce(&mut FormModelBuilder, &App)) -> FormModel {
    let mut builder = FormModelBuilder::new();
    build(&mut builder, cx);
    builder.build(cx)
}

impl FormModelBuilder {
    /// Create an empty form model builder.
    pub fn new() -> Self {
        Self { form: Form::new() }
    }

    /// Register a field with a key and binding.
    pub fn field<T: PartialEq + Clone + 'static>(
        &mut self,
        key: &'static str,
        binding: Binding<T>,
        cx: &App,
    ) -> &mut Self {
        self.form.field(key, binding, cx);
        self
    }

    /// Finalize the form model.
    pub fn build(mut self, cx: &mut App) -> FormModel {
        let dirty = self.form.build_is_dirty(cx);
        let submitted = Signal::new(cx, false);

        FormModel {
            form: self.form,
            dirty,
            submitted,
        }
    }
}

impl Default for FormModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FormModel {
    /// Return the underlying raw form.
    pub fn form(&self) -> &Form {
        &self.form
    }

    /// Return a memo that is `true` when any field differs from the current baseline.
    pub fn dirty(&self) -> &Memo<bool> {
        &self.dirty
    }

    /// Return a signal that tracks whether the form has been submitted.
    pub fn submitted(&self) -> &Signal<bool> {
        &self.submitted
    }

    /// Return whether the form is currently dirty.
    pub fn is_dirty(&self, cx: &App) -> bool {
        self.dirty.get(cx)
    }

    /// Return whether the form has been submitted.
    pub fn is_submitted(&self, cx: &App) -> bool {
        self.submitted.get(cx)
    }

    /// Mark the form as submitted.
    pub fn mark_submitted(&self, cx: &mut App) {
        self.submitted.set(cx, true);
    }

    /// Clear the submitted flag.
    pub fn reset_submission(&self, cx: &mut App) {
        self.submitted.set(cx, false);
    }

    /// Reset all fields to the current baseline and clear the submitted flag.
    pub fn reset(&self, cx: &mut App) {
        self.form.reset(cx);
        self.reset_submission(cx);
    }

    /// Commit the current values as the new baseline and clear the submitted flag.
    pub fn commit(&mut self, cx: &mut App) {
        self.form.commit(cx);
        self.reset_submission(cx);
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use crate::{ReactiveAppExt, init};

    use super::*;

    #[test]
    fn form_model_tracks_dirty_and_submission() {
        let mut app = TestApp::new();
        let (binding, mut form) = app.update(|cx| {
            init(cx);
            let binding = cx.binding(10);
            let form = use_form_model(cx, |form, cx| {
                form.field("count", binding.clone(), cx);
            });
            (binding, form)
        });

        app.read(|cx| {
            assert!(!form.is_dirty(cx));
            assert!(!form.is_submitted(cx));
        });

        app.update(|cx| {
            binding.set(cx, 99);
            form.mark_submitted(cx);
        });

        app.read(|cx| {
            assert!(form.is_dirty(cx));
            assert!(form.is_submitted(cx));
        });

        app.update(|cx| form.reset(cx));

        app.read(|cx| {
            assert_eq!(binding.get(cx), 10);
            assert!(!form.is_dirty(cx));
            assert!(!form.is_submitted(cx));
        });

        app.update(|cx| {
            binding.set(cx, 20);
            form.commit(cx);
        });

        app.read(|cx| {
            assert_eq!(binding.get(cx), 20);
            assert!(!form.is_dirty(cx));
            assert!(!form.is_submitted(cx));
        });
    }
}
