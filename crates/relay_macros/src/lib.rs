//! Procedural macros for relay.
//!
//! Currently provides `#[derive(Reactive)]` which transforms a plain struct
//! into one where every field is wrapped in `Signal<T>`, with generated
//! accessor methods. This enables field-level reactivity without the caller
//! manually creating individual signals.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, Ident, PathArguments, Type, parse_macro_input};

/// Derive reactive field-level accessors for a struct.
///
/// By default, each field is wrapped in `Signal<T>` internally. Mark a field
/// as `#[reactive(nested)]` to store its generated reactive wrapper instead,
/// allowing nested field-level tracking.
///
/// Generated methods:
/// - `ReactiveFoo::new(cx, field1, field2, ...)` — constructor
/// - `ReactiveFoo::from(cx, foo)` — constructor from a plain value
/// - `foo.snapshot(cx) -> Foo` — clone a plain snapshot
/// - `foo.set(cx, value)` — set all fields from a plain value
/// - `foo.get_name(cx) -> T` — read with dependency tracking (requires `T: Clone`)
/// - `foo.set_name(cx, value)` — write and notify (requires `T: PartialEq`)
/// - `foo.update_name(cx, |t| { ...; bool })` — in-place mutation
/// - `foo.signal_name() -> &Signal<T>` — direct signal access
/// - `foo.reactive_profile() -> &ReactiveProfile` — direct nested access
///
/// # Example
///
/// ```ignore
/// #[derive(Reactive)]
/// struct Counter {
///     count: i32,
///     label: String,
/// }
///
/// let counter = Counter::new(cx, 0, "Hello".into());
/// counter.set_count(cx, 5);
/// let label = counter.get_label(cx);
/// ```
#[proc_macro_derive(Reactive, attributes(reactive))]
pub fn derive_reactive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_reactive(input) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_reactive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let reactive_name = format_ident!("Reactive{}", name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Reactive derive only supports named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Reactive derive requires at least one field",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Reactive derive only supports structs",
            ));
        }
    };

    let reactive_fields = fields
        .iter()
        .map(ReactiveField::from_field)
        .collect::<syn::Result<Vec<_>>>()?;

    // Generate the wrapper struct with Signal<T> fields.
    let wrapper_fields = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        let storage_type = field.storage_type();
        quote! { #fname: #storage_type }
    });

    // Generate the constructor parameters and signal creation.
    let ctor_params = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        let ftype = &field.ty;
        quote! { #fname: #ftype }
    });
    let ctor_body = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        match &field.kind {
            ReactiveFieldKind::Signal => quote! { #fname: relay::Signal::new(cx, #fname) },
            ReactiveFieldKind::Nested { reactive_ty } => {
                quote! { #fname: #reactive_ty::from(cx, #fname) }
            }
        }
    });

    let destructure_fields = reactive_fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();
    let snapshot_fields = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        match &field.kind {
            ReactiveFieldKind::Signal => quote! { #fname: self.#fname.get(cx) },
            ReactiveFieldKind::Nested { .. } => quote! { #fname: self.#fname.snapshot(cx) },
        }
    });
    let set_fields = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        let set_method = format_ident!("set_{}", fname);
        quote! { self.#set_method(cx, #fname); }
    });
    let clone_fields = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        quote! { #fname: self.#fname.clone() }
    });
    let snapshot_bounds = reactive_fields.iter().map(|field| {
        let ftype = &field.ty;
        quote! { #ftype: Clone }
    });
    let set_bounds = reactive_fields.iter().map(|field| {
        let ftype = &field.ty;
        quote! { #ftype: PartialEq }
    });

    // Generate accessor methods for each field.
    let accessors = reactive_fields.iter().map(|field| {
        let fname = &field.name;
        let ftype = &field.ty;
        let get_method = format_ident!("get_{}", fname);
        let set_method = format_ident!("set_{}", fname);
        let update_method = format_ident!("update_{}", fname);
        let signal_method = format_ident!("signal_{}", fname);

        match &field.kind {
            ReactiveFieldKind::Signal => {
                quote! {
                    /// Read the field value with dependency tracking (requires `T: Clone`).
                    pub fn #get_method(&self, cx: &gpui::App) -> #ftype
                    where #ftype: Clone
                    {
                        self.#fname.get(cx)
                    }

                    /// Write the field value and notify dependents (requires `T: PartialEq`).
                    pub fn #set_method(&self, cx: &mut gpui::App, value: #ftype)
                    where #ftype: PartialEq
                    {
                        self.#fname.set(cx, value);
                    }

                    /// Mutate the field in place. The closure returns whether
                    /// dependents should be notified.
                    pub fn #update_method(&self, cx: &mut gpui::App, update: impl FnOnce(&mut #ftype) -> bool) {
                        self.#fname.update(cx, update);
                    }

                    /// Direct access to the underlying signal.
                    pub fn #signal_method(&self) -> &relay::Signal<#ftype> {
                        &self.#fname
                    }
                }
            }
            ReactiveFieldKind::Nested { reactive_ty } => {
                let reactive_method = format_ident!("reactive_{}", fname);

                quote! {
                    /// Read a plain snapshot of the nested field with dependency tracking.
                    pub fn #get_method(&self, cx: &gpui::App) -> #ftype
                    where #ftype: Clone
                    {
                        self.#fname.snapshot(cx)
                    }

                    /// Set all nested fields from a plain value.
                    pub fn #set_method(&self, cx: &mut gpui::App, value: #ftype) {
                        self.#fname.set(cx, value);
                    }

                    /// Mutate the nested value as a plain snapshot, then set all nested fields.
                    pub fn #update_method(&self, cx: &mut gpui::App, update: impl FnOnce(&mut #ftype) -> bool)
                    where #ftype: Clone
                    {
                        let mut value = self.#get_method(cx);
                        if update(&mut value) {
                            self.#set_method(cx, value);
                        }
                    }

                    /// Direct access to the nested reactive wrapper.
                    pub fn #reactive_method(&self) -> &#reactive_ty {
                        &self.#fname
                    }
                }
            }
        }
    });

    let expanded = quote! {
        /// Reactive wrapper generated by `#[derive(Reactive)]`.
        ///
        /// Each field is wrapped in `Signal<T>` for field-level dependency
        /// tracking — changing one field only notifies consumers of that
        /// specific field, not the entire struct.
        pub struct #reactive_name {
            #(#wrapper_fields,)*
        }

        impl #reactive_name {
            /// Create a new reactive struct, wrapping each field value in a
            /// `Signal<T>`.
            pub fn new(cx: &mut gpui::App, #(#ctor_params,)*) -> Self {
                Self {
                    #(#ctor_body,)*
                }
            }

            /// Create a reactive wrapper from a plain value.
            pub fn from(cx: &mut gpui::App, value: #name) -> Self {
                let #name { #(#destructure_fields,)* } = value;
                Self::new(cx, #(#destructure_fields,)*)
            }

            /// Clone a plain snapshot with dependency tracking.
            pub fn snapshot(&self, cx: &gpui::App) -> #name
            where
                #(#snapshot_bounds,)*
            {
                #name {
                    #(#snapshot_fields,)*
                }
            }

            /// Set all fields from a plain value.
            pub fn set(&self, cx: &mut gpui::App, value: #name)
            where
                #(#set_bounds,)*
            {
                let #name { #(#destructure_fields,)* } = value;
                #(#set_fields)*
            }

            #(#accessors)*
        }

        impl Clone for #reactive_name {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields,)*
                }
            }
        }
    };

    Ok(expanded)
}

struct ReactiveField {
    name: Ident,
    ty: Type,
    kind: ReactiveFieldKind,
}

enum ReactiveFieldKind {
    Signal,
    Nested { reactive_ty: TokenStream2 },
}

impl ReactiveField {
    fn from_field(field: &Field) -> syn::Result<Self> {
        let Some(name) = field.ident.clone() else {
            return Err(syn::Error::new_spanned(
                field,
                "Reactive derive only supports named fields",
            ));
        };

        let nested = is_nested_field(field)?;
        let kind = if nested {
            ReactiveFieldKind::Nested {
                reactive_ty: reactive_type_for(&field.ty)?,
            }
        } else {
            ReactiveFieldKind::Signal
        };

        Ok(Self {
            name,
            ty: field.ty.clone(),
            kind,
        })
    }

    fn storage_type(&self) -> TokenStream2 {
        match &self.kind {
            ReactiveFieldKind::Signal => {
                let ty = &self.ty;
                quote! { relay::Signal<#ty> }
            }
            ReactiveFieldKind::Nested { reactive_ty } => quote! { #reactive_ty },
        }
    }
}

fn is_nested_field(field: &Field) -> syn::Result<bool> {
    let mut nested = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("reactive") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("nested") {
                nested = true;
                Ok(())
            } else {
                Err(meta.error("unsupported reactive field attribute"))
            }
        })?;
    }

    Ok(nested)
}

fn reactive_type_for(ty: &Type) -> syn::Result<TokenStream2> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "#[reactive(nested)] only supports named struct field types",
        ));
    };

    if type_path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            ty,
            "#[reactive(nested)] does not support qualified paths",
        ));
    }

    let mut path = type_path.path.clone();
    let Some(last_segment) = path.segments.last_mut() else {
        return Err(syn::Error::new_spanned(
            ty,
            "#[reactive(nested)] requires a named field type",
        ));
    };

    if !matches!(last_segment.arguments, PathArguments::None) {
        return Err(syn::Error::new_spanned(
            ty,
            "#[reactive(nested)] does not support generic nested field types yet",
        ));
    }

    last_segment.ident = format_ident!("Reactive{}", last_segment.ident);
    Ok(quote! { #path })
}
