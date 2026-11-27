//! `amd_fidelityfx_loader_xbox.dll`: the interpose of the title's FidelityFX
//! imports (`VICTORY_UPSCALER=7` or `VICTORY_DLSS=<mode>`; see
//! [`victory_loader::upscaler`]).
//!
//! Round 11: the frame the title's FidelityFX path dispatches is the one a
//! DLSS Super Resolution upscale would take (same colour, depth, motion
//! vectors, jitter and output). This module binds the title's five imports of
//! the loader to wrappers of ours, keeps the title's own functions and calls
//! them for everything: the wrappers read the dispatches, say once what they
//! see, and hand the call back. The title's own upscaler therefore still runs;
//! the Streamline/DLSS engine that would replace the upscale dispatch is not
//! in this build (audit\dlss-prep.md, Step 5-6 of its plan) - the wrappers are
//! where it plugs in (`Taken::No` today, `Taken::Dlss` later).
//!
//! The description layouts are the FidelityFX SDK's API (MIT) as DexNom's fork
//! reads them (DexNom/VIctory, `crates/victory-hle/src/modules/fidelityfx.rs`,
//! GPL-3.0-or-later) through trustno1-coder/VIctory-NVIDIA
//! (`crates/victory-hle/src/modules/fidelityfx.rs`, commit c437b0594b).
#![allow(non_snake_case)]
// Title code calls these through its import table.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use victory_loader::{ImportResolver, Provenance, Resolution};
use victory_pe::ImportSymbol;

/// The DLL the title imports the API from (it loads the upscaler itself).
pub const LOADER_DLL: &str = "amd_fidelityfx_loader_xbox.dll";

// Return codes.
const OK: u32 = 0;

// Description types.
const CREATE_UPSCALE: u64 = 0x1_0000;
const DISPATCH_UPSCALE: u64 = 0x1_0001;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Header {
    kind: u64,
    next: *mut Header,
}

/// An upscaler context: flags, largest render and output sizes, a message
/// callback.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CreateUpscale {
    header: Header,
    flags: u32,
    max_render: [u32; 2],
    max_output: [u32; 2],
    message: usize,
}

/// A resource, its description (type, format, sizes, mips, flags, usage)
/// and the state it is in.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Resource {
    resource: *mut c_void,
    description: [u32; 8],
    state: u32,
}

/// One upscaling: the command list, then colour, depth, motion vectors,
/// exposure, reactive mask, transparency mask and output; jitter,
/// motion-vector scale, sizes, sharpening, frame time (ms), pre-exposure,
/// reset, camera and flags. (The C `bool`s are read as bytes.)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DispatchUpscale {
    header: Header,
    list: *mut c_void,
    resources: [Resource; 7],
    jitter: [f32; 2],
    motion_scale: [f32; 2],
    render_size: [u32; 2],
    upscale_size: [u32; 2],
    enable_sharpening: u8,
    sharpness: f32,
    frame_time_delta: f32,
    pre_exposure: f32,
    reset: u8,
    camera_near: f32,
    camera_far: f32,
    camera_fov_angle_vertical: f32,
    view_space_to_meters_factor: f32,
    flags: u32,
}

const _: () = assert!(size_of::<CreateUpscale>() == 0x30);
const _: () = assert!(size_of::<Resource>() == 0x30);
const _: () = assert!(size_of::<DispatchUpscale>() == 0x1b0);

type Create = unsafe extern "C" fn(*mut *mut c_void, *mut Header, *const c_void) -> u32;
type Destroy = unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> u32;
/// Configure, query and dispatch.
type Call = unsafe extern "C" fn(*mut *mut c_void, *mut Header) -> u32;

/// The title's own functions, by [`NAMES`].
static ORIGINALS: [AtomicUsize; 5] = [const { AtomicUsize::new(0) }; 5];
const NAMES: [&str; 5] = [
    "ffxCreateContext",
    "ffxDestroyContext",
    "ffxConfigure",
    "ffxQuery",
    "ffxDispatch",
];

/// An upscaler context the title made: its creation flags and largest sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Created {
    flags: u32,
    max_render: [u32; 2],
    max_output: [u32; 2],
}

static CONTEXTS: Mutex<Option<HashMap<usize, Created>>> = Mutex::new(None);

macro_rules! once {
    ($level:ident, $($arg:tt)*) => {{
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| tracing::$level!($($arg)*));
    }};
}

fn original(i: usize) -> usize {
    ORIGINALS[i].load(Relaxed)
}

/// The description as the API's, if it is the type asked for.
///
/// # Safety
/// `desc` points at a description the title passes (its `kind` field first).
unsafe fn read<T: Copy>(desc: *mut Header, kind: u64) -> Option<T> {
    if desc.is_null() {
        return None;
    }
    // SAFETY: the title's description; its first field is the type.
    let header = unsafe { std::ptr::read(desc) };
    if header.kind != kind {
        return None;
    }
    Some(unsafe { std::ptr::read(desc.cast::<T>()) })
}

unsafe extern "C" fn create(
    context: *mut *mut c_void,
    desc: *mut Header,
    memory: *const c_void,
) -> u32 {
    let call: Create = unsafe { std::mem::transmute(original(0)) };
    let code = unsafe { call(context, desc, memory) };
    if code == OK && !context.is_null() {
        // SAFETY: the title's description, read as the API's.
        if let Some(created) = unsafe { read::<CreateUpscale>(desc, CREATE_UPSCALE) } {
            once!(
                info,
                max_render = ?created.max_render,
                max_output = ?created.max_output,
                flags = created.flags,
                "fidelityfx: the title made its upscaler context"
            );
            let mut contexts = CONTEXTS.lock().unwrap_or_else(|e| e.into_inner());
            let context = unsafe { *context } as usize;
            contexts.get_or_insert_with(HashMap::new).insert(
                context,
                Created {
                    flags: created.flags,
                    max_render: created.max_render,
                    max_output: created.max_output,
                },
            );
        }
    }
    code
}

unsafe extern "C" fn destroy(context: *mut *mut c_void, memory: *const c_void) -> u32 {
    let call: Destroy = unsafe { std::mem::transmute(original(1)) };
    let code = unsafe { call(context, memory) };
    if code == OK && !context.is_null() {
        let context = unsafe { *context } as usize;
        let mut contexts = CONTEXTS.lock().unwrap_or_else(|e| e.into_inner());
        contexts.get_or_insert_with(HashMap::new).remove(&context);
    }
    code
}

unsafe extern "C" fn configure(context: *mut *mut c_void, desc: *mut Header) -> u32 {
    let call: Call = unsafe { std::mem::transmute(original(2)) };
    unsafe { call(context, desc) }
}

unsafe extern "C" fn query(context: *mut *mut c_void, desc: *mut Header) -> u32 {
    let call: Call = unsafe { std::mem::transmute(original(3)) };
    unsafe { call(context, desc) }
}

/// The upscale dispatch: what DLSS would take. Read, say once, and hand back to
/// the title's own upscaler (there is no DLSS engine in this build).
unsafe extern "C" fn dispatch(context: *mut *mut c_void, desc: *mut Header) -> u32 {
    // SAFETY: the title's description, read as the API's.
    if let Some(up) = unsafe { read::<DispatchUpscale>(desc, DISPATCH_UPSCALE) } {
        once!(
            info,
            render = ?up.render_size,
            upscale = ?up.upscale_size,
            jitter = ?up.jitter,
            motion_scale = ?up.motion_scale,
            reset = up.reset,
            "fidelityfx: the first upscale dispatch of the FidelityFX path (the one DLSS \
             would take) - the title's own upscaler runs: no DLSS engine in this build"
        );
    }
    let call: Call = unsafe { std::mem::transmute(original(4)) };
    unsafe { call(context, desc) }
}

/// Binds the title's imports of [`LOADER_DLL`] to the wrappers when the
/// FidelityFX path is asked for, keeping what `inner` binds them to (the
/// title's own DLL) for them to call; otherwise binds as `inner` does.
pub struct Interpose<'a> {
    pub inner: &'a dyn ImportResolver,
}

impl ImportResolver for Interpose<'_> {
    fn resolve(&self, dll: &str, symbol: &ImportSymbol) -> Resolution {
        let inner = self.inner.resolve(dll, symbol);
        if !dll.eq_ignore_ascii_case(LOADER_DLL) || victory_loader::upscaler::asked().is_none() {
            return inner;
        }
        let ImportSymbol::Name { name, .. } = symbol else {
            return inner;
        };
        let Some(i) = NAMES.iter().position(|n| n.eq_ignore_ascii_case(name)) else {
            return inner;
        };
        if let Resolution::Found { address, .. } = inner {
            ORIGINALS[i].store(address, Relaxed);
        } else {
            return inner;
        }
        let wrapper = match i {
            0 => create as *const () as usize,
            1 => destroy as *const () as usize,
            2 => configure as *const () as usize,
            3 => query as *const () as usize,
            _ => dispatch as *const () as usize,
        };
        once!(
            info,
            symbol = name.as_str(),
            "fidelityfx: the title's FidelityFX import is bound to the interposer"
        );
        Resolution::Found {
            address: wrapper,
            provenance: Provenance::Hle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_description_layouts_are_the_apis() {
        // Sizes asserted at compile time above; here the fields DLSS needs.
        assert_eq!(size_of::<Header>(), 16);
        assert_eq!(size_of::<DispatchUpscale>(), 0x1b0);
        assert_eq!(size_of::<CreateUpscale>(), 0x30);
        assert_eq!(size_of::<Resource>(), 0x30);
        assert_eq!(DISPATCH_UPSCALE, 0x1_0001);
        assert_eq!(CREATE_UPSCALE, 0x1_0000);
    }

    /// A description of the wrong type is not read as one of ours.
    #[test]
    fn a_description_of_another_type_is_refused() {
        let mut header = Header {
            kind: 0x1_0002,
            next: std::ptr::null_mut(),
        };
        // SAFETY: a description of our own on the stack.
        let got = unsafe { read::<CreateUpscale>(&mut header, CREATE_UPSCALE) };
        assert!(got.is_none());
        header.kind = CREATE_UPSCALE;
        // SAFETY: a description of our own on the stack, typed as asked.
        let got = unsafe { read::<CreateUpscale>(&mut header, CREATE_UPSCALE) };
        assert!(got.is_some());
    }
}
