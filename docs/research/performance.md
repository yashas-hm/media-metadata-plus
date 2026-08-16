# Thumbnail Extraction: Performance Analysis

> Target: EchoFrame — 10K / 50K / 100K+ video libraries.

---

## 1. Throughput Baseline

### Single-thumbnail extraction time (FFmpeg, pre-input seek)

| Resolution  | Codec | SSD total | HDD total   |
|-------------|-------|-----------|-------------|
| 1080p H.264 | h264  | ~15–25 ms | ~80–150 ms  |
| 1080p HEVC  | hevc  | ~20–40 ms | ~100–200 ms |
| 4K H.264    | h264  | ~30–60 ms | ~150–300 ms |
| 4K HEVC     | hevc  | ~40–75 ms | ~200–400 ms |

**Always use pre-input seek** (`-ss` before `-i` in FFmpeg terms, or seeking before opening the decoder in
`ffmpeg-next`). This uses the container index rather than decoding every frame to reach the timestamp — 10–100x faster
for long videos.

**Seek to 10% of duration**, not a fixed timestamp. This skips black intro frames and opening title cards (same strategy
used by Plex and Jellyfin).

### Effective throughput at scale

Assuming mixed 1080p/4K library, SSD, seek to 10% of duration:

| Workers | Thumbs/sec | 10K videos | 50K videos | 100K videos |
|---------|------------|------------|------------|-------------|
| 1       | ~20        | ~8 min     | ~42 min    | ~83 min     |
| 4       | ~80        | ~2 min     | ~10 min    | ~21 min     |
| 8       | ~160       | ~1 min     | ~5 min     | ~10 min     |

These are background processing times. With a warm cache, time to display visible thumbnails on startup is < 500 ms
regardless of library size.

---

## 2. Memory Characteristics

### Per-frame peak memory

```
Peak ≈ src_YUV420 + swscale_buf + RGB_frame + JPEG_output
     ≈ 1.5× (src_w × src_h) + (dst_w × dst_h × 3) + ~2 MB
```

| Source resolution | Peak per worker |
|-------------------|-----------------|
| 720p              | ~8 MB           |
| 1080p             | ~18 MB          |
| 4K                | ~45 MB          |

**Scale to display width at extraction time.** A 300px-wide thumbnail from a 4K source uses only ~2 MB peak instead of ~
45 MB — a 22x reduction. This is the single most impactful optimization.

### Safe concurrency limits

```rust
let workers = (num_cpus::get_physical())
    .min(available_memory_mb / 50)  // 50 MB budget per worker (4K safety margin)
    .max(1);
```

| Device                      | RAM budget | Safe workers (4K source) | Safe workers (1080p source) |
|-----------------------------|------------|--------------------------|-----------------------------|
| Low-end Android (3 GB)      | ~600 MB    | 2                        | 4                           |
| Mid-range mobile (6 GB)     | ~1.5 GB    | 4                        | 8                           |
| macOS Apple Silicon (16 GB) | ~8 GB      | 8                        | 16                          |
| macOS Apple Silicon (32 GB) | ~20 GB     | 12                       | 16+                         |

Use `num_cpus::get_physical()`, not `get()` (which includes hyperthreaded logical cores).

---

## 3. Parallelization

### Rayon (recommended — already in codebase)

`media_metadata_plus` already uses Rayon for `read_metadata_batch`. Thumbnail extraction extends the same pattern:

```rust
pub fn extract_thumbnails_batch(
    requests: Vec<ThumbnailRequest>,
) -> Vec<Option<ThumbnailResult>> {
    use rayon::prelude::*;
    requests.par_iter()
        .map(|r| extract_thumbnail_cached(r).ok())
        .collect()
}
```

Thumbnail extraction is CPU-bound on SSD (40% I/O, 60% CPU). Rayon up to physical core count is appropriate. On HDD, I/O
rises to 70–80% and causes seek thrashing above 2 workers — detect rotational storage and cap accordingly.

### Progressive delivery via `StreamSink`

For large batches, use `flutter_rust_bridge`'s `StreamSink` to deliver results as they complete instead of waiting for
the entire batch:

```rust
pub fn extract_thumbnails_stream(
    paths: Vec<String>,
    sink: StreamSink<ThumbnailResult>,
) {
    use rayon::prelude::*;
    paths.par_iter().for_each(|path| {
        if let Ok(result) = extract_thumbnail_cached(path) {
            sink.add(result).ok();
        }
    });
}
```

On the Dart side, consume with `StreamBuilder` — thumbnails render as they arrive with no batch-complete stall.

---

## 4. The Fast Path: Embedded `covr` Atom

Many MP4/MOV files — especially footage shot on iPhone — embed a JPEG thumbnail directly in the
`moov > udta > ilst > covr` atom. Reading this requires no FFmpeg, no video decoding: just box parsing.

The existing box scanner in `rust/src/video_reader.rs` already walks MP4 boxes. Extending it to read `covr` is a ~
50-line addition.

**Expected coverage:** 50–80% of a personal video library (all iPhone footage, modern Android footage, many GoPro/drone
videos).

**Cost:** ~0.5–2 ms (a filesystem read, no decode). This is 10–50x faster than FFmpeg thumbnail extraction.

This should be Phase 1 of any thumbnail implementation — it handles the majority of the use case with zero new
dependencies.

---

## 5. Caching

Caching is not optional at 50K+ videos. A cold-start full-library extraction takes hours; a warm cache reduces startup
time to milliseconds.

### Cache key

```rust
fn cache_key(path: &Path, target_width: u32) -> String {
    let meta = std::fs::metadata(path).unwrap();
    let mtime_ms = meta.modified().unwrap()
        .duration_since(std::time::UNIX_EPOCH).unwrap()
        .as_millis() as u64;
    let size = meta.len();
    // No full-file SHA-256 — hashing a 4 GB video is slower than extracting the thumbnail
    format!("{:016x}{:016x}{:08x}", hash_path(path), mtime_ms, size ^ target_width as u64)
}
```

A changed `mtime` or `size` produces a different key; orphaned old entries are swept at startup.

### Cache directory layout

A flat directory degrades above ~50K entries on macOS HFS+ and breaks on FAT32. Use two-level hex sharding:

```
<cache_root>/
  00/00a3f7c2..._{width}.jpg
  ff/ff987654..._{width}.jpg
```

256 shards × ~390 files each at 100K videos. Scales to 25M+ thumbnails.

```rust
fn cache_path(root: &Path, key: &str, width: u32) -> PathBuf {
    root.join(&key[..2]).join(format!("{key}_{width}.jpg"))
}
```

### Cache storage

- **JPEG files on filesystem** for thumbnail data (fast direct read, no serialization overhead)
- **SQLite index** for LRU metadata (last accessed, source path, mtime) — enables eviction without directory scans

Evict least-recently-accessed entries when available disk space drops below 500 MB.

### Cache size budget

At ~20 KB per 200px-wide JPEG:

| Library     | Cache size |
|-------------|------------|
| 10K videos  | ~200 MB    |
| 50K videos  | ~1 GB      |
| 100K videos | ~2 GB      |

---

## 6. Priority Queue Architecture

For large libraries, always process visible items first:

```
HIGH_PRIORITY queue  (visible + 1-screen buffer, ~50 items)
  → 3–4 Rayon workers
  → Target: < 100 ms per item

BACKGROUND queue  (full library remainder)
  → 1–2 Rayon workers
  → Best-effort throughput
```

When the user scrolls, promote visible items from BACKGROUND to HIGH_PRIORITY atomically. Use `SliverGrid` with a
`cacheExtent` of ~2 screen heights to pre-build off-screen items before they scroll into view.

---

## 7. Lessons from Production Systems

**Plex:** Seeks to 10% of duration. Runs each `ffmpeg` extraction in an isolated child process (memory isolation; a
corrupt file cannot OOM the server). Concurrency = one worker per CPU core via a SQLite work queue. Cache uses UUID keys
(not path-based) — path changes on remount do not invalidate cache.

**Jellyfin:** Always scales to 400px wide at extraction time — bounds peak memory for 4K source regardless of original
resolution. Concurrency defaults to `cpu_cores / 2` for thumbnails. Emits WebSocket events per-completed thumbnail
(progressive rendering, same as `StreamSink` pattern).

**macOS Photos.app:** Three-tier approach:

1. Read embedded JPEG from `covr` atom (instant, zero decode)
2. Decode first I-frame via VideoToolbox (GPU-accelerated, async)
3. Persistent sharded cache indexed by UUID in `Photos.sqlite`

**Key lessons applicable to EchoFrame:**

- Check for embedded `covr` before invoking FFmpeg
- Seek to 10% of duration, not a fixed offset
- Scale to display width at extraction time (not after)
- Use UUID/hash cache keys (not path-based) for rename resilience
- Deliver results progressively as they complete

---

## 8. Workload Summary

Assumptions: Apple Silicon Mac, 8 workers, SSD, 50 ms avg extraction, 20 KB/thumb cache, `covr` fast-path covers 60% of
library.

| Library | Background time (cold, no covr) | Background time (warm cache) | Peak RAM | Cache size |
|---------|---------------------------------|------------------------------|----------|------------|
| 10K     | ~10 min                         | < 5 sec                      | ~400 MB  | ~200 MB    |
| 50K     | ~52 min                         | < 5 sec                      | ~400 MB  | ~1 GB      |
| 100K    | ~104 min                        | < 5 sec                      | ~400 MB  | ~2 GB      |

With the `covr` fast path covering 60% of the library and 8 workers:

| Library | Effective background time |
|---------|---------------------------|
| 10K     | ~4 min                    |
| 50K     | ~21 min                   |
| 100K    | ~42 min                   |

The background time is acceptable because the UI always serves from cache (< 5 sec on subsequent opens) and users see
thumbnails progressively from the first scroll.