//! The title's own FidelityFX upscaler path (renderer 7) on Series X:
//! `VICTORY_UPSCALER=7` (the path, its own upscaler at `VICTORY_FFX_MODE`, and
//! the temporal scene scale at that mode's) and `VICTORY_DLSS=<mode>` (the same,
//! for a DLSS Super Resolution upscale to take the dispatches over; the
//! interposer in `victory_hle::modules::fidelityfx` listens, and the title's
//! own upscaler runs whenever DLSS does not).
//!
//! In the stock Series X image the renderer selection picks 6 (the title's own
//! temporal upscaler) and the FidelityFX path (7) only exists for Series S
//! (`mov eax, 6; cmovne eax, ecx` at +0x7e9421, the console check the branch
//! reads). The patch makes it 7 every time, notes `mode` as the FidelityFX
//! quality mode (the title's scaled branch then multiplies the output size by
//! the scale the scaled branch loads) and sets the temporal scene scale to the
//! same ratio, so the title draws `scale` of the output and its own upscaler
//! (or DLSS, later) writes the output.
//!
//! Ported from DexNom/VIctory's `prepare_fsr_4k` through
//! trustno1-coder/VIctory-NVIDIA (`apps/victory/src/native_1440p.rs`,
//! GPL-3.0-or-later). Every site is compared byte for byte first; nothing is
//! written if one differs. Our image's loader BLAKE3 hash is the retail
//! `b99f876b74d4a77f…` those sources require, and every compared byte was
//! checked here with a PE parse (`audit/round10/pecheck.py`).

use crate::force_1080p::{Patched, Site, matches, write};

/// A DLSS/FidelityFX quality mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dlaa,
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

impl Mode {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "dlaa" | "native" => Mode::Dlaa,
            "quality" => Mode::Quality,
            "balanced" => Mode::Balanced,
            "performance" => Mode::Performance,
            "ultraperformance" | "ultra-performance" | "ultra_performance" => {
                Mode::UltraPerformance
            }
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Dlaa => "dlaa",
            Mode::Quality => "quality",
            Mode::Balanced => "balanced",
            Mode::Performance => "performance",
            Mode::UltraPerformance => "ultraperformance",
        }
    }

    /// The render size over the output size (DLSS's ratios; the title's own
    /// FSR uses the same ones).
    pub fn scale(self) -> f32 {
        match self {
            Mode::Dlaa => 1.0,
            Mode::Quality => 2.0 / 3.0,
            Mode::Balanced => 0.58,
            Mode::Performance => 0.5,
            Mode::UltraPerformance => 1.0 / 3.0,
        }
    }

    /// The FidelityFX quality mode of the same ratio (native AA, quality,
    /// balanced, performance, ultra performance), which the title notes.
    pub fn ffx_mode(self) -> u8 {
        match self {
            Mode::Dlaa => 0,
            Mode::Quality => 1,
            Mode::Balanced => 2,
            Mode::Performance => 3,
            Mode::UltraPerformance => 4,
        }
    }
}

/// `VICTORY_DLSS=<mode|1>`: DLSS Super Resolution asked for (the title's
/// FidelityFX path is selected and the interposer takes its dispatches). `1`
/// is `quality`. `0`/unset: off. `VICTORY_FFX_MODE` sets the mode instead.
pub fn dlss_wanted() -> bool {
    match std::env::var("VICTORY_DLSS").ok().as_deref().map(str::trim) {
        None | Some("") | Some("0") => false,
        Some(_) => true,
    }
}

/// The mode the upscaler path (and DLSS, when it is on) asks for: `VICTORY_DLSS`
/// (a name or `1` = quality), else `VICTORY_FFX_MODE`, else quality.
pub fn asked() -> Option<Mode> {
    let mode = match std::env::var("VICTORY_UPSCALER").ok().as_deref().map(str::trim) {
        None | Some("") => None,
        Some("0") => return None,
        Some("7") => Some(
            std::env::var("VICTORY_FFX_MODE")
                .ok()
                .map(|v| parse_mode(&v))
                .unwrap_or(Some(Mode::Quality))?,
        ),
        Some(other) => {
            eprintln!("VIctory: VICTORY_UPSCALER={other}: expected 7 (the FidelityFX path) or 0");
            return None;
        }
    };
    if let Ok(spec) = std::env::var("VICTORY_DLSS") {
        if spec.trim() == "0" || spec.trim().is_empty() {
            return mode;
        }
        let dlss = if spec.trim() == "1" {
            Some(Mode::Quality)
        } else {
            parse_mode(&spec)
        };
        if dlss.is_none() {
            eprintln!(
                "VIctory: VICTORY_DLSS={spec}: expected 1, 0 or one of dlaa, quality, \
                 balanced, performance, ultraperformance"
            );
        }
        return dlss.or(mode);
    }
    mode
}

fn parse_mode(spec: &str) -> Option<Mode> {
    let mode = Mode::from_name(spec);
    if mode.is_none() {
        eprintln!(
            "VIctory: VICTORY_FFX_MODE={spec}: expected one of dlaa, quality, balanced, \
             performance, ultraperformance"
        );
    }
    mode
}

/// The renderer selection: `mov eax, 6; cmovne eax, ecx` - the title's own
/// temporal pass, or the FidelityFX path (7) when the console check said so.
pub const RENDERER_SELECTION: Site = Site {
    at: 0x7e9421,
    what: "the renderer selection (the title's temporal pass)",
    bytes: &[0xb8, 6, 0, 0, 0, 0x0f, 0x45, 0xc1],
};

/// What goes at [`RENDERER_SELECTION`]: renderer 7 every time.
pub fn renderer_code() -> Vec<u8> {
    let mut c = RENDERER_SELECTION.bytes.to_vec();
    c[1] = 7;
    c[5..].copy_from_slice(&[0x90, 0x90, 0x90]);
    c
}

/// The FidelityFX path's render size: the console check's result is noted as
/// the FidelityFX quality mode and picks the scaled branch (`movzx eax, al;
/// mov [r14+0x90], eax; test al, al; jne`), whose scale multiplies the output
/// size.
pub const UPSCALER_SIZING: Site = Site {
    at: 0x28ef69a,
    what: "the FidelityFX path's render size selection",
    bytes: &[0x0f, 0xb6, 0xc0, 0x41, 0x89, 0x86, 0x90, 0, 0, 0, 0x84, 0xc0, 0x75, 0x0a],
};

/// What goes at [`UPSCALER_SIZING`] for `mode`: `mov eax, mode; mov [r14+0x90],
/// eax; jmp` to the scaled branch.
pub fn sizing_code(mode: Mode) -> Vec<u8> {
    let mut c = vec![0xb8, mode.ffx_mode(), 0, 0, 0]; // mov eax, mode
    c.extend([0x41, 0x89, 0x86, 0x90, 0, 0, 0]); // mov [r14+0x90], eax
    c.extend([0xeb, 0x0a]); // jmp scaled
    c
}

/// The native branch's load of 1.0: `vmovss xmm0, [rip+disp]; jmp`.
pub const NATIVE_LOAD: Site = Site {
    at: 0x28ef6a8,
    what: "the upscaler path's native scale load",
    bytes: &[0xc5, 0xfa, 0x10, 0x05, 0x28, 0x78, 0x6d, 0x03, 0xeb, 0x08],
};

/// The scaled branch's load of the scale: `vmovss xmm0, [rip+disp]`.
pub const SCALED_LOAD: Site = Site {
    at: 0x28ef6b2,
    what: "the upscaler path's scaled load",
    bytes: &[0xc5, 0xfa, 0x10, 0x05, 0xa2, 0xe2, 0x86, 0x03],
};

/// The scale the scaled branch loads (shared with other code: not written).
pub const UPSCALER_SCALE: Site = Site {
    at: 0x615d95c,
    what: "the upscaler path's scale constant",
    bytes: &[0xab, 0xaa, 0x2a, 0x3f], // 2/3
};

/// Patches the image for the title's FidelityFX path at `mode` (see the module
/// documentation). `None`: the sites are not the ones expected (said which).
///
/// # Safety
/// The image of `size` bytes at `base` stays mapped and no thread runs its
/// code yet.
#[cfg(windows)]
pub unsafe fn apply(base: usize, size: usize, mode: Mode) -> Option<()> {
    let sites: [&Site; 5] = [
        &RENDERER_SELECTION,
        &UPSCALER_SIZING,
        &NATIVE_LOAD,
        &SCALED_LOAD,
        &UPSCALER_SCALE,
    ];
    // Every site first: a build that differs anywhere is left alone, not
    // patched in part.
    for site in sites {
        // SAFETY: the caller's promise.
        let found = unsafe { matches(base, size, site) };
        if found != Patched::Done {
            tracing::warn!(
                site = %format!("{:#x}", site.at),
                what = site.what,
                "upscaler: the title's bytes here are not the ones expected: left alone"
            );
            println!(
                "upscaler: {} at title+{:#x}: not the stock bytes, left alone",
                site.what, site.at
            );
            return None;
        }
    }
    // The two loads are one chain: the native branch's literal becomes the
    // mode's scale and the scaled branch loads it.
    let mut native: Vec<u8> = NATIVE_LOAD.bytes.to_vec();
    let mut scaled: Vec<u8> = SCALED_LOAD.bytes.to_vec();
    if mode != Mode::Quality {
        let literal = NATIVE_LOAD.at + 4;
        native[4..8].copy_from_slice(&mode.scale().to_le_bytes());
        let displacement = (literal as i64 - (SCALED_LOAD.at as i64 + 8)) as i32;
        scaled[4..8].copy_from_slice(&displacement.to_le_bytes());
    }
    let asked: Vec<(&Site, Vec<u8>)> = vec![
        (&RENDERER_SELECTION, renderer_code()),
        (&UPSCALER_SIZING, sizing_code(mode)),
        (&NATIVE_LOAD, native),
        (&SCALED_LOAD, scaled),
    ];
    let mut done = 0;
    for (site, code) in asked {
        // SAFETY: the caller's promise; the bytes were just checked above.
        let wrote = unsafe { write(base, size, site, &code) };
        let offset = format!("{:#x}", site.at);
        match wrote {
            Patched::Done => {
                done += 1;
                tracing::info!(site = %offset, what = site.what, "upscaler: patched")
            }
            other => tracing::warn!(
                site = %offset,
                what = site.what,
                "upscaler: not patched: {other:?}"
            ),
        }
        println!("upscaler: {} at title+{offset}: {wrote:?}", site.what);
    }
    (done == 4).then_some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_modes_are_the_ffx_ones() {
        for (name, mode, ffx, scale) in [
            ("dlaa", Mode::Dlaa, 0, 1.0),
            ("quality", Mode::Quality, 1, 2.0 / 3.0),
            ("balanced", Mode::Balanced, 2, 0.58),
            ("performance", Mode::Performance, 3, 0.5),
            ("ultraperformance", Mode::UltraPerformance, 4, 1.0 / 3.0),
            ("ultra_performance", Mode::UltraPerformance, 4, 1.0 / 3.0),
            ("native", Mode::Dlaa, 0, 1.0),
        ] {
            assert_eq!(Mode::from_name(name), Some(mode), "{name}");
            assert_eq!(mode.ffx_mode(), ffx, "{name}");
            assert_eq!(mode.scale(), scale, "{name}");
            assert_eq!(Mode::from_name(mode.name()), Some(mode), "{name}");
        }
        assert_eq!(Mode::from_name("4k"), None);
        assert_eq!(Mode::from_name(""), None);
    }

    /// The patched instructions keep their shapes: renderer 7 with the console
    /// check's flags neutralised, the mode written to the quality field and
    /// the jump to the scaled branch; the native load's literal becomes the
    /// scale and the scaled load's displacement points at it.
    #[test]
    fn the_upscaler_code_keeps_the_titles_shapes() {
        assert_eq!(
            RENDERER_SELECTION.bytes,
            [0xb8, 6, 0, 0, 0, 0x0f, 0x45, 0xc1],
            "stock Series X picks its own temporal pass (6)"
        );
        let renderer = renderer_code();
        assert_eq!(&renderer[..5], [0xb8, 7, 0, 0, 0], "renderer 7 every time");
        assert_eq!(&renderer[5..], [0x90, 0x90, 0x90], "no cmovne left");
        let sizing = sizing_code(Mode::Balanced);
        assert_eq!(&sizing[..5], [0xb8, 2, 0, 0, 0]);
        assert_eq!(&sizing[5..12], [0x41, 0x89, 0x86, 0x90, 0, 0, 0]);
        assert_eq!(&sizing[12..], [0xeb, 0x0a], "jmp the scaled branch");
        assert_eq!(UPSCALER_SIZING.bytes.len(), 14);
        assert_eq!(sizing.len(), UPSCALER_SIZING.bytes.len());
        // The native branch's literal (NATIVE_LOAD+4) is what the scaled
        // branch then loads: SCALED_LOAD+8+disp == NATIVE_LOAD+4.
        let literal = NATIVE_LOAD.at + 4;
        let displacement = (literal as i64 - (SCALED_LOAD.at as i64 + 8)) as i32;
        let mut native = NATIVE_LOAD.bytes.to_vec();
        native[4..8].copy_from_slice(&Mode::Performance.scale().to_le_bytes());
        let mut scaled = SCALED_LOAD.bytes.to_vec();
        scaled[4..8].copy_from_slice(&displacement.to_le_bytes());
        let loaded = (SCALED_LOAD.at + 8) as i64
            + i64::from(i32::from_le_bytes(scaled[4..8].try_into().unwrap()));
        assert_eq!(loaded as usize, literal);
        assert_eq!(
            f32::from_le_bytes(native[4..8].try_into().unwrap()),
            Mode::Performance.scale()
        );
        // The shared constant (2/3) is not written by the quality mode.
        assert_eq!(UPSCALER_SCALE.bytes, (2.0f32 / 3.0).to_le_bytes());
    }
}
