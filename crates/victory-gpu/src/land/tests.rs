//! Tests for [`crate::land`].
//!
//! Port of `crates/victory-gpu/src/land/tests.rs` from
//! `trustno1-coder/VIctory-NVIDIA` (GPL-3.0-or-later), commit `590af7c`,
//! brought over at their `main` (`d12eb1b`); adapted to this tree: a title
//! buffer is plain committed memory here (no `crate::inplace` view of a
//! section to register), and the tests of the beside-the-guards path, of
//! `watch::again_after_change` waiting, of `watch::writes`/notes and the
//! ignored measurement of the foreign tree are left out (the features are
//! not in this tree; see the notes file).
//!
//! The test of the landed-beside notes ([`Notes::flush`]'s one asking for
//! the job's blocks) is a port of the same from that tree, read at
//! `d12eb1b`, functions `watch.rs:1852-1934`, `shadow.rs:4181-4256`,
//! `land.rs:481-548`; adapted to this tree: whole 64 KiB blocks (this
//! tree's journal unit) instead of 4 KiB pages and parts, and a
//! write-watched allocation made here (the runtime's `xmem` makes the
//! title's).

use super::*;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, PAGE_READWRITE, VirtualAlloc, VirtualFree,
};

/// As the runtime makes the title memory [`crate::watch`] asks the kernel
/// about: write-watched pages.
const MEM_WRITE_WATCH: u32 = 0x20_0000;

struct Random(u64);

impl Random {
    fn next(&mut self) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 8) as usize
    }
}

/// Committed memory of ours, as the runtime makes the title's: 64 KiB
/// aligned, freed when dropped.
struct Pages(*mut u8, usize);

impl Pages {
    fn new(len: usize) -> Pages {
        // SAFETY: plain allocation.
        let at = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                len,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            )
        };
        assert!(!at.is_null());
        Pages(at.cast(), len)
    }

    /// As the runtime makes the title's: committed with write watching, so
    /// that [`crate::watch`] can ask the kernel about its pages (a landing
    /// resets the bits of what it wrote, [`block_reset`]).
    fn watched(len: usize) -> Pages {
        // SAFETY: plain allocation.
        let at = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                len,
                MEM_RESERVE | MEM_COMMIT | MEM_WRITE_WATCH,
                PAGE_READWRITE,
            )
        };
        assert!(!at.is_null());
        Pages(at.cast(), len)
    }

    fn bytes(&self) -> &[u8] {
        // SAFETY: the allocation.
        unsafe { std::slice::from_raw_parts(self.0, self.1) }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: the allocation, and the only reference.
        unsafe { std::slice::from_raw_parts_mut(self.0, self.1) }
    }
}

impl Drop for Pages {
    fn drop(&mut self) {
        // SAFETY: our allocation.
        unsafe { VirtualFree(self.0.cast(), 0, MEM_RELEASE) };
    }
}

/// Every number once, whatever the count and the threads, batch after batch
/// and from several callers.
#[test]
fn the_pool_calls_every_number_once() {
    for threads in [1usize, 2, 4, 7] {
        let pool = Pool::new("victory-land", threads);
        std::thread::scope(|s| {
            for _ in 0..3 {
                s.spawn(|| {
                    for round in 0..300usize {
                        let count = round * 7 % 50;
                        let calls: Vec<AtomicUsize> =
                            (0..count).map(|_| AtomicUsize::new(0)).collect();
                        pool.each(count, &|i| {
                            calls[i].fetch_add(1, Ordering::Relaxed);
                        });
                        assert!(calls.iter().all(|c| c.load(Ordering::Relaxed) == 1));
                    }
                });
            }
        });
    }
}

/// A job that panics is told to the caller, and the pool goes on.
#[test]
fn a_panicking_job_reaches_the_caller() {
    let pool = Pool::new("victory-land", 3);
    for bad in 0..8usize {
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pool.each(8, &|i| assert_ne!(i, bad, "(expected)"));
        }));
        assert!(ran.is_err());
        let n = AtomicUsize::new(0);
        pool.each(20, &|i| {
            n.fetch_add(i, Ordering::Relaxed);
        });
        assert_eq!(n.into_inner(), 190);
    }
}

/// The order pieces are landed in holds every byte of every piece, under its
/// copy, and no other; blocks come in order, pieces of several copies in one
/// block stay in the order given, and jobs never cut a block.
#[test]
fn ordered_pieces_hold_the_same_bytes() {
    let mut random = Random(99);
    for _ in 0..400 {
        let count = random.next() % 40;
        let pieces: Vec<Piece> = (0..count)
            .map(|_| {
                let block = random.next() % 12 * BLOCK;
                let from = random.next() % BLOCK / 4 * 4;
                let to = (from + 4 + random.next() % BLOCK / 4 * 4).min(BLOCK);
                (random.next() as u64 % 3, block + from, to - from)
            })
            .collect();
        let out = ordered(&pieces);
        let bytes = |copy: u64, at: usize, all: &[Piece]| {
            all.iter()
                .any(|p| p.0 == copy && p.1 <= at && at < p.1 + p.2)
        };
        let landed: Vec<Piece> = out
            .iter()
            .map(|&(i, at, len)| (pieces[i].0, at, len))
            .collect();
        for copy in 0..3 {
            for at in (0..12 * BLOCK).step_by(1024 * 4) {
                assert_eq!(
                    bytes(copy, at, &pieces),
                    bytes(copy, at, &landed),
                    "{pieces:?}"
                );
            }
        }
        assert!(
            out.iter()
                .all(|&(_, at, len)| len > 0 && at / BLOCK == (at + len - 1) / BLOCK)
        );
        assert!(out.windows(2).all(|w| w[0].1 / BLOCK <= w[1].1 / BLOCK));
        for block in 0..12 {
            let given: Vec<Piece> = pieces
                .iter()
                .copied()
                .filter(|p| p.1 / BLOCK == block)
                .collect();
            let here: Vec<Piece> = landed
                .iter()
                .copied()
                .filter(|p| p.1 / BLOCK == block)
                .collect();
            if given.iter().any(|p| p.0 != given[0].0) {
                assert_eq!(here, given);
            } else {
                assert!(
                    here.windows(2).all(|w| w[0].1 + w[0].2 < w[1].1),
                    "apart, in order"
                );
            }
        }
        let cut: Vec<(usize, usize)> = out.iter().map(|&(_, at, len)| (at, len)).collect();
        let jobs = jobs(&cut);
        assert_eq!(
            jobs.iter().cloned().flatten().collect::<Vec<_>>(),
            (0..cut.len()).collect::<Vec<_>>()
        );
        for w in jobs.windows(2) {
            assert_ne!(cut[w[0].end - 1].0 / BLOCK, cut[w[1].start].0 / BLOCK);
        }
    }
}

/// A piece is left out only if a fetch behind its own flight, or behind a
/// later one, brought all of it, and is forgotten when that flight is done.
#[test]
fn only_what_a_fetch_behind_the_flight_brought_is_left_out() {
    let mut had = Had::default();
    had.fetched(7, [(1, BLOCK, BLOCK), (2, 0x100, 0x200)].into_iter());
    assert!(had.holds(6, (1, BLOCK, BLOCK)) && had.holds(7, (1, BLOCK + 64, 128)));
    assert!(
        !had.holds(8, (1, BLOCK, BLOCK)),
        "the fetch ran ahead of flight 8"
    );
    assert!(!had.holds(7, (3, BLOCK, BLOCK)) && !had.holds(7, (1, 0, 64)));
    assert!(
        had.holds(7, (2, 0x100, 0x200))
            && !had.holds(7, (2, 0x100, 0x204))
            && !had.holds(7, (2, 0xfc, 8))
    );
    had.landing(7);
    assert!(had.holds(7, (1, BLOCK, BLOCK)));
    had.landing(8);
    assert!(had.0.is_empty());
}

/// A piece landed through the title's own address holds what the merge of
/// one thread and one pass ([`crate::safecopy::merge`]) is to leave, with
/// parts left out, wherever it begins and ends.
#[test]
fn a_piece_lands_what_the_card_wrote_in_its_parts() {
    assert!(crate::safecopy::on());
    let mut pages = Pages::new(2 * BLOCK);
    let title = pages.0 as usize;
    let mut random = Random(5);
    for round in 0..300usize {
        let from = (random.next() % 5000) & !3;
        let len = (random.next() % BLOCK).min(BLOCK - from).max(1);
        let base: Vec<u8> = (0..len).map(|_| random.next() as u8).collect();
        let (mut now, mut held) = (base.clone(), base.clone());
        for _ in 0..random.next() % 60 {
            let at = random.next() % len;
            let n = (random.next() % 300).min(len - at);
            for b in &mut now[at..at + n] {
                *b = random.next() as u8;
            }
        }
        let mut there = base.clone();
        for _ in 0..random.next() % 60 {
            there[random.next() % len] ^= 0x11;
        }
        // Left out: some ranges, anywhere.
        let mut parts = vec![(0, len)];
        for _ in 0..round % 4 {
            let a = random.next() % len;
            let b = (a + random.next() % 200).min(len);
            parts = parts
                .into_iter()
                .flat_map(|(x, y)| [(x, y.min(a)), (x.max(b), y)])
                .filter(|(x, y)| x < y)
                .collect();
        }
        // The title's bytes of the piece, put in as a refresh would.
        pages.bytes_mut()[from..from + len].copy_from_slice(&there);
        let mut notes = Notes::default();
        // SAFETY: vectors of `len` bytes, and the piece inside the allocation.
        let put = unsafe {
            piece(
                title + from,
                now.as_ptr(),
                held.as_mut_ptr(),
                &parts,
                true,
                &mut notes,
                std::ptr::null(),
            )
        };
        notes.flush();
        let mut expect_held = base.clone();
        let mut expect = there.clone();
        for &(a, b) in &parts {
            for i in a..b {
                if now[i] != base[i] {
                    expect[i] = now[i];
                    expect_held[i] = now[i];
                }
            }
        }
        assert!(held == expect_held, "round {round}");
        assert_eq!(
            &pages.bytes()[from..from + len],
            &expect[..],
            "round {round}"
        );
        // A dword is stored once, and only where it changes what is there,
        // so `put` is the dwords stored times four, never more than the
        // stretch.
        assert!(put <= len as u64, "round {round}: {put} bytes put of {len}");
    }
}

/// A piece whose title memory is not there puts nothing, and `held` takes
/// the card's bytes only if `take` says so; what was merged before the
/// missing page stays merged.
#[test]
fn a_piece_into_missing_memory_takes_or_leaves() {
    assert!(crate::safecopy::on());
    // SAFETY: a reservation of ours, nothing of it committed.
    let at = unsafe { VirtualAlloc(std::ptr::null(), BLOCK, MEM_RESERVE, PAGE_NOACCESS) } as usize;
    assert_ne!(at, 0);
    let now = [9u8; 100];
    for take in [true, false] {
        let mut held = [0u8; 100];
        let mut notes = Notes::default();
        // SAFETY: arrays of 100 bytes; the title's address is not committed.
        let put = unsafe {
            piece(
                at,
                now.as_ptr(),
                held.as_mut_ptr(),
                &[(0, 100)],
                take,
                &mut notes,
                std::ptr::null(),
            )
        };
        assert_eq!((put, held), (0, [if take { 9 } else { 0 }; 100]));
    }
    // SAFETY: our reservation.
    unsafe { VirtualFree(at as _, 0, MEM_RELEASE) };
}

/// Landings on several threads beside title threads that write the same
/// blocks (other dwords) lose nothing: no write of the title's is undone and
/// all the card wrote arrives.
#[test]
fn landings_beside_title_writes_lose_nothing() {
    assert!(crate::safecopy::on());
    const BLOCKS: usize = 12;
    const LEN: usize = BLOCKS * BLOCK;
    let pages = Pages::new(LEN);
    let at = pages.0 as usize;
    // SAFETY: committed, aligned, touched as atomics or by the landing's
    // stores of whole dwords.
    let dword = |i: usize| unsafe { &*((at + i * 4) as *const AtomicU32) };
    let stop = AtomicBool::new(false);
    let mut held = vec![0u8; LEN];
    std::thread::scope(|s| {
        let writers: Vec<_> = (0..3usize)
            .map(|w| {
                let stop = &stop;
                s.spawn(move || {
                    let mut value = 1u32;
                    while !stop.load(Ordering::Relaxed) {
                        // Its own dwords, in every other block.
                        for i in (w..LEN / 4)
                            .step_by(4 * 97)
                            .filter(|i| i * 4 / BLOCK % 2 == 0)
                        {
                            assert_eq!(
                                dword(i).swap(value, Ordering::Relaxed),
                                value - 1,
                                "dword {i}: undone"
                            );
                        }
                        value += 1;
                    }
                    value - 1
                })
            })
            .collect();
        let mut random = Random(77);
        let pieces: Vec<Piece> = (0..BLOCKS).map(|b| (1, at + b * BLOCK, BLOCK)).collect();
        let order = ordered(&pieces);
        let cut: Vec<(usize, usize)> = order.iter().map(|&(_, at, len)| (at, len)).collect();
        let jobs = jobs(&cut);
        let into = held.as_mut_ptr() as usize;
        for round in 1..250u32 {
            // The card's dwords: every fourth, from the third on, in some
            // pages of some blocks.
            let mut now = held.clone();
            for _ in 0..60 {
                let page = random.next() % (LEN / PAGE);
                for i in (page * 1024 + 3..(page + 1) * 1024).step_by(4) {
                    now[i * 4..i * 4 + 4].copy_from_slice(&(round * 4096 + i as u32).to_le_bytes());
                }
            }
            each(jobs.len(), &|j| {
                for &(address, len) in &cut[jobs[j].clone()] {
                    let offset = address - at;
                    let mut notes = Notes::default();
                    // SAFETY: each job its own pieces of the two vectors.
                    unsafe {
                        piece(
                            address,
                            now.as_ptr().add(offset),
                            (into + offset) as *mut u8,
                            &[(0, len)],
                            true,
                            &mut notes,
                            std::ptr::null(),
                        );
                    }
                    notes.flush();
                }
            });
            assert!(held == now, "round {round}");
            for i in (3..LEN / 4).step_by(4 * 61) {
                let expect = u32::from_le_bytes(now[i * 4..i * 4 + 4].try_into().expect("four"));
                assert_eq!(
                    dword(i).load(Ordering::Relaxed),
                    expect,
                    "round {round} dword {i}"
                );
            }
        }
        stop.store(true, Ordering::Relaxed);
        for (w, writer) in writers.into_iter().enumerate() {
            let last = writer.join().expect("nothing undone");
            for i in (w..LEN / 4)
                .step_by(4 * 97)
                .filter(|i| i * 4 / BLOCK % 2 == 0)
            {
                assert_eq!(dword(i).load(Ordering::Relaxed), last);
            }
        }
    });
}

/// [`Notes::flush`] makes one asking for the whole job: the journal's
/// `asked` moves by one, and every block the job noted carries that one
/// asking's number.
///
/// Port of the "one asking for the job's blocks" part of the notes tests of
/// `trustno1-coder/VIctory-NVIDIA` (GPL-3.0-or-later), read at `d12eb1b`;
/// adapted to this tree: whole 64 KiB blocks, with no 4 KiB parts.
#[test]
fn a_jobs_blocks_are_noted_with_one_asking() {
    const BLOCKS: usize = 3;
    let pages = Pages::watched(BLOCKS * BLOCK);
    let at = pages.0 as usize;
    crate::watch::forget_for_test(at, BLOCKS * BLOCK);
    let base = at / BLOCK;
    let mut notes = Notes::default();
    for block in base..base + BLOCKS {
        notes.pages.push(block);
        // No copy: what is under test is the asking the note is made at.
        notes.landed.push((block, std::ptr::null()));
    }
    let before = crate::watch::asked_for_test();
    notes.flush();
    assert_eq!(
        crate::watch::asked_for_test(),
        before + 1,
        "the job's blocks took more than one asking"
    );
    // After the note, an asking about the range reports every block at the
    // note's one number.
    let (now, written) = crate::watch::writes(at, at + BLOCKS * BLOCK, true).expect("write-watched");
    assert_eq!(written.len(), BLOCKS, "not every block was noted: {written:?}");
    assert!(
        written.iter().all(|&(block, asking)| (base..base + BLOCKS).contains(&block) && asking < now),
        "a block was not noted at this asking: {written:?}"
    );
    assert!(
        written.iter().all(|&(_, asking)| asking == written[0].1),
        "the blocks were noted at more than one asking: {written:?}"
    );
    assert!(notes.is_empty_for_test(), "the notes were not cleared");
    crate::watch::forget_for_test(at, BLOCKS * BLOCK);
}
