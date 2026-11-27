//! Lower the wave-wide allocation of two transmission tile lists on wave32.
//!
//! The title's transmission-tile program (`CS_TransmissionVolumetrics_Tiled`'s
//! allocator) takes the console's 64-lane wave for granted: each lane reserves
//! one slot in its front or back list, and the two classes come from the
//! incoming EXEC masks. On a host of 32-lane waves one wave-wide run of a
//! 64-lane program runs twice, so the allocation is taken once for each half
//! and the slot a lane reserves is the wrong one: whole tiles of open water
//! are left dark (around boats, along shores, under bridges) — exactly the
//! dark blocks `VICTORY_WATER_LIGHT=1` shows in our runs.
//!
//! The console program is patched before it is lowered: the `v_mov_b32` that
//! starts the slot count becomes inline constant 1, so every lane reserves a
//! slot of its own, and the instructions that allocate (address, atomic, wait)
//! are kept after it; the rest of the body is filled with the program's own
//! `s_nop`. The masks, counters, branch targets and index reversal stay.
//!
//! Ported from `trustno1-coder/VIctory-NVIDIA` PR #45 (files
//! `crates/victory-shader/src/emit/water_tile_append.rs` and the hook in
//! `emit/mod.rs`), GPL-3.0-or-later; their verification: wave32 output
//! byte-identical to the correction they measured on an RTX 5070 Ti, wave64
//! output unchanged.
//!
//! Not ported from them: the automatic name/hash discovery of the program —
//! this works off the container digest and the program's own fingerprint, both
//! checked before anything is written, and a mismatch is refused (the shader
//! then keeps its old lowering, never a half-written one).

use std::ops::Range;

use crate::native::{HwStage, NativeProgram};
use crate::ShaderError;

/// The container this patch knows (its digest, the 16 bytes a container
/// carries at its DXIL part): the transmission-tile allocator of the retail
/// v1.0.2.1 dump.
pub(super) const CONTAINER_ID: [u8; 16] = [
    0xf5, 0xd1, 0x0c, 0xde, 0x74, 0x33, 0x37, 0x82, 0xf3, 0x0f, 0x9b, 0xb5, 0x56, 0xc0, 0x73, 0xd4,
];
/// FNV-1a of the plain program body (2,112 bytes) this patch was made for.
const PROGRAM_ID: u64 = 0x377a_8d97_4fb8_631f;
/// The program's size in bytes, before anything is written.
const PROGRAM_BYTES: usize = 2112;

/// FNV-1a of a console program's code, the identity [`rewrite`] checks.
fn fingerprint(code: &[u8]) -> u64 {
    code.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Whether `native` is the program [`CONTAINER_ID`] names, and if so, lower
/// its two tile-list allocations for a host of 32-lane waves.
pub(super) fn rewrite(native: &mut NativeProgram) -> Result<(), ShaderError> {
    if native.stage != HwStage::Cs || !native.wave64 || native.code.len() != PROGRAM_BYTES {
        return Err(ShaderError::Malformed("unexpected transmission tile program"));
    }
    let code = &mut native.code;
    if fingerprint(code) != PROGRAM_ID {
        return Err(ShaderError::Malformed("unexpected transmission tile program body"));
    }

    // The incoming EXEC masks already select the two disjoint classes. Keep
    // those masks and the original counters, but reserve one slot per lane.
    // The original v_mov_b32 source is inline constant 0; its low bit makes
    // it inline constant 1. Keep every address, atomic, and wait from the
    // input program. Its branch targets and index reversal stay in place.
    let mut unit_move: [u8; 4] = code[0x6c4..0x6c8].try_into().unwrap();
    unit_move[0] |= 1;
    let nop: [u8; 4] = code[0x6d8..0x6dc].try_into().unwrap();
    inline_append(code, 0x6a8..0x704, unit_move, nop, &[0x6dc..0x6e4, 0x6f0..0x6f4]);
    inline_append(
        code,
        0x70c..0x774,
        unit_move,
        nop,
        &[0x740..0x748, 0x74c..0x754, 0x75c..0x760, 0x770..0x774],
    );
    Ok(())
}

/// Puts `unit_move` first in `body`, then the `kept` ranges in order, and
/// fills what is left with `nop` — a run of instructions appended in front of
/// the body's own ones, which have already allocated lazily.
fn inline_append(code: &mut [u8], body: Range<usize>, unit_move: [u8; 4], nop: [u8; 4], kept: &[Range<usize>]) {
    let mut append = Vec::with_capacity(body.len());
    append.extend_from_slice(&unit_move);
    for range in kept {
        append.extend_from_slice(&code[range.clone()]);
    }
    let written = append.len();
    let body = &mut code[body];
    body[..written].copy_from_slice(&append);
    for slot in body[written..].chunks_exact_mut(4) {
        slot.copy_from_slice(&nop);
    }
}

#[cfg(test)]
mod tests {
    /// Nothing is written before the program is identified: a program of
    /// another size, stage, wave or body is refused.
    #[test]
    fn another_program_is_refused() {
        use super::*;
        use crate::native::StageIo;
        let mut program = NativeProgram {
            stage: HwStage::Cs,
            wave64: true,
            vgprs: 0,
            layout: Vec::new(),
            used: Vec::new(),
            io: StageIo::None,
            code: vec![0u8; PROGRAM_BYTES],
        };
        assert!(rewrite(&mut program).is_err(), "an all-zero body is not the program");
        program.code.pop();
        assert!(rewrite(&mut program).is_err(), "another size is not the program");
    }

    /// Diagnostic (`VICTORY_WATER_PROBE_DIR` set to a directory of containers
    /// dumped by `translate`'s `VICTORY_DUMP_MATCH`): print each container's
    /// digest and every compute program's identity, so a byte patch can be
    /// checked against all the containers that carry a water program.
    /// `cargo test -p victory-shader --lib probe_water_containers -- --nocapture`
    #[test]
    fn probe_water_containers() {
        use super::*;
        let Some(dir) = std::env::var_os("VICTORY_WATER_PROBE_DIR") else {
            return;
        };
        for entry in std::fs::read_dir(&dir).expect("the probe directory") {
            let path = entry.expect("an entry").path();
            let bytes = std::fs::read(&path).expect("a container");
            let Ok(container) = crate::dxbc::Container::parse(&bytes) else {
                eprintln!("{}: not a container", path.display());
                continue;
            };
            let digest: String = container.digest.iter().map(|byte| format!("{byte:02x}")).collect();
            let Some(dxil) = container.part(crate::dxbc::FourCC::DXIL) else {
                eprintln!("{} digest={digest}: no DXIL part", path.display());
                continue;
            };
            let Ok(programs) = crate::native_programs(dxil.data) else {
                continue;
            };
            for p in programs.iter().filter(|p| p.stage == HwStage::Cs) {
                eprintln!(
                    "{} digest={digest} wave64={} len={} fnv={:016x} patch_digest={}",
                    path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                    p.wave64,
                    p.code.len(),
                    fingerprint(&p.code),
                    container.digest == CONTAINER_ID
                );
            }
        }
    }
}
