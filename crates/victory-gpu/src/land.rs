//! What shaders wrote goes into title memory on a few threads
//! (`VICTORY_LAND=1`, **off by default in this tree**: see [`on`] and
//! HANDOFF 42; unset or `0` merges piece by piece on the thread that lands
//! flights, as before).
//!
//! The relay's thread (`crate::host::relay`) was 58-60 ms of a 104 ms frame
//! landing (run record-1): some 1,500 pieces of 64 KiB, 98 MB, fetched from
//! copies in the card's memory, and each of them searched from both ends,
//! cleared room for, read from title memory, copied, merged and compared
//! again while it was put, one after the other; a lock and a protection
//! call to open the guard of each, and the kernel's fault on the first
//! write to each 4 KiB page of memory just made writable again.
//!
//! Here a piece is merged in one pass ([`crate::safecopy::merge`]): what
//! the card holds is compared with what it held 32 bytes at a time, title
//! memory is read only where they differ, just before the dwords that
//! change are stored. The pieces of a landing are cut into jobs for a few
//! threads kept for it ([`Pool`]).
//!
//! What stays in order: pieces in the same 64 KiB of title memory are in
//! one job, in the order they were given (two copies may hold the same
//! title memory); a landing's copies in the card's memory come before its
//! other copies; a landing is done, whole, before the title is told
//! anything and before the next one starts (`Host::relay_land`).
//!
//! Port of `crates/victory-gpu/src/land.rs` from
//! `trustno1-coder/VIctory-NVIDIA` (GPL-3.0-or-later), commit `590af7c`
//! (the file), brought over at their `main` (`d12eb1b`); adapted to this
//! tree: the beside-the-guards path is gone (no `crate::inplace::beside`
//! here), a piece is only ever put through the title's own address after
//! `crate::watch::writing` opened its guards; a page of title memory that
//! is not there is counted and left to the next landing rather than waited
//! for (`crate::watch::again_after_change` is not ported); no timeline
//! spans and no fast-core priority pinning.
//!
//! The landed-beside bookkeeping is ported too ([`taken`], [`Notes`],
//! [`crate::shadow::Shadow::landed`]). Because this tree's write goes
//! through the title's own address, the kernel's write watch sees the
//! landing's stores as the title's: so a block is noted only if
//! [`block_clean`] found it clean before the merge, its bits are reset after
//! it ([`block_reset`]), and the copy the landing went through is told it
//! holds the block. Noted after the bytes are in and after the reset, never
//! before. A title write to one of the block's pages between the check and
//! the reset is lost to the write watch: the same accepted hole
//! `Shadow::put_back_quiet` names, and `VICTORY_LAND_TAKEN=0` restores the
//! Round-19 behaviour.
//!
//! Port of the landed-beside bookkeeping from `trustno1-coder/VIctory-NVIDIA`
//! (GPL-3.0-or-later), read at `d12eb1b`; functions `watch.rs:1852-1934`,
//! `shadow.rs:4181-4256`, `land.rs:481-548`; adapted to this tree: whole
//! 64 KiB blocks instead of 4 KiB pages and parts (the journal's unit), no
//! `LANDED` record (nothing here reads one), no alias pages (`elsewhere`),
//! and the write-watch reset stands in for `watch::block_published`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock};

/// The unit pieces are cut in (of title memory), and the guards' blocks.
pub(crate) const BLOCK: usize = 64 << 10;
/// The unit title memory is there or not in.
const PAGE: usize = 4096;
/// About this many bytes to a job.
const JOB: usize = 256 << 10;

/// See the file. `VICTORY_LAND=1` opts in; **off by default in this tree**:
/// Round 19 measured the port as it stands (r19-run1 against r19-runA, the
/// same binary and the same scene) and the landing itself got faster (the
/// frame line's "copying back" 67.4 -> 47.0 ms) while the submitting
/// thread's "copies" rose 33.8 -> 51.9 ms and "preparing" 84.2 -> 107.2 ms:
/// the frame got slower (105.5 -> 117.4 ms, 9.50 -> 8.50 fps). A piece put
/// in by `safecopy` writes title memory directly, without the landed-beside
/// notes and the copies' taken pages that this tree's `put_back_quiet` and
/// `put_back_own` keep (the tree it came from has `watch::landed_beside`,
/// its `LANDED` bits and `Shadow::landed`; they are the part not ported, see
/// the file's note), so the next refreshes take the landed blocks again.
/// Use it for its own A/B (`VICTORY_LAND=1 VICTORY_LAND_THREADS=<n>`), not
/// in the locked profile, until those notes are ported.
pub(crate) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var_os("VICTORY_LAND").is_some_and(|v| v == "1")
            && crate::safecopy::on()
            && crate::shadow::exact_back()
    })
}

/// How many threads land, the relay's own counted (`VICTORY_LAND_THREADS`).
fn threads() -> usize {
    static THREADS: OnceLock<usize> = OnceLock::new();
    *THREADS.get_or_init(|| {
        let asked = std::env::var("VICTORY_LAND_THREADS")
            .ok()
            .and_then(|v| v.trim().parse().ok());
        asked
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map_or(2, |n| n.get() / 4)
                    .clamp(1, 4)
            })
            .clamp(1, 16)
    })
}

/// Whether pieces an earlier fetch holds as they are now are left out
/// ([`Had`]; `VICTORY_LAND_SKIP=1`, off by default: counted either way).
pub(crate) fn skip_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_LAND_SKIP").is_some_and(|v| v == "1"))
}

/// Whether what a landing put into title memory is noted at the end of its
/// job ([`Notes::flush`], [`crate::watch::landed_blocks`]) and the copy it
/// went through is told it holds it ([`crate::shadow::Shadow::landed`]);
/// `VICTORY_LAND_TAKEN=0` turns both off, and then nothing of the kernel's
/// write watch is asked or reset either: the landing copies and sends its
/// blocks again exactly as before this was ported.
///
/// What a landing puts into title memory the copy holds already: the card
/// wrote it there, and a copy in the card's memory was given it on the CPU's
/// side by the same merge. But through the title's own address the kernel
/// counts the block as written, for that copy as for every other one, so the
/// next refresh copied the whole block, sent it up and had the card hash it
/// again. The landing notes it instead, and the copy it went through notes
/// itself current: only where one copy alone landed in the block (two would
/// each hold only their own bytes of it).
///
/// Why the write watch is asked first: this tree's landing stores through
/// the title's own address, so a note alone buys nothing — the kernel would
/// report the block with a number above the note's, and every copy, the one
/// that landed included, would take it again. [`block_clean`] before the
/// stores and [`block_reset`] after them are what keep the kernel silent
/// (the same pair `crate::shadow::put_back_quiet` uses, with the same
/// accepted hole: a title write between the check and the reset is lost to
/// the write watch).
pub(crate) fn taken() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_LAND_TAKEN").is_none_or(|v| v != "0"))
}

/// Whether a copy in the card's memory compares the pages a landing noted
/// with title memory and leaves them as they are where they are equal,
/// instead of copying them and sending them up again (`VICTORY_LAND_ECHO`,
/// on by default; nothing without [`taken`]).
pub(crate) fn echo_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| taken() && std::env::var_os("VICTORY_LAND_ECHO").is_none_or(|v| v != "0"))
}

/// Since the last frame-rate line; microseconds first.
pub(crate) mod stat {
    /// The pieces to fetch gathered and put in the order they are landed.
    pub(crate) const GATHER: usize = 0;
    /// Looking for what shaders wrote to copies in the PC's memory.
    pub(crate) const SCAN: usize = 1;
    /// Merging what was fetched from copies in the card's memory: until the
    /// last job is done, and the jobs' time on all threads.
    pub(crate) const CARD: usize = 2;
    pub(crate) const CARD_ALL: usize = 3;
    /// Merging what the scan found in copies in the PC's memory.
    pub(crate) const BACK: usize = 4;
    pub(crate) const BACK_ALL: usize = 5;
    /// In the jobs: waiting for the lock against the title's bytes going in.
    pub(crate) const LOCK: usize = 6;
    /// In the jobs: guards opened before a write, and the notes at the end
    /// of a job.
    pub(crate) const NOTES: usize = 7;
    /// The flights' ranges done with; the rest of the landing.
    pub(crate) const DONE: usize = 8;
    pub(crate) const READ_BACKS: usize = 9;
    pub(crate) const TOLD: usize = 10;
    pub(crate) const REST: usize = 11;
    /// Counts: pieces merged, those with a new byte, their 4 KiB pages,
    /// those with a new byte, blocks of 32 bytes with a new byte, dwords put,
    /// and 4 KiB pages of title memory that were not there.
    pub(crate) const PIECES: usize = 12;
    pub(crate) const NEW_PIECES: usize = 13;
    pub(crate) const PAGES: usize = 14;
    pub(crate) const NEW_PAGES: usize = 15;
    pub(crate) const NEW_BLOCKS: usize = 16;
    pub(crate) const DWORDS: usize = 17;
    pub(crate) const MISSING: usize = 18;
    /// Bytes asked for twice in a landing (fetched once); pieces and bytes an
    /// earlier fetch held already.
    pub(crate) const TWICE: usize = 19;
    pub(crate) const HAD: usize = 20;
    pub(crate) const HAD_BYTES: usize = 21;
    /// Jobs.
    pub(crate) const JOBS: usize = 22;
    /// Blocks of 64 KiB that went in with their write watch clean before the
    /// merge and reset after it (noted), and those it was not clean for (the
    /// title wrote there first, or the kernel could not answer: copied again
    /// by the kernel's word, not noted).
    pub(crate) const QUIET: usize = 23;
    pub(crate) const UNQUIET: usize = 24;
    /// Blocks noted in the journal at the end of their job, and those of them
    /// the copy that landed through was told it holds (and their bytes).
    pub(crate) const LANDED: usize = 25;
    pub(crate) const STAMPED: usize = 26;
    pub(crate) const STAMPED_BYTES: usize = 27;
    /// Bytes of landed pages compared in copies in the card's memory that
    /// were equal to the copy's side (nothing copied, nothing sent up), and
    /// those that differed and went up.
    pub(crate) const ECHO: usize = 28;
    pub(crate) const ECHO_SENT: usize = 29;
    pub(crate) const COUNT: usize = 30;
}

static STATS: [AtomicU64; stat::COUNT] = [const { AtomicU64::new(0) }; stat::COUNT];

pub(crate) fn count(which: usize, n: u64) {
    STATS[which].fetch_add(n, Ordering::Relaxed);
}

/// Adds the microseconds since `from` to `which`; now.
pub(crate) fn spent(which: usize, from: std::time::Instant) -> std::time::Instant {
    let now = std::time::Instant::now();
    count(which, (now - from).as_micros() as u64);
    now
}

/// For the frame-rate line: where landing went, per frame.
pub(crate) fn log(frames: f64) {
    // The timing counters of `Host::relay_land` are kept whichever way the
    // landing goes; with it off this prints nothing, so `VICTORY_LAND=0`
    // leaves the frame line exactly as it was.
    if !on() {
        return;
    }
    let s: Vec<f64> = STATS
        .iter()
        .map(|s| s.swap(0, Ordering::Relaxed) as f64 / frames)
        .collect();
    if s[stat::PIECES] == 0.0 && s[stat::REST] == 0.0 {
        return;
    }
    let ms = |i: usize| s[i] / 1000.0;
    let mb = |v: f64| v / (1 << 20) as f64;
    tracing::info!(
        "landing per frame (VICTORY_LAND={}, {} threads): gathering the pieces {:.2} ms; merging what was fetched from \
         copies in the card's memory {:.2} ms ({:.2} ms on all threads, {:.0} jobs), merging what the scan found in copies \
         in the PC's memory {:.2} ms ({:.2} on all threads); of the threads' time {:.2} ms waiting for the title's bytes \
         going in, {:.2} ms opening guards; then ranges done with {:.2} ms, read-backs {:.2}, the title told {:.2}, the rest {:.2}",
        u8::from(on()),
        threads(),
        ms(stat::GATHER),
        ms(stat::CARD),
        ms(stat::CARD_ALL),
        s[stat::JOBS],
        ms(stat::BACK),
        ms(stat::BACK_ALL),
        ms(stat::LOCK),
        ms(stat::NOTES),
        ms(stat::DONE),
        ms(stat::READ_BACKS),
        ms(stat::TOLD),
        ms(stat::REST),
    );
    tracing::info!(
        "landed per frame: {:.0} pieces, {:.0} with a new byte; of their {:.0} pages of 4 KiB {:.0} with a new byte \
         ({:.1} MB), {:.1} MB in blocks of 32 bytes with one, {:.2} MB put as dwords; {:.0} pages of title memory not \
         there; asked for twice in a landing {:.1} MB; held already by an earlier fetch {:.0} pieces {:.1} MB \
         (VICTORY_LAND_SKIP={})",
        s[stat::PIECES],
        s[stat::NEW_PIECES],
        s[stat::PAGES],
        s[stat::NEW_PAGES],
        mb(s[stat::NEW_PAGES] * PAGE as f64),
        mb(s[stat::NEW_BLOCKS] * 32.0),
        mb(s[stat::DWORDS] * 4.0),
        s[stat::MISSING],
        mb(s[stat::TWICE]),
        s[stat::HAD],
        mb(s[stat::HAD_BYTES]),
        u8::from(skip_on()),
    );
    // Only with the landed-beside notes on: otherwise every one of these is
    // 0 and the line says nothing (VICTORY_LAND_TAKEN=0 is the old arm).
    if taken() {
        tracing::info!(
            "landed beside the guards per frame (VICTORY_LAND_TAKEN={}): {:.0} blocks of 64 KiB went in with their write \
             watch clean and reset, {:.0} did not (the title wrote there first, or the kernel could not answer; copied \
             again by the kernel's word); of the noted {:.0} the copy that landed through is told it holds ({:.1} MB the \
             next refresh need not copy); copies in the card's memory compared landed pages: {:.1} MB equal to the copy's \
             side (not sent up), {:.1} MB different (sent)",
            u8::from(taken()),
            s[stat::QUIET],
            s[stat::UNQUIET],
            s[stat::LANDED],
            mb(s[stat::STAMPED_BYTES]),
            mb(s[stat::ECHO]),
            mb(s[stat::ECHO_SENT]),
        );
    }
}

/// Threads kept for landing, and the work they are at.
struct Pool {
    state: Mutex<State>,
    more: Condvar,
    idle: Condvar,
    /// One batch at a time.
    turn: Mutex<()>,
    threads: usize,
}

struct State {
    /// What to do with each number below `count`.
    work: Option<Work>,
    next: usize,
    count: usize,
    /// Helpers inside a call.
    running: usize,
    panicked: bool,
}

#[derive(Clone, Copy)]
struct Work(*const (dyn Fn(usize) + Sync));

// SAFETY: the closure may be called from any thread (`Sync`), and
// `Pool::each` keeps it alive until no helper is inside a call.
unsafe impl Send for Work {}

impl Pool {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn new(name: &'static str, threads: usize) -> &'static Pool {
        let pool: &'static Pool = Box::leak(Box::new(Pool {
            state: Mutex::new(State {
                work: None,
                next: 0,
                count: 0,
                running: 0,
                panicked: false,
            }),
            more: Condvar::new(),
            idle: Condvar::new(),
            turn: Mutex::new(()),
            threads,
        }));
        for i in 1..threads {
            // (A helper that is not there is only missed.)
            let _ = std::thread::Builder::new()
                .name(format!("{name}-{i}"))
                .spawn(move || pool.help());
        }
        pool
    }

    fn help(&self) {
        // (The thread that calls `each` waits for this one's share.)
        let mut state = self.state();
        loop {
            let Some((work, i)) = take(&mut state) else {
                state = self.more.wait(state).unwrap_or_else(|e| e.into_inner());
                continue;
            };
            state.running += 1;
            drop(state);
            // SAFETY: counted in `running`, so `each` has not returned.
            let ran =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { (*work.0)(i) }));
            state = self.state();
            state.running -= 1;
            state.panicked |= ran.is_err();
            if state.running == 0 {
                self.idle.notify_all();
            }
        }
    }

    /// Calls `f` with every number below `count`, on the pool's threads and
    /// this one, and returns when every call has.
    fn each(&self, count: usize, f: &(dyn Fn(usize) + Sync)) {
        if count < 2 || self.threads < 2 {
            (0..count).for_each(f);
            return;
        }
        let _turn = self.turn.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: the lifetime is left out only until this call returns, and
        // it does not before no helper is inside `f` and none can enter.
        let work = Work(unsafe {
            std::mem::transmute::<
                *const (dyn Fn(usize) + Sync + '_),
                *const (dyn Fn(usize) + Sync + 'static),
            >(f)
        });
        let mut state = self.state();
        (state.work, state.next, state.count, state.panicked) = (Some(work), 0, count, false);
        drop(state);
        if count > self.threads {
            self.more.notify_all();
        } else {
            (1..count).for_each(|_| self.more.notify_one());
        }
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            loop {
                let Some((_, i)) = take(&mut self.state()) else {
                    break;
                };
                f(i);
            }
        }));
        let mut state = self.state();
        // (After a panic here: nothing more is begun.)
        state.work = None;
        while state.running > 0 {
            state = self.idle.wait(state).unwrap_or_else(|e| e.into_inner());
        }
        let panicked = state.panicked;
        drop(state);
        if let Err(panic) = ran {
            std::panic::resume_unwind(panic);
        }
        assert!(!panicked, "a job of landing panicked");
    }
}

/// The next number of the batch, if there is one.
fn take(state: &mut State) -> Option<(Work, usize)> {
    let work = state.work?;
    if state.next >= state.count {
        return None;
    }
    state.next += 1;
    Some((work, state.next - 1))
}

/// [`Pool::each`] on the threads kept for landing.
pub(crate) fn each(count: usize, f: &(dyn Fn(usize) + Sync)) {
    static POOL: OnceLock<&'static Pool> = OnceLock::new();
    POOL.get_or_init(|| Pool::new("victory-land", threads()))
        .each(count, f);
}

/// A piece to land: the copy's number, the title address, its bytes.
pub(crate) type Piece = (u64, usize, usize);

/// `pieces` in the order they are landed: by the 64 KiB block of title
/// memory they lie in, those in one block in the order given (each is
/// within one block). Where one copy alone has pieces in a block, those
/// that overlap or touch are one piece: a landing's flights declare parts
/// of the same memory, and what overlapped came down and was merged twice.
/// Each with the number of the piece given that it begins with.
pub(crate) fn ordered(pieces: &[Piece]) -> Vec<(usize, usize, usize)> {
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by_key(|&i| pieces[i].1 / BLOCK);
    let mut out: Vec<(usize, usize, usize)> = Vec::with_capacity(order.len());
    for group in order.chunk_by(|&a, &b| pieces[a].1 / BLOCK == pieces[b].1 / BLOCK) {
        let alone = group.len() > 1 && group.iter().all(|&i| pieces[i].0 == pieces[group[0]].0);
        if !alone {
            out.extend(group.iter().map(|&i| (i, pieces[i].1, pieces[i].2)));
            continue;
        }
        let mut sorted = group.to_vec();
        sorted.sort_by_key(|&i| pieces[i].1);
        let begun = out.len();
        for i in sorted {
            let (_, at, len) = pieces[i];
            match out[begun..].last_mut() {
                Some(last) if at <= last.1 + last.2 => {
                    let end = (last.1 + last.2).max(at + len);
                    count(stat::TWICE, (last.2 + len - (end - last.1)) as u64);
                    last.2 = end - last.1;
                }
                _ => out.push((i, at, len)),
            }
        }
    }
    out
}

/// Pieces (title address, bytes; in [`ordered`]'s order) as jobs: runs of
/// about [`JOB`] bytes that end where a block ends.
pub(crate) fn jobs(pieces: &[(usize, usize)]) -> Vec<std::ops::Range<usize>> {
    let mut out: Vec<std::ops::Range<usize>> = Vec::new();
    let (mut from, mut bytes) = (0, 0);
    for (i, &(at, len)) in pieces.iter().enumerate() {
        let new_block = i == 0 || pieces[i - 1].0 / BLOCK != at / BLOCK;
        if bytes >= JOB && new_block {
            out.push(from..i);
            (from, bytes) = (i, 0);
        }
        bytes += len;
    }
    if from < pieces.len() {
        out.push(from..pieces.len());
    }
    out
}

/// What earlier fetches brought, for as long as a flight they ran behind
/// has not landed: per fetch, the value the host's fence had been given
/// behind the last flight sent before it, and by copy (its number) and
/// piece the offsets it brought.
///
/// A fetch on the drawing queue runs behind everything sent before it, so
/// it brings what those flights' shaders wrote too, and the merge of it put
/// that into title memory. Their own landings flag the same pieces and
/// fetched them again, to find every byte merged. A piece that a fetch held
/// whole, behind the flight that flags it, has nothing more of that flight.
/// (Bytes left out of that merge, the title's that went up behind the
/// fetch, went up behind that flight too, and stand over what it wrote.)
///
/// Ported as a structure; this tree's relay does not keep a fetch's `Had`
/// yet, so nothing is left out on its word (`VICTORY_LAND_SKIP=0`).
#[derive(Default)]
pub(crate) struct Had(Vec<(u64, std::collections::HashMap<(u64, usize), (usize, usize)>)>);

/// (Kept whole from the port: the relay does not keep a fetch's `Had` yet, so
/// nothing calls these outside the tests; `VICTORY_LAND_SKIP=0`.)
#[allow(dead_code)]
impl Had {
    /// Flights from the one behind which `fence` is signalled on are landed
    /// next: what is of no use to them is dropped.
    pub(crate) fn landing(&mut self, fence: u64) {
        self.0.retain(|had| had.0 >= fence);
    }

    /// A fetch that ran behind every flight up to `fence` brought (copy,
    /// title address, bytes) `pieces`. They are merged before the next ask.
    pub(crate) fn fetched(&mut self, fence: u64, pieces: impl Iterator<Item = Piece>) {
        let mut all = std::collections::HashMap::new();
        for (copy, at, len) in pieces {
            // (Two parts of a block that do not touch: the longer one.)
            let had = all.entry((copy, at / BLOCK)).or_insert((at, at + len));
            if len > had.1 - had.0 {
                *had = (at, at + len);
            }
        }
        self.0.push((fence, all));
    }

    /// Whether a fetch behind the flight of `fence` brought all of the
    /// piece.
    #[allow(dead_code)]
    pub(crate) fn holds(&self, fence: u64, (copy, at, len): Piece) -> bool {
        self.0.iter().any(|(behind, all)| {
            *behind >= fence
                && all
                    .get(&(copy, at / BLOCK))
                    .is_some_and(|had| had.0 <= at && at + len <= had.1)
        })
    }
}

/// What a job put into title memory through the title's own address, to be
/// noted ([`Notes::flush`]).
#[derive(Default)]
pub(crate) struct Notes {
    /// The 64 KiB blocks of title memory a piece put bytes into with their
    /// write watch clean before the stores and reset after them (page
    /// numbers, as the journal counts them): to be noted at the end of the
    /// job, after every byte is in and every reset made.
    pages: Vec<usize>,
    /// Per such piece, its block and the copy it was landed through (which
    /// the job holds until the flush). The block is noted once; a second
    /// entry for it of another copy says two copies landed there, and then
    /// neither holds the block as it is now.
    landed: Vec<(usize, *const crate::shadow::Shadow)>,
}

// SAFETY: the pointers are of copies the job holds until `flush`, which
// reads them on the thread the job ran on (as in the tree this came from).
unsafe impl Send for Notes {}

impl Notes {
    /// At the end of a job: the blocks it put bytes into with their write
    /// watch clean and reset are noted, one asking for the job, and each
    /// block one copy alone landed in is noted current for that copy.
    ///
    /// Noted only now, after the bytes are in and the resets are made: a copy
    /// that read the block before them did so with the number of an asking
    /// made before that, which is under the one given here, and takes it
    /// again.
    pub(crate) fn flush(&mut self) {
        if self.pages.is_empty() {
            return;
        }
        let noting = std::time::Instant::now();
        let (now, blocks) = crate::watch::landed_blocks(&self.pages);
        count(stat::LANDED, blocks.len() as u64);
        for (block, before) in blocks {
            // Only where one copy alone landed in the block: with two, each
            // holds only its own bytes of it and can be noted current for
            // neither ([`taken`]).
            let mut through = self.landed.iter().filter(|l| l.0 == block).map(|l| l.1);
            let Some(copy) = through.next() else {
                continue;
            };
            if !copy.is_null() && through.all(|other| other == copy) {
                // SAFETY: the job holds its copies until this is done.
                let copy = unsafe { &*copy };
                let stamped = copy.landed(block, before, now);
                count(stat::STAMPED, u64::from(stamped));
                count(stat::STAMPED_BYTES, if stamped { BLOCK as u64 } else { 0 });
            }
        }
        self.pages.clear();
        self.landed.clear();
        spent(stat::NOTES, noting);
    }
}

#[cfg(test)]
impl Notes {
    /// Whether nothing was put into it: no block to note and no copy to note
    /// current (the test of a block a landing must leave to the kernel).
    pub(crate) fn is_empty_for_test(&self) -> bool {
        self.pages.is_empty() && self.landed.is_empty()
    }
}

/// Lands one piece: `parts` (offsets from and to) of the bytes at `now`,
/// which the card wrote, go into title memory from `title` on and into
/// `held` ([`crate::safecopy::merge`] says how, and what `take` is). The
/// piece lies in one 64 KiB block of title memory. Returns the bytes put.
///
/// # Safety
/// `now` and `held` are ours from 0 to the last part's end, and nothing but
/// the card writes `now` there, nothing but this call `held`.
pub(crate) unsafe fn piece(
    title: usize,
    now: *const u8,
    held: *mut u8,
    parts: &[(usize, usize)],
    take: bool,
    notes: &mut Notes,
    copy: *const crate::shadow::Shadow,
) -> u64 {
    let Some(end) = parts.last().map(|p| p.1) else {
        return 0;
    };
    // The pages that differ, before any is written: none, and the guards
    // stay; else they are opened once for the piece.
    let mut differing = [false; BLOCK / PAGE + 1];
    let base = title / PAGE;
    let mut pages = 0u64;
    for_pages(title, parts, |from, n| {
        pages += 1;
        // SAFETY: the caller's.
        if unsafe { crate::safecopy::differ(now.add(from), held.add(from), n) } {
            differing[(title + from) / PAGE - base] = true;
        }
    });
    count(stat::PAGES, pages);
    let Some(first) = differing.iter().position(|&d| d) else {
        return 0;
    };
    let last = differing.iter().rposition(|&d| d).unwrap_or(first);
    let (from, to) = (
        ((base + first) * PAGE).max(title),
        ((base + last + 1) * PAGE).min(title + end),
    );
    let block = title / BLOCK;
    let opening = std::time::Instant::now();
    // Whether the kernel's write watch knows nothing of the block before the
    // stores below: only then can what they set be told apart from the
    // title's own writes, reset, and the block noted as ours ([`taken`]).
    // Asked the way `Shadow::put_back_quiet` asks of its own range.
    let quiet = taken() && block_clean(block);
    if taken() && !quiet {
        count(stat::UNQUIET, 1);
    }
    // What shaders wrote is about to go into title memory there: its guarded
    // blocks are opened, so the stores below fault nothing, and every copy
    // of those blocks is told they changed.
    crate::watch::writing(from, to - from);
    spent(stat::NOTES, opening);
    let (mut new_pages, mut blocks, mut dwords, mut missing) = (0u64, 0u64, 0u64, 0u64);
    for_pages(title, parts, |from, n| {
        if !differing[(title + from) / PAGE - base] {
            return;
        }
        // SAFETY: the caller's; title memory, by the title's own address,
        // whose guards were opened above. A fault leaves through the
        // safecopy failing exit and comes back as `there == false`.
        let mut done = unsafe {
            crate::safecopy::merge(
                (title + from) as *mut u8,
                now.add(from),
                held.add(from),
                n,
                take,
            )
        };
        if !done.there {
            // Away while the runtime changes a mapping beside it (or gone
            // for good). Not waited for here (`watch::again_after_change`
            // is not ported): counted, and the next flight that declares
            // the range lands it if title memory is back by then. `held`
            // takes `now` as `take` says (the caller has the card's bytes
            // nowhere else).
            missing += 1;
            if take {
                // SAFETY: as above; the page the first pass stopped on is
                // not written through, but takes the card's bytes in `held`.
                let again = unsafe {
                    crate::safecopy::merge(
                        (title + from) as *mut u8,
                        now.add(from),
                        held.add(from),
                        n,
                        true,
                    )
                };
                done = crate::safecopy::Merged {
                    new: done.new.max(again.new),
                    dwords: done.dwords + again.dwords,
                    there: again.there,
                };
            }
        }
        blocks += done.new;
        dwords += done.dwords;
        if done.new > 0 {
            new_pages += 1;
        }
    });
    // The bytes are in: the block's write-watch bits, which the stores above
    // set, are cleared, so that the next asking does not report them as the
    // title's write (a number above the note made at the end of the job) and
    // the copy the landing went through is not told to take its own output
    // again. Only a block that got bytes and lost none is reset and noted: a
    // reset of one the title wrote to, or that our merge could not finish,
    // would lose its write for nothing.
    if quiet && missing == 0 && new_pages > 0 {
        block_reset(block);
        count(stat::QUIET, 1);
        notes.pages.push(block);
        notes.landed.push((block, copy));
    }
    count(stat::NEW_PIECES, u64::from(new_pages > 0));
    count(stat::NEW_PAGES, new_pages);
    count(stat::NEW_BLOCKS, blocks);
    count(stat::DWORDS, dwords);
    count(stat::MISSING, missing);
    crate::shadow::PUT[0].fetch_add(u64::from(dwords > 0), Ordering::Relaxed);
    crate::shadow::PUT[2].fetch_add(dwords * 4, Ordering::Relaxed);
    count(stat::PIECES, 1);
    dwords * 4
}

/// Whether the kernel's write watch reports nothing written in 64 KiB block
/// `block` of title memory since its pages were last reset, and the block is
/// write-watched at all: the check `Shadow::put_back_quiet` makes of its own
/// range before it stores. False where the kernel cannot answer (view
/// memory, uncommitted pages, a block across allocations), and then the
/// block is landed the old way: reset by nobody, reported by the kernel, and
/// never noted or stamped.
fn block_clean(block: usize) -> bool {
    use windows_sys::Win32::System::Memory::GetWriteWatch;
    let mut found = [std::ptr::null_mut::<std::ffi::c_void>(); 32];
    let mut count = found.len();
    let mut granularity = 0u32;
    // SAFETY: a query of title memory into an array of ours (a block is 16
    // pages of 4 KiB, the unit of the answer).
    let asked = unsafe {
        GetWriteWatch(
            0,
            (block * BLOCK) as _,
            BLOCK,
            found.as_mut_ptr(),
            &mut count,
            &mut granularity,
        )
    };
    asked == 0 && count == 0
}

/// Clears the kernel's write-watch bits of 64 KiB block `block` of title
/// memory, which the stores of a landing set: what would have reported them
/// as the title's writes forgets them, and the journal's note, made after the
/// bytes are in, is what tells every copy.
///
/// Only a block [`block_clean`] found write-watched and clean before the
/// stores is reset.
///
/// shortcut: a title write to one of the block's pages between the check and
/// the reset is lost to the write watch (as `Shadow::put_back_quiet`'s own
/// doc says of its range; `VICTORY_LAND_TAKEN=0` lands the old way).
fn block_reset(block: usize) {
    use windows_sys::Win32::System::Memory::ResetWriteWatch;
    // SAFETY: `block_clean` found the block write-watched and clean before
    // the stores, and it is title memory this landing was given.
    unsafe { ResetWriteWatch((block * BLOCK) as _, BLOCK) };
}

/// `parts` of a piece that begins at `title`, cut where 4 KiB pages of title
/// memory end: offset and bytes.
fn for_pages(title: usize, parts: &[(usize, usize)], mut f: impl FnMut(usize, usize)) {
    for &(from, to) in parts {
        let mut at = from;
        while at < to {
            let n = (PAGE - (title + at) % PAGE).min(to - at);
            f(at, n);
            at += n;
        }
    }
}

#[cfg(test)]
mod tests;
