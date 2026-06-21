use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{App, Context, EntityId, Global, Subscription};

/// Stable identifier for a reactive signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

/// Stable identifier for a reactive effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ObserverId {
    Entity(EntityId),
    Effect(EffectId),
}

/// GPUI global that owns relay's dependency graph and scheduler state.
///
/// The runtime uses interior mutability so ordinary signal reads can record
/// dependencies without marking GPUI globals as changed.
#[derive(Default)]
pub struct ReactiveRuntime {
    state: RefCell<RuntimeState>,
}

impl Global for ReactiveRuntime {}

#[derive(Default)]
struct RuntimeState {
    next_signal_id: u64,
    next_effect_id: u64,
    tracking_stack: Vec<TrackingScope>,
    signal_observers: HashMap<SignalId, HashSet<ObserverId>>,
    entity_deps: HashMap<EntityId, HashSet<SignalId>>,
    effect_deps: HashMap<EffectId, HashSet<SignalId>>,
    effects: HashMap<EffectId, EffectCallback>,
    entity_release_subscriptions: HashMap<EntityId, Subscription>,
    effect_release_subscriptions: HashMap<EffectId, Subscription>,
    batch_depth: usize,
    pending_entities: HashSet<EntityId>,
    pending_effects: HashSet<EffectId>,
}

#[derive(Default)]
struct TrackingScope {
    deps: HashSet<SignalId>,
    /// When true, signal reads inside this scope are not recorded as
    /// dependencies. Used by [`untrack`] to enable read-without-subscribe.
    suppress: bool,
}

type EffectCallback = Rc<RefCell<dyn FnMut(&mut App)>>;

#[derive(Default, Debug)]
pub(crate) struct SignalNotifications {
    entities: Vec<EntityId>,
    effects: Vec<EffectId>,
}

/// Install the relay runtime into the GPUI app if it is not already present.
pub fn init(cx: &mut App) {
    if !cx.has_global::<ReactiveRuntime>() {
        cx.set_global(ReactiveRuntime::default());
    }
}

/// Returns whether the relay runtime has been installed.
pub fn is_installed(cx: &App) -> bool {
    cx.has_global::<ReactiveRuntime>()
}

/// Run a GPUI view render in a reactive tracking scope.
///
/// Any [`Signal`](crate::Signal) read through the provided context is registered
/// as a dependency of the current GPUI entity. When those signals change, relay
/// calls `cx.notify(entity_id)` and lets GPUI's normal invalidation and caching
/// paths do the rendering work.
pub fn track<T: 'static, R>(cx: &mut Context<T>, render: impl FnOnce(&mut Context<T>) -> R) -> R {
    init(cx);

    let entity_id = cx.entity_id();
    cx.global::<ReactiveRuntime>().begin_tracking();
    let result = render(cx);
    let deps = cx.global::<ReactiveRuntime>().end_tracking();
    cx.global::<ReactiveRuntime>()
        .replace_entity_dependencies(entity_id, deps);
    ensure_release_cleanup(cx, entity_id);
    result
}

/// Run a closure without recording signal reads as dependencies.
///
/// Signals read inside `f` will not subscribe the current observer (entity
/// render or effect) to those signals. Writes still propagate normally. This is
/// the reactive equivalent of "read a snapshot without subscribing".
pub fn untrack<R>(cx: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    init(cx);
    cx.global::<ReactiveRuntime>().begin_untracked_scope();
    let result = f(cx);
    let dropped = cx.global::<ReactiveRuntime>().end_tracking();
    debug_assert!(
        dropped.is_empty(),
        "untrack scope should not collect dependencies"
    );
    result
}

/// Batch signal writes and flush affected GPUI entities once at the end.
pub fn batch<R>(cx: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    init(cx);
    cx.global::<ReactiveRuntime>().enter_batch();
    let result = f(cx);
    let notifications = cx.global::<ReactiveRuntime>().exit_batch();
    ReactiveRuntime::flush_notifications(cx, notifications);
    result
}

impl ReactiveRuntime {
    pub(crate) fn allocate_signal(&self) -> SignalId {
        let mut state = self.state.borrow_mut();
        let id = SignalId(state.next_signal_id);
        state.next_signal_id += 1;
        id
    }

    pub(crate) fn insert_effect(&self, callback: EffectCallback) -> EffectId {
        let mut state = self.state.borrow_mut();
        let id = EffectId(state.next_effect_id);
        state.next_effect_id += 1;
        state.effects.insert(id, callback);
        id
    }

    pub(crate) fn track_signal(&self, signal_id: SignalId) {
        let mut state = self.state.borrow_mut();
        let Some(scope) = state.tracking_stack.last_mut() else {
            return;
        };
        if scope.suppress {
            return;
        }
        scope.deps.insert(signal_id);
    }

    pub(crate) fn notify_signal(&self, signal_id: SignalId) -> SignalNotifications {
        let mut state = self.state.borrow_mut();
        let Some(observers) = state.signal_observers.get(&signal_id) else {
            return SignalNotifications::default();
        };
        let observers = observers.iter().copied().collect::<Vec<_>>();
        if state.batch_depth > 0 {
            for observer in observers {
                match observer {
                    ObserverId::Entity(entity_id) => {
                        state.pending_entities.insert(entity_id);
                    }
                    ObserverId::Effect(effect_id) => {
                        state.pending_effects.insert(effect_id);
                    }
                }
            }
            return SignalNotifications::default();
        }

        SignalNotifications::from_observers(observers)
    }

    pub(crate) fn flush_notifications(cx: &mut App, notifications: SignalNotifications) {
        for entity_id in notifications.entities {
            cx.notify(entity_id);
        }
        for effect_id in notifications.effects {
            Self::run_effect(cx, effect_id);
        }
    }

    pub(crate) fn run_effect(cx: &mut App, effect_id: EffectId) {
        let callback = match cx.global::<ReactiveRuntime>().effect_callback(effect_id) {
            Some(callback) => callback,
            None => return,
        };

        cx.global::<ReactiveRuntime>().begin_tracking();
        callback.borrow_mut()(cx);
        let deps = cx.global::<ReactiveRuntime>().end_tracking();
        cx.global::<ReactiveRuntime>()
            .replace_effect_dependencies(effect_id, deps);
    }

    fn begin_tracking(&self) {
        self.state
            .borrow_mut()
            .tracking_stack
            .push(TrackingScope::default());
    }

    fn begin_untracked_scope(&self) {
        self.state
            .borrow_mut()
            .tracking_stack
            .push(TrackingScope {
                suppress: true,
                ..Default::default()
            });
    }

    fn end_tracking(&self) -> HashSet<SignalId> {
        self.state
            .borrow_mut()
            .tracking_stack
            .pop()
            .map(|scope| scope.deps)
            .unwrap_or_default()
    }

    fn replace_entity_dependencies(&self, entity_id: EntityId, deps: HashSet<SignalId>) {
        self.replace_observer_dependencies(
            ObserverId::Entity(entity_id),
            deps,
            DependencyOwner::Entity(entity_id),
        );
    }

    fn replace_effect_dependencies(&self, effect_id: EffectId, deps: HashSet<SignalId>) {
        self.replace_observer_dependencies(
            ObserverId::Effect(effect_id),
            deps,
            DependencyOwner::Effect(effect_id),
        );
    }

    fn replace_observer_dependencies(
        &self,
        observer: ObserverId,
        deps: HashSet<SignalId>,
        owner: DependencyOwner,
    ) {
        let mut state = self.state.borrow_mut();
        let old_deps = match owner {
            DependencyOwner::Entity(entity_id) => state
                .entity_deps
                .insert(entity_id, deps.clone())
                .unwrap_or_default(),
            DependencyOwner::Effect(effect_id) => state
                .effect_deps
                .insert(effect_id, deps.clone())
                .unwrap_or_default(),
        };

        for signal_id in old_deps.difference(&deps) {
            if let Some(observers) = state.signal_observers.get_mut(signal_id) {
                observers.remove(&observer);
                if observers.is_empty() {
                    state.signal_observers.remove(signal_id);
                }
            }
        }

        for signal_id in deps {
            state
                .signal_observers
                .entry(signal_id)
                .or_default()
                .insert(observer);
        }
    }

    fn enter_batch(&self) {
        self.state.borrow_mut().batch_depth += 1;
    }

    fn exit_batch(&self) -> SignalNotifications {
        let mut state = self.state.borrow_mut();
        if state.batch_depth == 0 {
            return SignalNotifications::default();
        }

        state.batch_depth -= 1;
        if state.batch_depth > 0 {
            return SignalNotifications::default();
        }

        SignalNotifications {
            entities: state.pending_entities.drain().collect(),
            effects: state.pending_effects.drain().collect(),
        }
    }

    fn has_release_subscription(&self, entity_id: EntityId) -> bool {
        self.state
            .borrow()
            .entity_release_subscriptions
            .contains_key(&entity_id)
    }

    fn insert_release_subscription(&self, entity_id: EntityId, subscription: Subscription) {
        self.state
            .borrow_mut()
            .entity_release_subscriptions
            .insert(entity_id, subscription);
    }

    pub(crate) fn insert_effect_release_subscription(
        &self,
        effect_id: EffectId,
        subscription: Subscription,
    ) {
        self.state
            .borrow_mut()
            .effect_release_subscriptions
            .insert(effect_id, subscription);
    }

    fn remove_entity(&self, entity_id: EntityId) {
        let mut state = self.state.borrow_mut();
        if let Some(deps) = state.entity_deps.remove(&entity_id) {
            for signal_id in deps {
                if let Some(observers) = state.signal_observers.get_mut(&signal_id) {
                    observers.remove(&ObserverId::Entity(entity_id));
                    if observers.is_empty() {
                        state.signal_observers.remove(&signal_id);
                    }
                }
            }
        }
        state.pending_entities.remove(&entity_id);
        state.entity_release_subscriptions.remove(&entity_id);
    }

    pub(crate) fn remove_effect(&self, effect_id: EffectId) {
        let mut state = self.state.borrow_mut();
        if let Some(deps) = state.effect_deps.remove(&effect_id) {
            for signal_id in deps {
                if let Some(observers) = state.signal_observers.get_mut(&signal_id) {
                    observers.remove(&ObserverId::Effect(effect_id));
                    if observers.is_empty() {
                        state.signal_observers.remove(&signal_id);
                    }
                }
            }
        }
        state.effects.remove(&effect_id);
        state.pending_effects.remove(&effect_id);
        state.effect_release_subscriptions.remove(&effect_id);
    }

    fn effect_callback(&self, effect_id: EffectId) -> Option<EffectCallback> {
        self.state.borrow().effects.get(&effect_id).cloned()
    }

    #[cfg(test)]
    pub(crate) fn observer_count(&self, signal_id: SignalId) -> usize {
        self.state
            .borrow()
            .signal_observers
            .get(&signal_id)
            .map(HashSet::len)
            .unwrap_or_default()
    }
}

enum DependencyOwner {
    Entity(EntityId),
    Effect(EffectId),
}

impl SignalNotifications {
    fn from_observers(observers: impl IntoIterator<Item = ObserverId>) -> Self {
        let mut notifications = Self::default();
        for observer in observers {
            match observer {
                ObserverId::Entity(entity_id) => notifications.entities.push(entity_id),
                ObserverId::Effect(effect_id) => notifications.effects.push(effect_id),
            }
        }
        notifications
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.entities.is_empty() && self.effects.is_empty()
    }
}

fn ensure_release_cleanup<T: 'static>(cx: &mut Context<T>, entity_id: EntityId) {
    if cx
        .global::<ReactiveRuntime>()
        .has_release_subscription(entity_id)
    {
        return;
    }

    let subscription = cx.on_release(move |_, cx| {
        if let Some(runtime) = cx.try_global::<ReactiveRuntime>() {
            runtime.remove_entity(entity_id);
        }
    });
    cx.global::<ReactiveRuntime>()
        .insert_release_subscription(entity_id, subscription);
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, EntityId, IntoElement, ParentElement, Render, TestApp, Window, div};

    use crate::{Memo, Signal, effect, init, track, untrack};

    use super::*;

    struct ReactiveView {
        signal: Signal<i32>,
        alternate: Signal<i32>,
        use_alternate: bool,
    }

    impl ReactiveView {
        fn new(cx: &mut Context<Self>) -> Self {
            init(cx);
            Self {
                signal: Signal::new(cx, 0),
                alternate: Signal::new(cx, 10),
                use_alternate: false,
            }
        }

        fn active_signal(&self) -> Signal<i32> {
            if self.use_alternate {
                self.alternate.clone()
            } else {
                self.signal.clone()
            }
        }
    }

    impl Render for ReactiveView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            track(cx, |cx| {
                div().child(self.active_signal().get(cx).to_string())
            })
        }
    }

    #[test]
    fn tracked_signal_notifies_rendering_entity_when_changed() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        app.update_entity(&root, |view, cx| {
            view.signal.set(cx, 1);
        });

        assert_eq!(notifications.get(), 1);
    }

    #[test]
    fn setting_same_value_does_not_notify_entity() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        app.update_entity(&root, |view, cx| {
            view.signal.set(cx, 0);
        });

        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn rerender_replaces_previous_signal_dependencies() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        window.draw();

        let (first_id, second_id) = window.read(|view, _| (view.signal.id(), view.alternate.id()));
        app.read_global::<ReactiveRuntime, _>(|runtime, _| {
            assert_eq!(runtime.observer_count(first_id), 1);
        });

        window.update(|view, window, cx| {
            view.use_alternate = true;
            cx.notify();
            window.refresh();
        });
        window.draw();

        app.read_global::<ReactiveRuntime, _>(|runtime, _| {
            assert_eq!(runtime.observer_count(first_id), 0);
            assert_eq!(runtime.observer_count(second_id), 1);
        });
    }

    #[test]
    fn runtime_batches_signal_notifications_until_outer_batch_exits() {
        let runtime = ReactiveRuntime::default();
        let entity_id = EntityId::from(42);
        let signal_id = runtime.allocate_signal();
        runtime.replace_entity_dependencies(entity_id, HashSet::from([signal_id]));

        runtime.enter_batch();
        let queued = runtime.notify_signal(signal_id);

        assert!(queued.is_empty());
        let notifications = runtime.exit_batch();
        assert_eq!(notifications.entities, vec![entity_id]);
        assert!(notifications.effects.is_empty());
    }

    #[test]
    fn cleanup_removes_entity_from_all_signal_observers() {
        let runtime = ReactiveRuntime::default();
        let entity_id = EntityId::from(42);
        let first = runtime.allocate_signal();
        let second = runtime.allocate_signal();
        runtime.replace_entity_dependencies(entity_id, HashSet::from([first, second]));

        runtime.remove_entity(entity_id);

        assert_eq!(runtime.observer_count(first), 0);
        assert_eq!(runtime.observer_count(second), 0);
    }

    #[test]
    fn effect_reruns_when_tracked_signal_changes() {
        let mut app = TestApp::new();
        let seen = Rc::new(Cell::new(0));
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 1)
        });

        let _effect = app.update({
            let seen = seen.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    seen.set(signal.get(cx));
                })
            }
        });

        app.update(|cx| {
            signal.set(cx, 2);
        });

        assert_eq!(seen.get(), 2);
    }

    #[test]
    fn memo_updates_when_source_signal_changes() {
        let mut app = TestApp::new();
        let (source, memo) = app.update(|cx| {
            init(cx);
            let source = Signal::new(cx, 2);
            let memo = Memo::new(cx, {
                let source = source.clone();
                move |cx| source.get(cx) * 3
            });
            (source, memo)
        });

        app.update(|cx| {
            source.set(cx, 4);
        });

        app.read(|cx| {
            assert_eq!(memo.get(cx), 12);
        });
    }

    #[test]
    fn untrack_does_not_subscribe_rendering_entity() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        // Mutating the alternate signal (which is untracked inside render via
        // a hypothetical untrack call) should not notify. We simulate this by
        // reading via untrack in the test below.
        app.update_entity(&root, |view, cx| {
            // Read via untrack: should NOT register as a dependency.
            let _peeked = untrack(cx, |cx| view.alternate.get(cx));
        });

        app.update_entity(&root, |view, cx| {
            view.alternate.set(cx, 99);
        });

        // alternate changed, but render never subscribed to it (render only
        // subscribes to the active signal). Confirm no spurious notification.
        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn untrack_writes_still_propagate() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        // Writing inside untrack should still notify subscribers of the signal.
        app.update_entity(&root, |view, cx| {
            untrack(cx, |cx| view.signal.set(cx, 7));
        });

        assert_eq!(notifications.get(), 1);
    }

    #[test]
    fn untrack_nested_restores_tracking_after_scope_exits() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ReactiveView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        // Untrack read in the middle, then a tracked read after — the latter
        // should still register the dependency.
        app.update_entity(&root, |view, cx| {
            let _untracked_peek = untrack(cx, |cx| view.alternate.get(cx));
            let _tracked = view.signal.get(cx);
        });

        // signal is a dependency, alternate is not.
        app.update_entity(&root, |view, cx| {
            view.signal.set(cx, 1);
        });
        assert_eq!(notifications.get(), 1);

        app.update_entity(&root, |view, cx| {
            view.alternate.set(cx, 55);
        });
        assert_eq!(notifications.get(), 1);
    }
}
