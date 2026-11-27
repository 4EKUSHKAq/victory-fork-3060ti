//! The title's output size and render scale (`VICTORY_FORCE_1080P`,
//! `VICTORY_NATIVE_1440P`, `VICTORY_OUTPUT`, `VICTORY_RENDER_SCALE`; each off
//! without it): places in the title's code changed at load, each compared byte
//! for byte first and left alone, said so, if it differs (another build of the
//! title).
//!
//! What the title does (`game_scarlett_master.exe` 1.0.2.1, image offsets):
//!
//! - **Output size.** Its settings (`settings.xml` in its storage,
//!   `video/resolutionFullscreen`; read into the live settings at +0x204/+0x208,
//!   the defaults' 1920x1080 at +0x5b37a20 overwritten) are applied by
//!   +0x22434a0: from +0x224352b it hands them to +0x2478130, the one writer
//!   of the screen size (+0xa9438cc/+0xa9438d0), and when the height changes
//!   calls +0xd8c3a0. The back buffers take that size through the copy
//!   +0x5eb24f0 makes (+0x75e6fe0). The display mode list it matches against
//!   is its own (+0x5a38910: 1280x720, 1664x936, 1920x1080, 2560x1440,
//!   3840x2160 at 60 Hz, table +0x615c0c0), not the DXGI output's, which is
//!   why `VICTORY_DISPLAY_MODE` changes nothing. Here that block becomes:
//!   write the size asked for into the settings (+0x204/+0x208), then the
//!   title's own else-path (screen size set, +0xd8c3a0 if the height
//!   changed). Everything that reads the settings after it (the device
//!   parameters +0x1e0fb30, the render size +0x7b4e10) sees the same size.
//!   The back buffers' pool (64 MB on Series X, 82 MB on Series S) is checked
//!   to be used up but for 2 MB (+0x247af25, an `int3`); for another size
//!   that check is jumped over.
//! - **Render scale.** The upscaler the title picks gives the render size
//!   from the output size (+0x7e92f0 -> the upscaler's +0x50 method): the
//!   title's own (+0x7e9440) multiplies by a scale at +0x90 of it, set to
//!   2/3 at +0x28f1e78 (so 2560x1440 for 4K, 1280x720 for 1080p). Here that
//!   constant is replaced for `VICTORY_RENDER_SCALE` (1.0: the render targets
//!   are the output's own 1920x1080).
//! - **Native 1440p** (`VICTORY_NATIVE_1440P=1`): the title presents at
//!   3840x2160 on Series X, its scene drawn at 2560x1440 and scaled up by its
//!   temporal pass. With the switch the Series X SDR display mode entry
//!   (+0x615c100: 3840x2160 at 60 Hz) becomes 2560x1440, the Settings
//!   preview's viewport load (+0x58048d4, which loads the legacy 3840x2160
//!   size) reads that display mode instead, the temporal scene scale
//!   (+0x28f1e7e) becomes 1.0 and the output size (2560x1440 or 1920x1080):
//!   the title makes an output of that size with the interface laid out for it
//!   and no 4K pass runs. The scanout reservation (+0x247ad4a, 64 MB: two tiled
//!   4K buffers) is sized for the mode's own pair of buffers (see
//!   [`scanout_code`]: 30 MB at 1440p, 18 MB at 1080p), so a native output also
//!   returns 34 MB (1440p) or 46 MB (1080p) of VRAM.
//!   From DexNom/VIctory's `native_1440p.rs` via trustno1-coder/VIctory-NVIDIA
//!   (`apps/victory/src/native_1440p.rs`, GPL-3.0-or-later): four fields, and
//!   their retail build hash, b99f876b74d4a77f…, is exactly our image's (the
//!   loader's BLAKE3 of the file; its SHA-256 is e95020b2…), so every byte
//!   they compare is here too (checked with a PE parse, `audit/round10/
//!   pecheck.py`).
//!
//! The port from Belobug's `VIctory-Belobug-AMD` (`crates/victory-loader/src/
//! resolution.rs`, GPL-3.0-or-later, the same title build: every site below
//! compares equal in our copy) covers the output size and the title's own
//! render scale; FidelityFX's scale (+0x28ef680) and the upscaler picker
//! (+0x7e93f0) are not ported (Round 8; the Series S FidelityFX path is the one
//! that divides by a zero jitter phase count at some scales).
//!
//! `VICTORY_FORCE_1080P=1` is the Round 8 switch: output 1920x1080 and render
//! scale 1.0, so the title itself makes its back buffers and render targets
//! 1920x1080. `VICTORY_OUTPUT`/`VICTORY_RENDER_SCALE` set on their own ask for
//! the same things piecemeal (they win over the forced values).
//!
//! `VICTORY_NATIVE_1440P=1` (Round 10 default): output 2560x1440, the SDR
//! display mode and the Settings viewport patched, render scale 1.0.
//!
//! `VICTORY_NATIVE_1080P=1` (Round 11): output 1920x1080 with the same fields -
//! the display mode's 16-byte record becomes the title's own 1920x1080 row
//! (+0x615c0e0: 1920x1080 at 60 Hz, flag 1; the stock SDR entry is the 4K row
//! at +0x615c100), so the interface is laid out at 1080p instead of the
//! blown-up Round 8 picture, where only the output size changed. It wins over
//! `VICTORY_NATIVE_1440P`.

/// Sizes the title has a display mode for (its own list, +0x615c0c0).
pub const MODES: [(u32, u32); 5] = [
    (1280, 720),
    (1664, 936),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
];

/// `VICTORY_FORCE_1080P=1`: output 1920x1080 and the title's own render scale
/// 1.0 (off otherwise, said why).
pub fn forced() -> bool {
    let spec = std::env::var("VICTORY_FORCE_1080P").ok();
    match spec.as_deref().map(str::trim) {
        None | Some("") | Some("0") => false,
        Some("1") => true,
        Some(other) => {
            eprintln!("VIctory: VICTORY_FORCE_1080P={other}: expected 1 (on) or 0 (off)");
            false
        }
    }
}

/// `VICTORY_NATIVE_1440P=1`: the native 1440p patch of DexNom's fork through
/// trustno1-coder/VIctory-NVIDIA: a 2560x1440 output, the Series X SDR display
/// mode made 2560x1440 (the interface laid out for it), the Settings preview's
/// viewport pointed at it and the temporal scene scale 1.0.
pub fn native_1440p() -> bool {
    let spec = std::env::var("VICTORY_NATIVE_1440P").ok();
    match spec.as_deref().map(str::trim) {
        None | Some("") | Some("0") => false,
        Some("1") => true,
        Some(other) => {
            eprintln!("VIctory: VICTORY_NATIVE_1440P={other}: expected 1 (on) or 0 (off)");
            false
        }
    }
}

/// The size `VICTORY_NATIVE_1440P=1` asks for.
pub const NATIVE_1440P: (u32, u32) = (2560, 1440);

/// `VICTORY_NATIVE_1080P=1`: a true 1920x1080 output with the title's own
/// display mode and Settings viewport for it (Round 11). It wins over
/// [`native_1440p`], which in turn wins over [`forced`].
pub fn native_1080p() -> bool {
    let spec = std::env::var("VICTORY_NATIVE_1080P").ok();
    match spec.as_deref().map(str::trim) {
        None | Some("") | Some("0") => false,
        Some("1") => true,
        Some(other) => {
            eprintln!("VIctory: VICTORY_NATIVE_1080P={other}: expected 1 (on) or 0 (off)");
            false
        }
    }
}

/// The size `VICTORY_NATIVE_1080P=1` asks for, the title's own 1080p mode.
pub const NATIVE_1080P: (u32, u32) = (1920, 1080);

/// `VICTORY_OUTPUT=<w>x<h>`: one of [`MODES`]. `VICTORY_FORCE_1080P=1` without
/// it asks for 1920x1080. `None` if unset (or not a mode, said why).
pub fn asked_output() -> Option<(u32, u32)> {
    if let Ok(spec) = std::env::var("VICTORY_OUTPUT") {
        let size = parse_output(&spec);
        if size.is_none() {
            eprintln!(
                "VIctory: VICTORY_OUTPUT={spec}: expected one of the title's modes: \
                 1920x1080, 2560x1440, 1280x720, 1664x936, 3840x2160"
            );
            return None;
        }
        return size;
    }
    if native_1080p() {
        return Some(NATIVE_1080P);
    }
    if native_1440p() {
        return Some(NATIVE_1440P);
    }
    forced().then_some((1920, 1080))
}

pub fn parse_output(spec: &str) -> Option<(u32, u32)> {
    let (w, h) = spec.trim().split_once(['x', 'X'])?;
    let size = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    MODES.contains(&size).then_some(size)
}

/// `VICTORY_RENDER_SCALE=<s>`, 0.5 to 1.0 (1.0: the render targets are the
/// output's own size): the title's own upscaler's scale (Series X; 2/3 without
/// it). `VICTORY_FORCE_1080P=1` without it asks for 1.0. `None` if unset (or
/// out of range, said why).
pub fn asked_render_scale() -> Option<f32> {
    if let Ok(spec) = std::env::var("VICTORY_RENDER_SCALE") {
        let scale = parse_render_scale(&spec);
        if scale.is_none() {
            eprintln!("VIctory: VICTORY_RENDER_SCALE={spec}: expected 0.5 to 1.0");
            return None;
        }
        return scale;
    }
    if let Some(mode) = crate::upscaler::asked() {
        // The title's FidelityFX path draws the scene at this share of the
        // output (the upscaler patch sets the same scale for its own pass).
        return Some(mode.scale());
    }
    if native_1080p() || native_1440p() {
        return Some(1.0);
    }
    forced().then_some(1.0)
}

pub fn parse_render_scale(spec: &str) -> Option<f32> {
    spec.trim()
        .parse::<f32>()
        .ok()
        .filter(|s| (0.5..=1.0).contains(s))
}

/// A place to change: its offset in the image and the bytes there (compared
/// first; the replacement is at most as long).
pub struct Site {
    pub at: usize,
    pub what: &'static str,
    pub bytes: &'static [u8],
}

/// The screen size setter, its screen height, and what +0x22434a0 calls when
/// the height changed.
pub const SET_SCREEN_SIZE: usize = 0x2478130;
pub const SCREEN_HEIGHT: usize = 0xa9438d0;
pub const HEIGHT_CHANGED: usize = 0xd8c3a0;

/// +0x22434a0 from +0x224352b to +0x2243568: the settings' size handed to the
/// screen size.
pub const OUTPUT: Site = Site {
    at: 0x224352b,
    what: "settings' output size applied",
    bytes: &[
        0x8b, 0x0d, 0x9b, 0x03, 0x70, 0x08, // mov ecx, [+0xa9438cc]
        0x8b, 0x86, 0x04, 0x02, 0x00, 0x00, // mov eax, [rsi+204h]
        0x39, 0xc1, 0x75, 0x0d, // cmp ecx, eax ; jne
        0x8b, 0x96, 0x08, 0x02, 0x00, 0x00, // mov edx, [rsi+208h]
        0xe8, 0xea, 0x4b, 0x23, 0x00, 0xeb, 0x20, // call +0x2478130 ; jmp +0x2243568
        0x8b, 0xbe, 0x08, 0x02, 0x00, 0x00, // mov edi, [rsi+208h]
        0x8b, 0x2d, 0x7c, 0x03, 0x70, 0x08, // mov ebp, [+0xa9438d0]
        0x89, 0xc1, 0x89, 0xfa, // mov ecx, eax ; mov edx, edi
        0xe8, 0xd3, 0x4b, 0x23, 0x00, // call +0x2478130
        0x39, 0xfd, 0x74, 0x07, 0xb1, 0x01, // cmp ebp, edi ; je ; mov cl, 1
        0xe8, 0x38, 0x8e, 0xb4, 0xfe, // call +0xd8c3a0
    ],
};

/// +0x247ad10, after the back buffers are made in their pool (64 MB on Series
/// X, 82 MB on Series S): `int3` unless less than 2 MB of the pool is left
/// over. Sized for 3840x2160; with another output size the check becomes a
/// jump over the `int3`; the pool stays as large.
pub const POOL_CHECK: Site = Site {
    at: 0x247af25,
    what: "back buffer pool's left-over check",
    bytes: &[0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0x76, 0x01, 0xcc], // cmp rax, 1FFFFFh ; jbe +1 ; int3
};

/// What goes at [`POOL_CHECK`]: `jmp +1` in place of `jbe +1`.
pub fn pool_check_code() -> Vec<u8> {
    let mut c = POOL_CHECK.bytes[..8].to_vec();
    c[6] = 0xeb;
    c
}

/// The scanout reservation (+0x247ad4a): `mov ecx, 4000000h`, the 64 MB room
/// for two tiled 3840x2160 RGBA8 buffers. The native patches size it for their
/// own output (two buffers, each rounded to 64 KiB, the pair rounded up to
/// 2 MiB - the unit of the title's allocation check): 30 MB at 1440p, 18 MB at
/// 1080p, so 34 MB and 46 MB of VRAM come back. From VIctory-NVIDIA's
/// `native_1440p.rs`, which computes it from `victory_formats::texture::Texture`.
pub const SCANOUT_RESERVATION: Site = Site {
    at: 0x247ad4a,
    what: "the scanout reservation (64 MB)",
    bytes: &[0xb9, 0, 0, 0, 4], // mov ecx, 4000000h
};

/// The room two scanout buffers of `width` x `height` need: each tiled RGBA8
/// buffer rounded to 64 KiB, the pair rounded up to 2 MiB (the unit of the
/// title's allocation check at [`POOL_CHECK`]). `None` if victory-formats does
/// not know the layout.
pub fn scanout_pool_size(width: u32, height: u32) -> Option<u32> {
    use victory_formats::texture::{Dimension, Swizzle, Texture, TextureDesc};
    let texture = Texture::new(&TextureDesc {
        dimension: Dimension::Tex2d,
        width,
        height,
        depth_or_array: 1,
        mips: 1,
        format: 28, // RGBA8
        samples: 1,
        swizzle: Swizzle::SW_64KB_D_X,
    })
    .ok()?;
    let pool = (texture.size.next_multiple_of(64 * 1024) * 2).next_multiple_of(2 << 20);
    u32::try_from(pool).ok()
}

/// What goes at [`SCANOUT_RESERVATION`] for `width` x `height`: the same
/// `mov ecx, imm32`, the immediate made the pair of buffers' room.
pub fn scanout_code(width: u32, height: u32) -> Option<Vec<u8>> {
    let pool = scanout_pool_size(width, height)?;
    let mut c = vec![SCANOUT_RESERVATION.bytes[0]]; // mov ecx, imm32
    c.extend(pool.to_le_bytes());
    Some(c)
}

/// The Series X SDR display mode entry (+0x615c100): 3840x2160 at 60 Hz in the
/// title's own mode table (its fifth 16-byte entry). The native 1440p patch
/// makes it 2560x1440; `VICTORY_FORCE_1080P` alone leaves it alone.
pub const SDR_MODE: Site = Site {
    at: 0x615c100,
    what: "the title's SDR display mode (3840x2160)",
    bytes: &[0, 15, 0, 0, 0x70, 8, 0, 0, 60, 0, 0, 0, 1, 0, 0, 0],
};

/// What goes at [`SDR_MODE`] for `width` x `height` (8 bytes of its 16).
pub fn sdr_mode_code(width: u32, height: u32) -> Vec<u8> {
    let mut c = SDR_MODE.bytes[..8].to_vec();
    c[..4].copy_from_slice(&width.to_le_bytes());
    c[4..8].copy_from_slice(&height.to_le_bytes());
    c
}

/// Settings' preview viewport size load (+0x58048d4): `vmovss xmm1,
/// [rip+disp]` reads the legacy 3840x2160 size even when a 1440p interface
/// surface exists. The native 1440p patch points it at the SDR display mode's
/// width.
pub const UI_VIEWPORT: Site = Site {
    at: 0x58048d4,
    what: "Settings' preview viewport size load",
    bytes: &[0xc5, 0xfb, 0x10, 0x0d, 0x10, 0xf0, 0x13, 0x05], // vmovss xmm1, [rip+0x513f010]
};

/// What goes at [`UI_VIEWPORT`]: the same load, its displacement made to the
/// SDR display mode's entry.
pub fn ui_viewport_code() -> Vec<u8> {
    let mut c = UI_VIEWPORT.bytes.to_vec();
    let displacement = (SDR_MODE.at as i64 - (UI_VIEWPORT.at as i64 + 8)) as i32;
    c[4..8].copy_from_slice(&displacement.to_le_bytes());
    c
}

/// The title's own upscaler, its method +0x28f1e70: `mov dword [rcx+90h],
/// 2/3` (the scale +0x7e9440 renders at).
pub const OWN_SCALE: Site = Site {
    at: 0x28f1e78,
    what: "the title's own upscaler's render scale",
    bytes: &[0xc7, 0x81, 0x90, 0x00, 0x00, 0x00, 0xab, 0xaa, 0x2a, 0x3f],
};

/// `rel32` from the end of an instruction at image offset `next` to `to`.
fn rel(to: usize, next: usize) -> [u8; 4] {
    ((to as i64 - next as i64) as i32).to_le_bytes()
}

/// What goes at [`OUTPUT`] for `width` x `height` (the same length).
pub fn output_code(width: u32, height: u32) -> Vec<u8> {
    let at = OUTPUT.at;
    let mut c = Vec::with_capacity(OUTPUT.bytes.len());
    c.push(0xb8); // mov eax, width
    c.extend(width.to_le_bytes());
    c.extend([0x89, 0x86, 0x04, 0x02, 0x00, 0x00]); // mov [rsi+204h], eax
    c.push(0xbf); // mov edi, height
    c.extend(height.to_le_bytes());
    c.extend([0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]); // mov [rsi+208h], edi
    c.extend([0x8b, 0x2d]); // mov ebp, [screen height]
    c.extend(rel(SCREEN_HEIGHT, at + c.len() + 4));
    c.extend([0x89, 0xc1, 0x89, 0xfa]); // mov ecx, eax ; mov edx, edi
    c.push(0xe8); // call the setter
    c.extend(rel(SET_SCREEN_SIZE, at + c.len() + 4));
    let end = OUTPUT.bytes.len();
    c.extend([0x39, 0xfd, 0x74, (end - (c.len() + 4)) as u8]); // cmp ebp, edi ; je end
    c.extend([0xb1, 0x01, 0xe8]); // mov cl, 1 ; call
    c.extend(rel(HEIGHT_CHANGED, at + c.len() + 4));
    c.extend([0xeb, (end - (c.len() + 2)) as u8]); // jmp end
    c.resize(end, 0xcc);
    c
}

/// What goes at [`OWN_SCALE`] for `scale`.
pub fn own_scale_code(scale: f32) -> Vec<u8> {
    let mut c = OWN_SCALE.bytes[..6].to_vec();
    c.extend(scale.to_le_bytes());
    c
}

/// What became of a site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Patched {
    Done,
    /// Its bytes are not the ones expected: left alone.
    Differs,
    Failed(&'static str),
}

/// Compares `site`'s bytes in the image of `size` bytes at `base`.
///
/// # Safety
/// The image is mapped.
pub unsafe fn matches(base: usize, size: usize, site: &Site) -> Patched {
    if site
        .at
        .checked_add(site.bytes.len())
        .is_none_or(|end| end > size)
    {
        return Patched::Failed("outside the image");
    }
    // SAFETY: inside the mapped image (checked above).
    let there =
        unsafe { std::slice::from_raw_parts((base + site.at) as *const u8, site.bytes.len()) };
    if there == site.bytes {
        Patched::Done
    } else {
        Patched::Differs
    }
}

/// Writes `code` over `site` if its bytes are the expected ones.
///
/// # Safety
/// The image stays mapped and no thread runs its code yet.
#[cfg(windows)]
pub unsafe fn write(base: usize, size: usize, site: &Site, code: &[u8]) -> Patched {
    use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
    use windows_sys::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, VirtualProtect};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    if code.len() > site.bytes.len() {
        return Patched::Failed("replacement longer than the bytes compared");
    }
    // SAFETY: the caller's promise.
    let found = unsafe { matches(base, size, site) };
    if found != Patched::Done {
        return found;
    }
    let at = base + site.at;
    let mut old = 0;
    // SAFETY: inside the image (checked); made writable for this and put
    // back; nothing runs it yet.
    unsafe {
        if VirtualProtect(at as *const _, code.len(), PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return Patched::Failed("Windows would not make it writable");
        }
        std::ptr::copy_nonoverlapping(code.as_ptr(), at as *mut u8, code.len());
        VirtualProtect(at as *const _, code.len(), old, &mut old);
        FlushInstructionCache(GetCurrentProcess(), at as *const _, code.len());
    }
    Patched::Done
}

/// Applies what the switches ask for; a line each in the log and on standard
/// output. `VICTORY_FORCE_1080P=1` asks for 1920x1080 and the title's own
/// render scale 1.0; `VICTORY_OUTPUT`/`VICTORY_RENDER_SCALE` for their own
/// values (one of the title's modes, 0.5 to 1.0).
///
/// # Safety
/// The image of `size` bytes at `base` stays mapped and no thread runs its
/// code yet.
#[cfg(windows)]
pub unsafe fn apply_asked(base: usize, size: usize) {
    let mut asked: Vec<(&Site, Vec<u8>, String)> = Vec::new();
    let output = asked_output();
    if let Some((w, h)) = output {
        asked.push((&OUTPUT, output_code(w, h), format!("{w}x{h}")));
        if (w, h) != (3840, 2160) {
            asked.push((&POOL_CHECK, pool_check_code(), format!("{w}x{h}")));
        }
        if (w, h) == NATIVE_1080P || (w, h) == NATIVE_1440P {
            // The native fields: the display mode the title reads (the game's
            // own record for that size), Settings' preview viewport and the
            // scanout reservation. The size is what matters, so the same
            // bytes serve VICTORY_NATIVE_1080P, VICTORY_NATIVE_1440P and a
            // VICTORY_OUTPUT of that size.
            asked.push((&SDR_MODE, sdr_mode_code(w, h), format!("{w}x{h}")));
            asked.push((&UI_VIEWPORT, ui_viewport_code(), format!("{w}x{h}")));
            match scanout_code(w, h) {
                Some(code) => asked.push((&SCANOUT_RESERVATION, code, format!("{w}x{h}"))),
                None => eprintln!(
                    "VIctory: {w}x{h}: victory-formats does not know the scanout layout, \
                     the reservation is left as it is"
                ),
            }
        } else if native_1080p() || native_1440p() {
            eprintln!(
                "VIctory: the native display patch with VICTORY_OUTPUT={w}x{h}: \
                 its fields are for 1920x1080 and 2560x1440 only, left alone"
            );
        }
    }
    if let Some(s) = asked_render_scale() {
        asked.push((&OWN_SCALE, own_scale_code(s), format!("{s}")));
    }
    for (site, code, value) in asked {
        // SAFETY: the caller's promise.
        let done = unsafe { write(base, size, site, &code) };
        let offset = format!("{:#x}", site.at);
        match done {
            Patched::Done => {
                tracing::info!(site = %offset, what = site.what, value, "resolution: patched")
            }
            Patched::Differs => tracing::warn!(
                site = %offset,
                what = site.what,
                "title build differs, not patched: the title's bytes there are not the ones expected"
            ),
            Patched::Failed(why) => {
                tracing::warn!(site = %offset, what = site.what, why, "resolution: not patched")
            }
        }
        println!(
            "resolution: {} ({value}) at title+{offset}: {done:?}",
            site.what
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_parse() {
        assert_eq!(parse_output("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_output(" 2560X1440 "), Some((2560, 1440)));
        assert_eq!(parse_output("1600x900"), None);
        assert_eq!(parse_output("1080p"), None);
        assert_eq!(parse_render_scale("0.67"), Some(0.67));
        assert_eq!(parse_render_scale("0.5"), Some(0.5));
        assert_eq!(parse_render_scale("1"), Some(1.0));
        assert_eq!(parse_render_scale("0.4"), None);
        assert_eq!(parse_render_scale("1.5"), None);
    }

    /// The calls and loads in the bytes compared are the ones the replacement
    /// keeps (whatever build: relative calls and the else-path are checked).
    #[test]
    fn the_output_block_is_the_titles() {
        let b = OUTPUT.bytes;
        assert_eq!(b.len(), 61);
        let target = |op_end: usize| {
            (OUTPUT.at + op_end) as i64
                + i64::from(i32::from_le_bytes(
                    b[op_end - 4..op_end].try_into().unwrap(),
                ))
        };
        assert_eq!(target(27), SET_SCREEN_SIZE as i64);
        assert_eq!(target(50), SET_SCREEN_SIZE as i64);
        assert_eq!(target(61), HEIGHT_CHANGED as i64);
        assert_eq!(target(41), SCREEN_HEIGHT as i64);
        assert_eq!(target(6), (SCREEN_HEIGHT - 4) as i64, "the width before it");
        // jne to the else path at 29; jmp +0x20 and je +7 both land at the end.
        assert_eq!((b[14], 16 + b[15] as usize), (0x75, 29));
        assert_eq!((b[27], 29 + b[28] as usize), (0xeb, 61));
        assert_eq!((b[52], 54 + b[53] as usize), (0x74, 61));
    }

    #[test]
    fn the_output_code_keeps_the_titles_else_path() {
        let c = output_code(1920, 1080);
        assert_eq!(c.len(), OUTPUT.bytes.len());
        assert_eq!(c[0], 0xb8);
        assert_eq!(c[1..5], 1920u32.to_le_bytes());
        assert_eq!(c[5..11], [0x89, 0x86, 0x04, 0x02, 0x00, 0x00]);
        assert_eq!(c[12..16], 1080u32.to_le_bytes());
        assert_eq!(c[16..22], [0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]);
        let target = |op_end: usize| {
            (OUTPUT.at + op_end) as i64
                + i64::from(i32::from_le_bytes(
                    c[op_end - 4..op_end].try_into().unwrap(),
                ))
        };
        assert_eq!(c[22..24], [0x8b, 0x2d]);
        assert_eq!(target(28), SCREEN_HEIGHT as i64);
        assert_eq!(c[32], 0xe8);
        assert_eq!(target(37), SET_SCREEN_SIZE as i64);
        assert_eq!(c[37..40], [0x39, 0xfd, 0x74]);
        assert_eq!(41 + c[40] as usize, 61, "je lands at the end");
        assert_eq!(c[43], 0xe8);
        assert_eq!(target(48), HEIGHT_CHANGED as i64);
        assert_eq!(c[48], 0xeb);
        assert_eq!(50 + c[49] as usize, 61, "jmp lands at the end");
        assert!(c[50..].iter().all(|&x| x == 0xcc));
    }

    #[test]
    fn the_scale_code() {
        let own = own_scale_code(1.0);
        assert_eq!(own[..6], OWN_SCALE.bytes[..6]);
        assert_eq!(own[6..], 1.0f32.to_le_bytes());
        assert_eq!(OWN_SCALE.bytes[6..], (2.0f32 / 3.0).to_le_bytes());
        assert_eq!(
            pool_check_code(),
            [0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0xeb, 0x01]
        );
    }

    /// The native 1440p patch writes the display mode's size and points the
    /// Settings viewport's load at that mode, keeping everything else of the
    /// compared bytes; the stock mode entry is the 4K one.
    #[test]
    fn the_native_1440p_fields_keep_the_titles() {
        assert_eq!(
            &SDR_MODE.bytes[..8],
            &[0, 15, 0, 0, 0x70, 8, 0, 0],
            "stock Series X SDR mode is 3840x2160"
        );
        let mode = sdr_mode_code(2560, 1440);
        assert_eq!(mode.len(), 8, "only the size of the 16-byte entry is written");
        assert_eq!(&mode[..4], 2560u32.to_le_bytes());
        assert_eq!(&mode[4..8], 1440u32.to_le_bytes());
        assert_eq!(&SDR_MODE.bytes[8..], &[60, 0, 0, 0, 1, 0, 0, 0], "60 Hz and the flags stay");
        let viewport = ui_viewport_code();
        assert_eq!(&viewport[..4], &UI_VIEWPORT.bytes[..4], "the load stays");
        let loaded = UI_VIEWPORT.at as i64
            + 8
            + i64::from(i32::from_le_bytes(viewport[4..8].try_into().unwrap()));
        assert_eq!(loaded, SDR_MODE.at as i64, "the load reads the display mode");
        assert_eq!(UI_VIEWPORT.bytes, [0xc5, 0xfb, 0x10, 0x0d, 0x10, 0xf0, 0x13, 0x05]);
        assert_eq!(NATIVE_1440P, (2560, 1440));
    }

    /// The native 1080p patch asks for the size of the title's own third mode
    /// entry (+0x615c0e0: 1920x1080 at 60 Hz, flag 1) - the record the patched
    /// SDR entry becomes - and the scanout reservation for that pair of
    /// buffers.
    #[test]
    fn the_native_1080p_fields_are_the_titles_own_1080p_mode() {
        const OWN_1080P: [u8; 16] =
            [0x80, 0x07, 0, 0, 0x38, 0x04, 0, 0, 60, 0, 0, 0, 1, 0, 0, 0];
        let mode = sdr_mode_code(1920, 1080);
        assert_eq!(&mode[..8], &OWN_1080P[..8], "the game's own 1920x1080 row");
        assert_eq!(&SDR_MODE.bytes[8..], &OWN_1080P[8..], "60 Hz and the flag stay");
        assert_eq!(NATIVE_1080P, (1920, 1080));
        // The reservation's stock immediate is 64 MB; the pairs are 64 MB at
        // 4K (the value the stock code writes), 30 MB at 1440p and 18 MB at
        // 1080p - 34 MB and 46 MB of VRAM come back.
        assert_eq!(SCANOUT_RESERVATION.bytes, [0xb9, 0, 0, 0, 4]);
        assert_eq!(scanout_pool_size(3840, 2160), Some(0x0400_0000));
        assert_eq!(scanout_pool_size(2560, 1440), Some(30 << 20));
        assert_eq!(scanout_pool_size(1920, 1080), Some(18 << 20));
        // The site keeps its instruction; only the immediate moves.
        let code = scanout_code(1920, 1080).unwrap();
        assert_eq!(code, [0xb9, 0, 0, 0x20, 0x01]);
        assert_eq!(code.len(), SCANOUT_RESERVATION.bytes.len());
        assert_eq!(code[0], SCANOUT_RESERVATION.bytes[0]);
    }

    #[cfg(windows)]
    #[test]
    fn a_site_is_written_only_over_its_bytes() {
        use windows_sys::Win32::System::Memory::{
            MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, VirtualAlloc, VirtualFree,
        };
        // SAFETY: fresh memory of our own, read-only to code as the image's is.
        unsafe {
            let place =
                VirtualAlloc(std::ptr::null(), 0x1000, MEM_COMMIT | MEM_RESERVE, 0x04) as *mut u8;
            assert!(!place.is_null());
            let site = Site {
                at: 0x100,
                what: "test",
                bytes: OWN_SCALE.bytes,
            };
            std::ptr::copy_nonoverlapping(site.bytes.as_ptr(), place.add(0x100), site.bytes.len());
            assert_eq!(
                write(place as usize, 0x1000, &site, &own_scale_code(0.5)),
                Patched::Done
            );
            assert_eq!(
                std::slice::from_raw_parts(place.add(0x106), 4),
                0.5f32.to_le_bytes()
            );
            // Already changed: its bytes differ now, nothing is written again.
            assert_eq!(
                write(place as usize, 0x1000, &site, &own_scale_code(0.75)),
                Patched::Differs
            );
            assert_eq!(
                std::slice::from_raw_parts(place.add(0x106), 4),
                0.5f32.to_le_bytes()
            );
            let far = Site {
                at: 0xffa,
                what: "test",
                bytes: OWN_SCALE.bytes,
            };
            assert!(matches!(
                write(place as usize, 0x1000, &far, &own_scale_code(0.5)),
                Patched::Failed(_)
            ));
            VirtualFree(place.cast(), 0, MEM_RELEASE);
        }
    }
}
