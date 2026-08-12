# Limux resource-drain evidence packet — 2026-08-12

Owner: `gula`
Repository: `/home/riche/MCPs/limux`
Incident collaborator: `momo`
Packet finalized: `2026-08-12T09:35:58-04:00`

## Runtime provenance

The measured installed host and CLI resolve to:

```text
/home/riche/.local/limux-reviewed/stable/main-15ccb28ed4a8-matched-20260731/libexec/limux-host
limux-host 0.2.3 (15ccb28ed4a8, release) install-id=main-15ccb28ed4a8-matched-20260731 channel=stable
/home/riche/.local/limux-reviewed/stable/main-15ccb28ed4a8-matched-20260731/libexec/limux-cli
limux-cli 0.2.3 (15ccb28ed4a8, release) install-id=main-15ccb28ed4a8-matched-20260731 channel=stable
```

The installed `install-info.json` identifies full source SHA
`15ccb28ed4a849b5d4ec33bb5f2a93fd709752cc`, release profile, stable channel,
and install ID `main-15ccb28ed4a8-matched-20260731`. This differs from the
source checkout used for the analysis; all comparative measurements below are
therefore tied to the installed binary identity, not repository `HEAD`.

Measurement tools and resolved paths:

```text
/usr/bin/ps          ps from procps-ng 4.0.4
/usr/bin/top         top from procps-ng 4.0.4
/usr/bin/jq          jq-1.7
/usr/bin/sha256sum   sha256sum (GNU coreutils) 9.4
/usr/bin/git         git version 2.43.0
/usr/bin/date        date (GNU coreutils) 9.4
/usr/bin/getconf     glibc 2.39; CLK_TCK=100
```

## Persistent daily-driver attribution

Read-only captures at `2026-08-12T08:49–08:55-04:00` found no live Cargo,
`rustc`, or Xvfb process. Installed host PID `28869` used seven
`llvmpipe-0..6` workers and had no open `/dev/dxg` or `/dev/dri` file
descriptor. `/dev/dxg` existed, so the precise conclusion is: an available WSL
GPU device was not used by this process.

The host accumulated `465,684` CPU ticks; its seven llvmpipe workers accounted
for `413,772`, or 88.9%. Initial memory was approximately `1,012,188 KiB` RSS /
`990,388 KiB` PSS with 55 threads. A later capture was `1,390,536 KiB` RSS /
`1,368,828 KiB` PSS with 61 threads. The increase coincided with two additional
terminal PTYs and six renderer/IO threads, supporting high per-surface
allocation; it does not by itself prove a fixed-workload leak.

The exact read-only capture forms were:

```bash
pid="$(pgrep -n -x limux-host)"
ps -p "$pid" -o pid=,lstart=,etime=,%cpu=,rss=,vsz=,cmd=
ps -eo pid,ppid,lstart,etime,pcpu,pmem,rss,vsz,nlwp,stat,comm --sort=-rss
awk '/^(VmRSS|Threads):/ {print}' "/proc/$pid/status"
awk '/^(Pss|Rss):/ {print}' "/proc/$pid/smaps_rollup"
top -b -H -n 1 -p "$pid"
for task in "/proc/$pid"/task/*; do tr -d '\n' < "$task/comm"; printf '\n'; done | sort | uniq -c | sort -nr
pgrep -af '(^|/)(cargo|rustc)( |$)'
```

No signal, stop, restart, install, or configuration mutation was performed on
that daily-driver host.

## Isolated backend sequence

The repository runner was invoked in isolated preview channels with the exact
installed host and CLI above, an isolated session fixture, and a unique artifact
directory:

```bash
scripts/renderer-backend-preview/renderer-backend-preview.sh \
  --host /home/riche/.local/limux-reviewed/stable/main-15ccb28ed4a8-matched-20260731/libexec/limux-host \
  --cli /home/riche/.local/limux-reviewed/stable/main-15ccb28ed4a8-matched-20260731/libexec/limux-cli \
  --session-template <isolated-session.json> \
  --artifacts <unique-gula-evidence-directory> \
  --start wsl-d3d12-gl
```

Results:

| Run | Captured UTC | Attempt order | Outcome |
|---|---|---|---|
| r1 | 12:59:57 | D3D12, desktop, software | All rejected; the initial `{}` session fixture was invalid, so this is retained as setup-failure evidence only. |
| r2 | 13:00:42 | D3D12, desktop, software | Valid fixture; bounded software accepted. |
| r3 | 13:01:28 | D3D12 | D3D12 accepted with debug diagnostics enabled. |
| r4 | 13:01:42 | D3D12 | D3D12 accepted with Mesa diagnostics only. |
| r5 | 13:01:50 | D3D12 | Clean D3D12 attempt accepted. |

The sequence proves D3D12 is available, but the initial inconsistency means a
direct one-shot forced default is not justified. Product selection must retain
a process boundary and health predicate.

## Bounded software measurement

The final corrected measurement used a fresh isolated host with one workspace
and three visible, healthy, realized terminal surfaces for `5.627557`
seconds—the same duration as momo's incident sample. The measurement harness
supplied no per-terminal command or output workload, and the artifacts do not
preserve the terminal child argv. No agent metadata was attached, so the run
does not establish that momo, nafo, zen_gpt, or any equivalent workload was
running. The historical fixture used `/tmp` as its terminal cwd; it was run
before the operator prohibited further `/tmp` use. The host was launched with:

```text
GSK_RENDERER=gl
LIBGL_ALWAYS_SOFTWARE=1
GALLIUM_DRIVER=llvmpipe
LP_NUM_THREADS=2
```

Two independent CPU readings agreed:

- `/proc/<pid>/stat`: 14 ticks at `CLK_TCK=100`, or 2.49% of one core.
- Interval `top`, excluding its initial cumulative row: 1.6% average.

The same capture recorded `472,440 KiB` RSS, `405,333 KiB` PSS, 27 threads,
two llvmpipe workers, `127.94` Ghostty ticks/s, and `2.67` queued render
actions/s. Every terminal was healthy and realized before and after the sample.
This proves that Mesa honored the two-worker setting and that this bounded
configuration was healthy and low-CPU under this isolated idle workload. It
does not prove a reduction against an otherwise identical unbounded run, because
no same-fixture unbounded A/B baseline was captured. It also does not establish
the effect for the loaded daily-driver workload or prove that setting the
variable globally is product-safe.

The isolated run did not restore the daily driver's 30 workspaces, 41 saved
terminal tabs, 17 suspended agents, terminal output, or scrollback. Only the
installed binary identity and sample duration matched. Terminal dimensions were
`56x18`, `56x39`, and `56x18` cells at `568x396`, `568x823`, and `568x395`
pixels. The packet contains no evidence that window state, visible pixel area,
font settings, output rate, or scrollback matched the daily driver. Daily-driver
and isolated memory/CPU figures must therefore not be presented as an
apples-to-apples performance comparison.

The command mechanics were `setsid env` with isolated `LIMUX_SOCKET`,
`LIMUX_SESSION_DIR`, and XDG directories, followed by:

```bash
start_ticks="$(awk '{print $14 + $15}' "/proc/$host_pid/stat")"
top -b -d 1 -n 6 -p "$host_pid" > top-samples.txt
timeout 5.627557s tail -f /dev/null
end_ticks="$(awk '{print $14 + $15}' "/proc/$host_pid/stat")"
awk '/^VmRSS:/ {print $2}' "/proc/$host_pid/status"
awk '/^Pss:/ {print $2}' "/proc/$host_pid/smaps_rollup"
ps -T -p "$host_pid" -o comm=
```

The isolated hosts were stopped and reaped after capture. No installed runtime
was changed.

## Source correlation and safe remedy

- `rust/limux-host-linux/src/main.rs:520-529` disables GLES and Vulkan before
  GTK initialization but does not select WSL D3D12.
- `rust/limux-host-linux/src/terminal.rs:1127-1179` has an 8 ms visible timer
  and a 100 ms fallback timer; `terminal.rs:1326-1336` also coalesces Ghostty
  wakeups into application ticks. Ghostty documents each wakeup as requiring a
  full tick, so blanket wakeup suppression is not a valid first fix.
- `ghostty/src/apprt/embedded.zig:1016-1044` copies the host process environment
  for each terminal. `ghostty/src/apprt/embedded.zig:572-583` and
  `ghostty/src/termio/Exec.zig:807-813` can add/override child variables but
  cannot remove inherited variables.
- Consequently, automatic `GSK_RENDERER`, `GALLIUM_DRIVER`, or
  `LP_NUM_THREADS` injection in the launcher/host would leak into every terminal
  shell and nested Mesa application. Empty values are not removals.
- Removing variables from the host after GTK/Mesa initialization is thread-unsafe
  on Unix. A rejected D3D12 initialization also requires a fresh process because
  GDK caches the GL initialization failure.

The smallest correct product path is therefore:

1. Add an owned/upstream libghostty C API that lets a surface remove selected
   inherited environment keys before its terminal child is spawned.
2. At a process boundary, try D3D12 first.
3. Accept only a healthy, realized `GskGLRenderer` with `/dev/dxg` open and no
   software indicator.
4. Reap a rejected process and retry in a fresh process.
5. Use bounded software only after the injected renderer variables can be
   excluded from terminal children, unless the operator explicitly accepts
   propagation.

Limux's repository rule treats the vendored `ghostty/` tree as read-only, so no
vendor patch was made. TaskMaster tag `limux-resource-crash-20260716`, Task 7,
tracks the blocker and exact unblock condition.

## Acceptance checks

A source implementation is not effective until an isolated installed build
proves all of the following:

- rejected renderer attempts launch zero user terminal processes and are fully
  reaped before fallback;
- accepted D3D12 uses `GskGLRenderer`, holds `/dev/dxg`, has no llvmpipe worker,
  and all terminal surfaces are healthy and realized;
- only Limux-injected renderer variables are absent from the actual terminal
  child, while caller-provided renderer variables and unrelated values remain
  exact;
- bounded software reproduces the two-worker resource result without variable
  leakage;
- live daily-driver promotion/restart occurs only after explicit operator
  approval and a rollback path.

## Apples-to-apples validation still required

No current result is a same-workload unbounded-versus-bounded comparison. The
smallest non-disruptive approximation is to prepare a private, repository-owned
copy of the saved session layout; disable every agent restore command in that
copy; and launch two sequential isolated hosts using the exact same installed
binary, display, settings, window geometry, surface layout, and deterministic
terminal-output replay. Run A must force llvmpipe without `LP_NUM_THREADS`;
Run B must differ only by `LP_NUM_THREADS=2`. Both runs must record host and
llvmpipe CPU ticks, RSS/PSS, thread counts, renderer diagnostics, per-surface
dimensions, tick/render deltas, output bytes/rate, and scrollback depth over the
same measured interval. All sockets, session state, logs, and fixtures must live
under this repository-owned evidence directory.

That isolated A/B can decide whether the cap helps a matched high-surface-count
replay without touching the current live host. It still cannot prove behavior
for the actual momo/nafo/zen_gpt processes. A truly equivalent real-workload A/B
requires operator-approved fresh-process runs because Mesa reads the setting at
process initialization; it cannot be applied to the already-running host.
Nothing in this packet authorizes that restart or any live-host mutation.

## Load-bearing hashes

```text
7b357a63fff2cdceba13fa1db2f099faf53fb1cbb402c9dcc8f64fd41d3f86c2  preview-r1/result.json
1ea062116a8ec9302801951176b79bea8a7d86f5f1d66ec0800ecd9f8f71d75d  preview-r2/result.json
4a4f0e3a9b84911a71ed63fa4b7f0f01f49e4a6dfcef8579ea803b2a12796f7b  preview-r3/result.json
a05d4440eae3aac21d16bf66e0ab741e36708fa19b5224aff19ab75c1abbf098  preview-r4/result.json
12bc9d94b0bb3afcc400a04b96cd1a7275d8a52bf0fda6651b201bca2db4ed50  preview-r5/result.json
21546648f11670699f4270ad4a66f0a066619c94c42c683b07d93533957c568e  bounded-software-r2/result.json
39046b5863415f52f243af9329c7270937248818efe5989de73c545458d410e6  bounded-software-r4/result.json
```

Historical capture files were relocated into this repository-owned directory
after the operator prohibited further use of `/tmp`. No new artifact, log,
fixture, watcher state, or command execution uses `/tmp` in this lane.
