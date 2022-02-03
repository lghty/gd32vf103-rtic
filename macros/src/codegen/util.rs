use core::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use rtic_syntax::{ast::App, Context};
use syn::{Attribute, Ident, LitInt, PatType};

use crate::check::Extra;

const RTIC_INTERNAL: &str = "__rtic_internal";

/// Turns `capacity` into an unsuffixed integer literal
pub fn capacity_literal(capacity: usize) -> LitInt {
    LitInt::new(&capacity.to_string(), Span::call_site())
}

/// Identifier for the free queue
pub fn fq_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_FQ", task.to_string()))
}

/// Generates a `Mutex` implementation
pub fn impl_mutex(
    _extra: &Extra,
    cfgs: &[Attribute],
    resources_prefix: bool,
    name: &Ident,
    ty: TokenStream2,
    level: u8,
    ptr: TokenStream2,
) -> TokenStream2 {
    let (path, priority) = if resources_prefix {
        (quote!(shared_resources::#name), quote!(self.priority()))
    } else {
        (quote!(#name), quote!(self.priority))
    };

    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                unsafe {
                    rtic::export::lock(
                        #ptr,
                        #level,
                        #priority,
                        f,
                    )
                }
            }
        }
    )
}

/// Generates an identifier for the `INPUTS` buffer (`spawn` & `schedule` API)
pub fn inputs_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_INPUTS", task))
}

/// Generates an identifier for the `INSTANTS` buffer (`schedule` API)
pub fn monotonic_instants_ident(task: &Ident, monotonic: &Ident) -> Ident {
    mark_internal_name(&format!("{}_{}_INSTANTS", task, monotonic))
}

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("interrupt", span)
}

pub fn timer_queue_marker_ident() -> Ident {
    mark_internal_name("TIMER_QUEUE_MARKER")
}

/// Whether `name` is an exception with configurable priority
pub fn is_exception(name: &Ident) -> bool {
    let s = name.to_string();

    matches!(
        &*s,
        "MemoryManagement"
            | "BusFault"
            | "UsageFault"
            | "SecureFault"
            | "SVCall"
            | "DebugMonitor"
            | "PendSV"
            | "SysTick"
    )
}

/// Mark a name as internal
pub fn mark_internal_name(name: &str) -> Ident {
    Ident::new(&format!("{}_{}", RTIC_INTERNAL, name), Span::call_site())
}

/// Generate an internal identifier for monotonics
pub fn internal_monotonics_ident(task: &Ident, monotonic: &Ident, ident_name: &str) -> Ident {
    mark_internal_name(&format!(
        "{}_{}_{}",
        task.to_string(),
        monotonic.to_string(),
        ident_name,
    ))
}

/// Generate an internal identifier for tasks
pub fn internal_task_ident(task: &Ident, ident_name: &str) -> Ident {
    mark_internal_name(&format!("{}_{}", task.to_string(), ident_name))
}

fn link_section_index() -> usize {
    static INDEX: AtomicUsize = AtomicUsize::new(0);

    INDEX.fetch_add(1, Ordering::Relaxed)
}

// NOTE `None` means in shared memory
pub fn link_section_uninit() -> Option<TokenStream2> {
    let section = format!(".uninit.rtic{}", link_section_index());

    Some(quote!(#[link_section = #section]))
}

// Regroups the inputs of a task
//
// `inputs` could be &[`input: Foo`] OR &[`mut x: i32`, `ref y: i64`]
pub fn regroup_inputs(
    inputs: &[PatType],
) -> (
    // args e.g. &[`_0`],  &[`_0: i32`, `_1: i64`]
    Vec<TokenStream2>,
    // tupled e.g. `_0`, `(_0, _1)`
    TokenStream2,
    // untupled e.g. &[`_0`], &[`_0`, `_1`]
    Vec<TokenStream2>,
    // ty e.g. `Foo`, `(i32, i64)`
    TokenStream2,
) {
    if inputs.len() == 1 {
        let ty = &inputs[0].ty;

        (
            vec![quote!(_0: #ty)],
            quote!(_0),
            vec![quote!(_0)],
            quote!(#ty),
        )
    } else {
        let mut args = vec![];
        let mut pats = vec![];
        let mut tys = vec![];

        for (i, input) in inputs.iter().enumerate() {
            let i = Ident::new(&format!("_{}", i), Span::call_site());
            let ty = &input.ty;

            args.push(quote!(#i: #ty));

            pats.push(quote!(#i));

            tys.push(quote!(#ty));
        }

        let tupled = {
            let pats = pats.clone();
            quote!((#(#pats,)*))
        };
        let ty = quote!((#(#tys,)*));
        (args, tupled, pats, ty)
    }
}

/// Get the ident for the name of the task
pub fn get_task_name(ctxt: Context, app: &App) -> Ident {
    let s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    Ident::new(&s, Span::call_site())
}

/// Generates a pre-reexport identifier for the "shared resources" struct
pub fn shared_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("SharedResources");

    mark_internal_name(&s)
}

/// Generates a pre-reexport identifier for the "local resources" struct
pub fn local_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("LocalResources");

    mark_internal_name(&s)
}

/// Generates an identifier for a ready queue
///
/// There may be several task dispatchers, one for each priority level.
/// The ready queues are SPSC queues
pub fn rq_ident(priority: u8) -> Ident {
    mark_internal_name(&format!("P{}_RQ", priority))
}

/// Generates an identifier for the `enum` of `schedule`-able tasks
pub fn schedule_t_ident() -> Ident {
    Ident::new("SCHED_T", Span::call_site())
}

/// Generates an identifier for the `enum` of `spawn`-able tasks
///
/// This identifier needs the same structure as the `RQ` identifier because there's one ready queue
/// for each of these `T` enums
pub fn spawn_t_ident(priority: u8) -> Ident {
    Ident::new(&format!("P{}_T", priority), Span::call_site())
}

/// Suffixed identifier
pub fn suffixed(name: &str) -> Ident {
    let span = Span::call_site();
    Ident::new(name, span)
}

/// Generates an identifier for a timer queue
pub fn tq_ident(name: &str) -> Ident {
    mark_internal_name(&format!("TQ_{}", name))
}

/// Generates an identifier for monotonic timer storage
pub fn monotonic_ident(name: &str) -> Ident {
    mark_internal_name(&format!("MONOTONIC_STORAGE_{}", name))
}

pub fn static_shared_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("shared_resource_{}", name.to_string()))
}

pub fn static_local_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("local_resource_{}", name.to_string()))
}

pub fn declared_static_local_resource_ident(name: &Ident, task_name: &Ident) -> Ident {
    mark_internal_name(&format!(
        "local_{}_{}",
        task_name.to_string(),
        name.to_string()
    ))
}

pub fn need_to_lock_ident(name: &Ident) -> Ident {
    Ident::new(
        &format!("{}_that_needs_to_be_locked", name.to_string()),
        name.span(),
    )
}

/// The name to get better RT flag errors
pub fn rt_err_ident() -> Ident {
    Ident::new(
        "you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml",
        Span::call_site(),
    )
}
