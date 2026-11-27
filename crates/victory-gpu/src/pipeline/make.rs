//! Host pipelines are made by workers, not by the title thread that first
//! records a draw with them. Translating a shader and the host driver's
//! compile take from milliseconds to seconds each (structurisation of the
//! biggest compute programs ~0.4 s, some driver compiles 3.4 s), and
//! the title's first world frame wants over 1,200 new pipelines: made
//! on its recording threads, that frame took more than 30 s and the
//! title's GPU watchdog ended the run (decision 0007, "Pipelines made off
//! the title's threads").
//!
//! A first use queues the pipeline and waits for it within a budget per
//! frame, so the cheap ones (menus, one-off passes such as a colour-grading
//! bake) are there at once as before; past the budget a draw or dispatch
//! whose pipeline is not ready is dropped, as an untranslated one is.
//!
//! The default budget stays half a second per title frame in this fork. The
//! ported mechanism lets a run make it a few ms per frame
//! (`VICTORY_PIPELINE_WAIT_MS=4`): frames of the world then are not held by
//! pipelines still being made (the title begins two or three frames per
//! frame it presents), and a draw whose pipeline is not ready is dropped
//! until it is, as a game streams in what it has not compiled yet. The
//! decoration bake's pipelines, a pass the title runs once, still get half
//! a second (`VICTORY_PIPELINE_WAIT_ONCE_MS`).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::Shader;
use super::host::{GraphicsState, HostPipeline};

/// What a host pipeline is made from, and the host pipeline once made.
pub(crate) struct Pending {
    shaders: Vec<Arc<Shader>>,
    pub(crate) graphics: Option<Box<GraphicsState>>,
    queue_state: AtomicU8,
    made: OnceLock<Option<HostPipeline>>,
    /// What identifies it across runs (shader containers and state), and
    /// whether a draw or dispatch has requested it yet.
    key: String,
    used: AtomicBool,
    /// Why its shaders are left out for the whole run ([`super::left_out`]);
    /// `None`: it runs.
    left_out: Option<&'static str>,
}

// SAFETY: D3D12 objects are free-threaded; the graphics state (whose input
// elements point into its own names) is only read after creation.
unsafe impl Send for Pending {}
// SAFETY: as above.
unsafe impl Sync for Pending {}

impl Pending {
    pub(crate) fn new(shaders: Vec<Arc<Shader>>, graphics: Option<Box<GraphicsState>>) -> Pending {
        let key = key(&shaders, graphics.as_deref());
        let left_out = super::left_out(&shaders);
        Pending {
            shaders,
            graphics,
            queue_state: AtomicU8::new(NOT_QUEUED),
            made: OnceLock::new(),
            key,
            used: AtomicBool::new(false),
            left_out,
        }
    }

    fn make(&self) {
        self.made
            .get_or_init(|| HostPipeline::new(&self.shaders, self.graphics.as_deref()));
        let (lock, done) = &DONE;
        let _held = lock.lock().unwrap_or_else(|e| e.into_inner());
        done.notify_all();
    }
}

/// What identifies a title pipeline across runs (the early list, the
/// pre-build's list): its shader containers and graphics state.
pub(crate) fn key(shaders: &[Arc<Shader>], graphics: Option<&GraphicsState>) -> String {
    let mut key = blake3::Hasher::new();
    for s in shaders {
        key.update(s.stage.as_bytes());
        key.update(s.hash.as_bytes());
    }
    key.update(graphics.map(|g| g.describe()).unwrap_or_default().as_bytes());
    key.finalize().to_hex()[..32].to_owned()
}

/// Why a pipeline has no host pipeline (diagnostics): its shaders that do
/// not translate, else "refused" or "being made"; `None` if it has one.
#[cfg(feature = "diagnostics")]
pub(crate) fn missing(p: &Pending) -> Option<String> {
    if let Some(why) = p.left_out {
        return Some(why.to_owned());
    }
    match p.made.get() {
        Some(Some(_)) => None,
        None => Some("being made".into()),
        Some(None) => {
            let failed: Vec<String> = p
                .shaders
                .iter()
                .filter(|s| matches!(s.translated.get(), Some(None)))
                .map(|s| format!("{} {}", s.stage, s.hash))
                .collect();
            Some(if failed.is_empty() {
                format!("refused {:?}", p.shaders)
            } else {
                failed.join(" + ")
            })
        }
    }
}

/// Signalled whenever a worker has made a pipeline.
static DONE: (Mutex<()>, Condvar) = (Mutex::new(()), Condvar::new());

/// Waiting allowed per title frame, over all threads
/// (`VICTORY_PIPELINE_WAIT_MS`, 500; 4 is the A/B value).
fn wait_budget() -> Duration {
    static BUDGET: OnceLock<Duration> = OnceLock::new();
    *BUDGET.get_or_init(|| {
        let millis = std::env::var("VICTORY_PIPELINE_WAIT_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(500);
        tracing::info!(millis, "host pipeline wait budget per frame");
        Duration::from_millis(millis)
    })
}

/// Waiting allowed per title frame for the pipelines of a pass the title
/// runs once (the decoration bake, `crate::draw::BAKE_PIXEL_SHADERS`):
/// `VICTORY_PIPELINE_WAIT_ONCE_MS`, 500.
fn once_budget() -> Duration {
    static BUDGET: OnceLock<Duration> = OnceLock::new();
    *BUDGET.get_or_init(|| {
        let millis = std::env::var("VICTORY_PIPELINE_WAIT_ONCE_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(500);
        Duration::from_millis(millis)
    })
}

impl Pending {
    /// Whether this is a pipeline of a pass the title runs once.
    fn runs_once(&self) -> bool {
        self.shaders.iter().any(|s| crate::draw::BAKE_PIXEL_SHADERS.contains(&s.hash.as_str()))
    }
}

/// Pipelines queued and not made yet.
static WAITING: AtomicU64 = AtomicU64::new(0);

/// Pipelines queued and not made yet (for the frame line).
pub(crate) fn waiting() -> u64 {
    WAITING.load(Ordering::Relaxed)
}

/// The host pipeline if it is made (or is made within the frame's budget);
/// queues it on first use.
pub(crate) fn ready(p: &Arc<Pending>) -> Option<&HostPipeline> {
    if p.left_out.is_some() {
        return None;
    }
    // A still-pending pipeline was requested by the title too. Prewarm it
    // on the next launch, even if this run never got to use it.
    if !p.used.load(Ordering::Relaxed) && !p.used.swap(true, Ordering::Relaxed) {
        super::library::note_used(&p.key);
    }
    if p.made.get().is_none() {
        start(p, REQUESTED, None);
        let frame = crate::frame::current();
        // The first 600 frames (loading) make theirs whatever it costs;
        // later frames only while this frame is under the compute cap.
        let early = frame < 600;
        if p.graphics.is_none() && (early || (compute_waits() && compute_room(frame))) {
            // Made here, or waited for while a worker makes it.
            let began = Instant::now();
            p.make();
            if !early {
                compute_spent(frame, began.elapsed());
            }
        } else {
            wait(p, frame);
        }
    }
    p.made.get()?.as_ref()
}

/// Whether a dispatch waits for its compute pipeline until it is made,
/// whatever the frame's budget ([`wait_budget`]); a draw still waits within
/// the budget and is dropped while its pipeline is not there.
///
/// A dropped draw is drawn again in the next frame. A dropped dispatch may
/// not be: the title runs some compute passes once, when a scene changes
/// (a buffer reset or cleared), and what they leave is read for as long as
/// the scene lasts. In the prison corridor a compute program met for the
/// first time took 4.4 s to make on a 16 GB machine; its one dispatch was
/// dropped after the half second of budget, and the windows of the room
/// before stayed blended over the whole picture for minutes (run 195).
/// A wait of seconds once (the pipeline library has it on the next run)
/// against a picture wrong until the scene ends.
/// On by default; `VICTORY_PIPELINE_COMPUTE_WAIT=0`: within the budget, as
/// before.
fn compute_waits() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_PIPELINE_COMPUTE_WAIT").is_none_or(|v| v != "0"))
}

/// The most a frame waits for compute pipelines made at their dispatch
/// ([`compute_waits`]), over all threads, before its later dispatches go
/// back to the frame's budget ([`wait`]).
///
/// With a new cache the first cutscene met 457 compute programs in one
/// frame at about a second each: the frame did not end for over 30 s and
/// the title's own hang watchdog stored to address 0. Capped, the frame
/// ends and the rest are made by the workers and waited for within the
/// budget, as before [`compute_waits`]. The check comes before a make, so a
/// frame can run over by the makes already started on other threads
/// (seconds each): `VICTORY_PIPELINE_COMPUTE_WAIT_MS`, 8,000 by default,
/// leaves room for that under the watchdog's 30 s.
/// Port of fxceless/victory-fork PR #1450 (GPL-3.0-or-later), one file;
/// adapted to this tree's `ready`, which has no `frame < 600` early rule.
fn compute_cap() -> Duration {
    static MS: OnceLock<u64> = OnceLock::new();
    Duration::from_millis(*MS.get_or_init(|| {
        std::env::var("VICTORY_PIPELINE_COMPUTE_WAIT_MS")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(8_000)
    }))
}

/// The frame whose compute waits are counted, and the microseconds counted.
static COMPUTE_WAITED: Mutex<(u64, u64)> = Mutex::new((0, 0));

/// Whether frame `frame` may still wait for a compute pipeline ([`compute_cap`]).
fn compute_room(frame: u64) -> bool {
    let mut w = COMPUTE_WAITED.lock().unwrap_or_else(|e| e.into_inner());
    if w.0 != frame {
        *w = (frame, 0);
    }
    let room = u128::from(w.1) < compute_cap().as_micros();
    if !room {
        static SAID: AtomicBool = AtomicBool::new(false);
        if !SAID.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                frame,
                cap_ms = u64::try_from(compute_cap().as_millis()).unwrap_or(u64::MAX),
                "a frame waited its cap for compute pipelines; the rest wait within its budget (said once)"
            );
        }
    }
    room
}

/// Counts a compute wait to `frame`, if that is still the frame counted.
fn compute_spent(frame: u64, took: Duration) {
    let mut w = COMPUTE_WAITED.lock().unwrap_or_else(|e| e.into_inner());
    if w.0 == frame {
        w.1 = w.1.saturating_add(u64::try_from(took.as_micros()).unwrap_or(u64::MAX));
    }
}

/// Makes `p` on this thread if it is not made yet (waiting for a worker
/// that is making it); whether it has a host pipeline.
pub(crate) fn now(p: &Arc<Pending>) -> bool {
    if p.left_out.is_some() {
        return false;
    }
    p.make();
    p.made.get().is_some_and(|made| made.is_some())
}

/// At the title's creation of a pipeline: starts it at once if an earlier
/// run drew with it (during loading, so the world's first frames find it).
///
/// A pipeline no earlier run drew with is queued at creation too, behind
/// requested ones, while fewer than `VICTORY_PIPELINE_EAGER` (256) are
/// waiting, so the workers make it in parallel before its first use
/// (XboxCompatibility T-0037 through Belobug/VIctory-Belobug-AMD #622,
/// GPL-3.0-or-later): made only at first use, a loading screen that creates
/// and uses 64 pipelines a frame waited for each in turn on its presenting
/// thread — 24 of 24 loading frames >= 70 ms, max ~140 ms; queued at
/// creation, none >= 70 ms, max ~45 ms.
pub(crate) fn early(p: &Arc<Pending>) {
    if p.left_out.is_some() || p.shaders.is_empty() {
        return;
    }
    if super::library::wanted_early(&p.key) {
        super::library::EARLY.fetch_add(1, Ordering::Relaxed);
        start(p, EARLY, None);
    } else if WAITING.load(Ordering::Relaxed) < eager_limit() {
        start(p, EARLY, Some(eager_limit()));
    }
}

/// Pipelines queued at creation by [`early`] without being on the early list.
pub(crate) static EAGER: AtomicU64 = AtomicU64::new(0);

/// Up to how many queued pipelines a new one is still queued at creation
/// (`VICTORY_PIPELINE_EAGER`: 0 off, as before; default 256). Bounded so a
/// title that creates thousands of pipelines it may never use does not keep
/// every worker busy with them.
fn eager_limit() -> u64 {
    static LIMIT: OnceLock<u64> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        let limit = std::env::var("VICTORY_PIPELINE_EAGER").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(256);
        tracing::info!(limit, "pipelines queued at creation while fewer are waiting (VICTORY_PIPELINE_EAGER)");
        limit
    })
}

/// Whether the frame's budget is one for all threads together: a thread
/// that waits takes what is left of it and gives back what it did not
/// use. Counted only when a wait had ended, the budget was one per thread:
/// each of the title's five recording threads began its first wait of a
/// frame with nothing counted yet and waited the whole half second, 2.5 s
/// of waiting in every frame in which pipelines were being made (runs
/// 34-49: "2504 ms waiting for pipelines" in each such frame).
/// On by default; `VICTORY_PIPELINE_WAIT_SHARED=0` counts as before.
fn shared_budget() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_PIPELINE_WAIT_SHARED").is_none_or(|v| v != "0"))
}

/// Waits for `p` while frame `frame` has budget left.
fn wait(p: &Pending, frame: u64) {
    let budget = if p.runs_once() { once_budget().max(wait_budget()) } else { wait_budget() };
    if budget.is_zero() {
        return;
    }
    // The frame the budget belongs to, and what was waited of it (or is
    // taken by a thread waiting now).
    static SPENT: Mutex<(u64, Duration)> = Mutex::new((u64::MAX, Duration::ZERO));
    let shared = shared_budget();
    let left = {
        let mut spent = SPENT.lock().unwrap_or_else(|e| e.into_inner());
        if spent.0 != frame {
            *spent = (frame, Duration::ZERO);
        }
        let left = budget.saturating_sub(spent.1);
        if shared {
            spent.1 += left;
        }
        left
    };
    if left.is_zero() {
        return;
    }
    let started = Instant::now();
    let (lock, done) = &DONE;
    let mut held = lock.lock().unwrap_or_else(|e| e.into_inner());
    while p.made.get().is_none() {
        let rest = left.saturating_sub(started.elapsed());
        if rest.is_zero() {
            break;
        }
        held = done.wait_timeout(held, rest).unwrap_or_else(|e| e.into_inner()).0;
    }
    drop(held);
    let waited = started.elapsed().min(left);
    {
        let mut spent = SPENT.lock().unwrap_or_else(|e| e.into_inner());
        // A frame that began meanwhile has its own budget.
        if spent.0 == frame {
            if shared {
                spent.1 = spent.1.saturating_sub(left - waited);
            } else {
                spent.1 += waited;
            }
        }
    }
    crate::framelog::add(crate::framelog::T::PipelineWaitUs, started.elapsed().as_micros() as u64);
    #[cfg(feature = "diagnostics")]
    crate::copystat::add(
        crate::copystat::C::PipelineWaitUs,
        started.elapsed().as_micros() as u64,
    );
}

const NOT_QUEUED: u8 = 0;
const EARLY: u8 = 1;
const REQUESTED: u8 = 2;
const MAKING: u8 = 3;

struct Queue {
    early: VecDeque<Arc<Pending>>,
    requested: VecDeque<Arc<Pending>>,
}

impl Queue {
    const fn new() -> Self {
        Self {
            early: VecDeque::new(),
            requested: VecDeque::new(),
        }
    }

    /// Queues an unrequested pipeline or promotes one queued at creation.
    /// Returns whether it is new to the queue, or `None` if nothing changed.
    fn push(&mut self, p: &Arc<Pending>, priority: u8) -> Option<bool> {
        let prev = p.queue_state.load(Ordering::Relaxed);
        if prev >= priority {
            return None;
        }
        p.queue_state.store(priority, Ordering::Relaxed);
        if priority == REQUESTED {
            self.requested.push_back(p.clone());
        } else {
            self.early.push_back(p.clone());
        }
        Some(prev == NOT_QUEUED)
    }

    fn pop(&mut self) -> Option<Arc<Pending>> {
        let p = self.requested.pop_front().or_else(|| {
            // An early entry can have been promoted and claimed already.
            std::iter::from_fn(|| self.early.pop_front())
                .find(|p| p.queue_state.load(Ordering::Relaxed) == EARLY)
        })?;
        p.queue_state.store(MAKING, Ordering::Relaxed);
        Some(p)
    }

    /// Admission and the caller's `WAITING` increment run with `QUEUE`
    /// locked: a pipeline no earlier run drew with is queued only while fewer
    /// than `limit` are waiting (Belobug #622).
    fn push_eager(&mut self, p: &Arc<Pending>, waiting: u64, limit: u64) -> Option<bool> {
        if waiting >= limit {
            return None;
        }
        self.push(p, EARLY)
    }
}

static QUEUE: Mutex<Queue> = Mutex::new(Queue::new());
static AVAILABLE: Condvar = Condvar::new();

/// Start or promote work for a draw without waiting behind the entire early
/// list. Queue changes and worker claims happen under the same lock.
/// `speculative_limit` is [`early`]'s speculative admission: the pipeline is
/// queued only while fewer than that many are waiting, and never made on this
/// thread where no worker runs.
fn start(p: &Arc<Pending>, priority: u8, speculative_limit: Option<u64>) {
    if p.queue_state.load(Ordering::Relaxed) >= priority {
        return;
    }
    if !workers() {
        if speculative_limit.is_none() && p.queue_state.swap(MAKING, Ordering::Relaxed) == NOT_QUEUED {
            p.make();
        }
        return;
    }
    let mut queue = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
    let admitted = match speculative_limit {
        Some(limit) => queue.push_eager(p, WAITING.load(Ordering::Relaxed), limit),
        None => queue.push(p, priority),
    };
    if let Some(new) = admitted {
        if new {
            if speculative_limit.is_some() {
                EAGER.fetch_add(1, Ordering::Relaxed);
            }
            PEAK.fetch_max(WAITING.fetch_add(1, Ordering::Relaxed) + 1, Ordering::Relaxed);
        }
        AVAILABLE.notify_one();
    }
}

fn next() -> Arc<Pending> {
    let mut queue = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
    loop {
        if let Some(p) = queue.pop() {
            return p;
        }
        queue = AVAILABLE.wait(queue).unwrap_or_else(|e| e.into_inner());
    }
}

/// Starts the workers (half the logical processors, 2 to 8) on first use.
fn workers() -> bool {
    static RUNNING: OnceLock<bool> = OnceLock::new();
    *RUNNING.get_or_init(|| {
        let mut running = false;
        let workers = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).clamp(2, 8));
        for i in 0..workers {
            let spawned = std::thread::Builder::new()
                .name(format!("victory-pipelines-{i}"))
                .spawn(|| {
                    // Below the title's own threads (loading, recording).
                    // SAFETY: a plain call on our own thread.
                    unsafe {
                        use windows::Win32::System::Threading::*;
                        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
                    }
                    loop {
                        next().make();
                        if WAITING.fetch_sub(1, Ordering::Relaxed) == 1 {
                            drained();
                        } else {
                            super::library::save(false);
                        }
                    }
                });
            if let Err(e) = spawned {
                tracing::warn!(error = %e, "no pipeline worker");
            } else {
                running = true;
            }
        }
        running
    })
}

/// The most pipelines waiting at once since the workers last ran out.
static PEAK: AtomicU64 = AtomicU64::new(0);

/// The workers ran out of work: logged after a batch of 10 or more, and
/// the cache saved (at most every 20 s).
fn drained() {
    use super::library::{EARLY, LOADED, MADE};
    if PEAK.swap(0, Ordering::Relaxed) >= 10 {
        tracing::info!(
            made = MADE.load(Ordering::Relaxed),
            loaded = LOADED.load(Ordering::Relaxed),
            started_early = EARLY.load(Ordering::Relaxed),
            "host pipelines all made"
        );
    }
    super::library::save(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A first use queues the pipeline and, within the budget, gets the
    /// worker's answer (here: no host pipeline for no shaders).
    #[test]
    fn first_use_is_made_by_a_worker() {
        let p = Arc::new(Pending::new(Vec::new(), None));
        assert!(ready(&p).is_none());
        assert_ne!(p.queue_state.load(Ordering::Relaxed), NOT_QUEUED);
        let started = Instant::now();
        while p.made.get().is_none() && started.elapsed() < Duration::from_secs(10) {
            std::thread::yield_now();
        }
        assert!(matches!(p.made.get(), Some(None)));
    }

    #[test]
    fn pending_pipeline_is_recorded_for_the_next_launch() {
        // A graphics pipeline: a dispatch waits for its compute pipeline even
        // in the frame's budget (`VICTORY_PIPELINE_COMPUTE_WAIT`), but this
        // test is about the worker's queue and the early list, so the wait
        // path stays the one under test.
        let mut p = Pending::new(Vec::new(), Some(Box::new(GraphicsState::default())));
        p.key = "pending-pipeline-test".to_owned();
        // Simulate a worker that has taken the job but cannot finish it yet.
        p.queue_state.store(MAKING, Ordering::Relaxed);
        let p = Arc::new(p);

        assert!(!super::super::library::wanted_early(&p.key));
        assert!(ready(&p).is_none());
        assert!(p.made.get().is_none());
        assert!(super::super::library::wanted_early(&p.key));
    }

    #[test]
    fn draws_promote_early_pipelines_without_claiming_them_twice() {
        let earlier = Arc::new(Pending::new(Vec::new(), None));
        let promoted = Arc::new(Pending::new(Vec::new(), None));
        let requested = Arc::new(Pending::new(Vec::new(), None));
        let mut queue = Queue::new();

        assert_eq!(queue.push(&earlier, EARLY), Some(true));
        assert_eq!(queue.push(&promoted, EARLY), Some(true));
        assert_eq!(queue.push(&requested, REQUESTED), Some(true));
        assert_eq!(queue.push(&promoted, REQUESTED), Some(false));
        assert_eq!(queue.push(&promoted, REQUESTED), None);

        assert!(Arc::ptr_eq(&queue.pop().unwrap(), &requested));
        assert!(Arc::ptr_eq(&queue.pop().unwrap(), &promoted));
        assert_eq!(queue.push(&promoted, REQUESTED), None);
        assert!(Arc::ptr_eq(&queue.pop().unwrap(), &earlier));
        assert!(queue.pop().is_none());
    }

    #[test]
    fn speculative_admission_rechecks_capacity_before_queueing() {        let mut queue = Queue::new();
        let first = Arc::new(Pending::new(Vec::new(), None));
        let second = Arc::new(Pending::new(Vec::new(), None));
        assert_eq!(queue.push_eager(&first, 0, 0), None);
        assert_eq!(queue.push_eager(&first, 0, 1), Some(true));
        assert_eq!(queue.push_eager(&second, 1, 1), None);
        assert_eq!(second.queue_state.load(Ordering::Relaxed), NOT_QUEUED);
        assert_eq!(queue.push(&second, REQUESTED), Some(true));
        assert_eq!(queue.push(&first, REQUESTED), Some(false));
    }

    /// The compute wait is capped per frame: what the frame waited is
    /// counted, and once past the cap its later dispatches go to the frame's
    /// budget instead (`VICTORY_PIPELINE_COMPUTE_WAIT_MS`; fxceless/
    /// victory-fork #1450).
    #[test]
    fn a_frames_compute_waits_are_capped() {
        // A frame number of this test's own: the counter is shared.
        let frame = crate::frame::current() + 1_000_000;
        assert!(compute_room(frame));
        compute_spent(frame, compute_cap() + Duration::from_millis(1));
        assert!(!compute_room(frame));
        // The next frame starts with nothing waited.
        assert!(compute_room(frame + 1));
    }
}
