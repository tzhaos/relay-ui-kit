use std::{cell::Cell, rc::Rc};

use gpui::TestApp;
use relay::{Reactive, effect, init};

#[derive(Clone, Debug, PartialEq, Eq, Reactive)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Reactive)]
struct User {
    name: String,
    #[reactive(nested)]
    address: Address,
}

#[test]
fn reactive_derive_snapshots_and_sets_nested_fields() {
    let mut app = TestApp::new();
    let user = app.update(|cx| {
        init(cx);
        ReactiveUser::from(
            cx,
            User {
                name: "Ada".into(),
                address: Address {
                    city: "London".into(),
                    zip: "SW1A".into(),
                },
            },
        )
    });

    app.read(|cx| {
        assert_eq!(
            user.snapshot(cx),
            User {
                name: "Ada".into(),
                address: Address {
                    city: "London".into(),
                    zip: "SW1A".into(),
                },
            }
        );
    });

    app.update(|cx| {
        user.set_address(
            cx,
            Address {
                city: "Paris".into(),
                zip: "75001".into(),
            },
        );
    });

    app.read(|cx| {
        assert_eq!(user.reactive_address().get_city(cx), "Paris");
        assert_eq!(user.get_address(cx).zip, "75001");
    });
}

#[test]
fn reactive_derive_tracks_nested_fields_independently() {
    let mut app = TestApp::new();
    let user = app.update(|cx| {
        init(cx);
        ReactiveUser::from(
            cx,
            User {
                name: "Ada".into(),
                address: Address {
                    city: "London".into(),
                    zip: "SW1A".into(),
                },
            },
        )
    });

    let runs = Rc::new(Cell::new(0));
    let _effect = app.update({
        let runs = runs.clone();
        let user = user.clone();
        move |cx| {
            effect(cx, move |cx| {
                let _ = user.reactive_address().get_city(cx);
                runs.set(runs.get() + 1);
            })
        }
    });

    assert_eq!(runs.get(), 1);

    app.update(|cx| user.set_name(cx, "Grace".into()));
    assert_eq!(runs.get(), 1);

    app.update(|cx| user.reactive_address().set_zip(cx, "10001".into()));
    assert_eq!(runs.get(), 1);

    app.update(|cx| user.reactive_address().set_city(cx, "New York".into()));
    assert_eq!(runs.get(), 2);
}
