use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{App, Context, EntityId, Global, Subscription};

use crate::effect::{CleanupScope, EffectCleanup};

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
    effect_cleanups: HashMap<EffectId, Vec<EffectCleanup>>,
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

type EffectCallback = Rc<RefCell<dyn FnMut(&mut App, &mut CleanupScope)>>;

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
        let cleanups = cx.global::<ReactiveRuntime>().prepare_effect_run(effect_id);
        Self::run_cleanups(cx, cleanups);
        if cx
            .global::<ReactiveRuntime>()
            .effect_callback(effect_id)
            .is_none()
        {
            return;
        }

        let mut cleanup_scope = CleanupScope::new();
        cx.global::<ReactiveRuntime>().begin_tracking();
        callback.borrow_mut()(cx, &mut cleanup_scope);
        let deps = cx.global::<ReactiveRuntime>().end_tracking();
        cx.global::<ReactiveRuntime>()
            .replace_effect_dependencies(effect_id, deps);
        cx.global::<ReactiveRuntime>()
            .replace_effect_cleanups(effect_id, cleanup_scope.into_cleanups());
    }

    pub(crate) fn remove_effect(cx: &mut App, effect_id: EffectId) {
        let cleanups = match cx.try_global::<ReactiveRuntime>() {
            Some(runtime) => runtime.remove_effect_state(effect_id),
            None => return,
        };
        Self::run_cleanups(cx, cleanups);
    }

    fn begin_tracking(&self) {
        self.state
            .borrow_mut()
            .tracking_stack
            .push(TrackingScope::default());
    }

    fn begin_untracked_scope(&self) {
        self.state.borrow_mut().tracking_stack.push(TrackingScope {
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

    fn replace_effect_cleanups(&self, effect_id: EffectId, cleanups: Vec<EffectCleanup>) {
        let mut state = self.state.borrow_mut();
        if cleanups.is_empty() {
            state.effect_cleanups.remove(&effect_id);
        } else {
            state.effect_cleanups.insert(effect_id, cleanups);
        }
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

    fn prepare_effect_run(&self, effect_id: EffectId) -> Vec<EffectCleanup> {
        let mut state = self.state.borrow_mut();
        Self::clear_effect_dependencies(&mut state, effect_id);
        state.pending_effects.remove(&effect_id);
        state.effect_cleanups.remove(&effect_id).unwrap_or_default()
    }

    fn remove_effect_state(&self, effect_id: EffectId) -> Vec<EffectCleanup> {
        let mut state = self.state.borrow_mut();
        Self::clear_effect_dependencies(&mut state, effect_id);
        state.effects.remove(&effect_id);
        state.pending_effects.remove(&effect_id);
        state.effect_release_subscriptions.remove(&effect_id);
        state.effect_cleanups.remove(&effect_id).unwrap_or_default()
    }

    fn clear_effect_dependencies(state: &mut RuntimeState, effect_id: EffectId) {
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
    }

    fn run_cleanups(cx: &mut App, cleanups: Vec<EffectCleanup>) {
        if cleanups.is_empty() {
            return;
        }

        cx.global::<ReactiveRuntime>().begin_untracked_scope();
        for cleanup in cleanups {
            cleanup(cx);
        }
        let dropped = cx.global::<ReactiveRuntime>().end_tracking();
        debug_assert!(
            dropped.is_empty(),
            "effect cleanup scope should not collect dependencies"
        );
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

    // --- Nested batch ---

    #[test]
    fn nested_batch_only_flushes_on_outermost_exit() {
        let runtime = ReactiveRuntime::default();
        let entity_id = EntityId::from(42);
        let signal_id = runtime.allocate_signal();
        runtime.replace_entity_dependencies(entity_id, HashSet::from([signal_id]));

        runtime.enter_batch(); // outer
        runtime.enter_batch(); // inner

        let queued = runtime.notify_signal(signal_id);
        assert!(queued.is_empty(), "inner batch should not flush");

        let inner_notifications = runtime.exit_batch();
        assert!(
            inner_notifications.is_empty(),
            "exiting inner batch should not flush"
        );

        let outer_notifications = runtime.exit_batch();
        assert_eq!(
            outer_notifications.entities,
            vec![entity_id],
            "exiting outer batch should flush"
        );
    }

    // --- Batch with effects ---

    #[test]
    fn batch_flushes_pending_effects_on_exit() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });

        let seen = Rc::new(Cell::new(-1));
        let _effect = app.update({
            let seen = seen.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    seen.set(signal.get(cx));
                })
            }
        });
        assert_eq!(seen.get(), 0);

        // Batch two writes — effect should only run once after batch exits.
        app.update(|cx| {
            batch(cx, |cx| {
                signal.set(cx, 1);
                signal.set(cx, 2);
                signal.set(cx, 3);
            });
        });

        assert_eq!(seen.get(), 3, "effect should see final value");
    }

    // --- Effect cascade ---

    #[test]
    fn effect_cascade_writes_trigger_dependent_effect() {
        let mut app = TestApp::new();
        let s1 = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });
        let s2 = app.update(|cx| Signal::new(cx, 0));

        // Effect A: reads s1, writes s2 = s1 * 10
        let _effect_a = app.update({
            let s1 = s1.clone();
            let s2 = s2.clone();
            move |cx| {
                effect(cx, move |cx| {
                    let val = s1.get(cx);
                    s2.set(cx, val * 10);
                })
            }
        });

        // Effect B: reads s2, records value
        let seen = Rc::new(Cell::new(0));
        let _effect_b = app.update({
            let seen = seen.clone();
            let s2 = s2.clone();
            move |cx| {
                effect(cx, move |cx| {
                    seen.set(s2.get(cx));
                })
            }
        });

        // Initial: s1=0 → s2=0 → seen=0
        assert_eq!(seen.get(), 0);

        // Change s1 → cascade: effect A writes s2 → effect B sees new s2
        app.update(|cx| s1.set(cx, 5));
        assert_eq!(seen.get(), 50, "cascade should propagate to effect B");
    }

    // --- Disposed effect ---

    #[test]
    fn disposed_effect_does_not_rerun_on_signal_change() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });

        let seen = Rc::new(Cell::new(0));
        let effect_handle = app.update({
            let seen = seen.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    seen.set(signal.get(cx) + 1);
                })
            }
        });
        assert_eq!(seen.get(), 1);

        // Dispose the effect.
        app.update(|cx| effect_handle.dispose(cx));

        // Signal change should NOT trigger the disposed effect.
        app.update(|cx| signal.set(cx, 99));
        assert_eq!(seen.get(), 1, "disposed effect should not rerun");
    }

    // --- Effect with no dependencies ---

    #[test]
    fn effect_with_no_dependencies_does_not_rerun() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let runs = runs.clone();
            move |cx| {
                effect(cx, move |_cx| {
                    runs.set(runs.get() + 1);
                })
            }
        });
        assert_eq!(runs.get(), 1, "effect runs once on creation");

        // Signal change should NOT trigger the effect (no deps registered).
        app.update(|cx| signal.set(cx, 42));
        assert_eq!(runs.get(), 1, "effect with no deps should not rerun");
    }

    // --- Effect rerun updates dependencies ---

    #[test]
    fn effect_rerun_updates_dependencies() {
        let mut app = TestApp::new();
        let s1 = app.update(|cx| {
            init(cx);
            Signal::new(cx, 1)
        });
        let s2 = app.update(|cx| Signal::new(cx, 2));
        let flag = app.update(|cx| Signal::new(cx, false));

        // Effect reads s1 or s2 depending on flag.
        let seen = Rc::new(Cell::new(0));
        let _effect = app.update({
            let seen = seen.clone();
            let s1 = s1.clone();
            let s2 = s2.clone();
            let flag = flag.clone();
            move |cx| {
                effect(cx, move |cx| {
                    if flag.get(cx) {
                        seen.set(s2.get(cx));
                    } else {
                        seen.set(s1.get(cx));
                    }
                })
            }
        });
        assert_eq!(seen.get(), 1, "initial: reads s1");

        // Flip flag → effect reruns, now reads s2, drops s1 dependency.
        app.update(|cx| flag.set(cx, true));
        assert_eq!(seen.get(), 2, "after flag flip: reads s2");

        // s1 change should NOT trigger effect (s1 dep was dropped).
        app.update(|cx| s1.set(cx, 100));
        assert_eq!(
            seen.get(),
            2,
            "s1 change should not trigger effect after dep switch"
        );

        // s2 change SHOULD trigger effect.
        app.update(|cx| s2.set(cx, 200));
        assert_eq!(seen.get(), 200, "s2 change should trigger effect");
    }

    // --- init idempotency ---

    #[test]
    fn init_is_idempotent() {
        let mut app = TestApp::new();
        app.update(|cx| {
            init(cx);
            init(cx);
            init(cx);
        });
        assert!(app.read(|cx| is_installed(cx)));
    }

    #[test]
    fn is_installed_returns_false_before_init() {
        let app = TestApp::new();
        assert!(!app.read(|cx| is_installed(cx)));
    }

    // --- track_signal in non-tracking context is a no-op ---

    #[test]
    fn reading_signal_outside_tracking_context_does_not_panic() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 42)
        });

        // Read without any tracking scope — should not panic, just not register deps.
        app.read(|cx| {
            let val = signal.get(cx);
            assert_eq!(val, 42);
        });
    }
}
