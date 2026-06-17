# nsearch

A simple search engine built in Rust. Crawls web pages, builds an inverted index, and ranks results using BM25 or TF-IDF.

## Usage

### 1. Crawl

Fetch pages starting from a seed URL and write them to a JSONL file.

```bash
cargo run -- crawl --seed <url> [--limit <n>] [--output <path>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--seed` | required | Starting URL |
| `--limit` | 100 | Max pages to crawl |
| `--output` | `data/pages.jsonl` | Output file |

```bash
cargo run -- crawl --seed https://en.wikipedia.org/wiki/Rust_(programming_language) --limit 50
```

Up to 10 pages are fetched concurrently. Each page is written as a JSON line: `{"url": "...", "title": "...", "text": "..."}`.

---

### 2. Index

Build an inverted index from the crawled pages.

```bash
cargo run -- index [--pages-path <path>] [--index-output <path>] [--store-output <path>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--pages-path` | `data/pages.jsonl` | Input pages file |
| `--index-output` | `data/index.json` | Output index file |
| `--store-output` | `data/store.json` | Output doc store file |

```bash
cargo run -- index --pages-path data/pages.jsonl
```

---

### 3. Query

Search the index and print ranked results.

```bash
cargo run -- query "<terms>" [--index-path <path>] [--store-path <path>] [--scorer <bm25|tfidf>] [--k1 <f>] [--b <f>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--index-path` | `data/index.json` | Index file |
| `--store-path` | `data/store.json` | Doc store file |
| `--scorer` | `bm25` | Ranking algorithm (`bm25` or `tfidf`) |
| `--k1` | `1.2` | BM25 term frequency saturation |
| `--b` | `0.75` | BM25 document length normalization |

```bash
cargo run -- query "memory safety ownership"
cargo run -- query "async await" --scorer tfidf
```

Prints the top 10 results with score, title, and URL.

---

## End-to-end example

```bash
cargo run -- crawl --seed https://en.wikipedia.org/wiki/Rust_(programming_language) --limit 50
cargo run -- index
cargo run -- query "memory safety ownership"
```

## Running tests

```bash
cargo test
```
