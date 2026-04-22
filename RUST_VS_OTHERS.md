# Rust vs Python vs Go: Key Differences

A practical comparison using the weather CLI as a concrete example.

---

## 1. Memory Management: Ownership vs. Garbage Collection

This is the biggest difference.

### Python

You create objects and the garbage collector cleans them up. You don't think about it.

```python
# Python - just create and forget
response = requests.get(url)
data = response.json()  # GC handles cleanup eventually
```

### Go

Go also uses garbage collection. You allocate and the runtime cleans up.

```go
resp, _ := http.Get(url)
body, _ := io.ReadAll(resp.Body)
// body gets GC'd eventually
```

### Rust

Every value has an **owner**. When the owner goes out of scope, memory is freed immediately—no GC, no pauses.

```rust
let body = reqwest::get(url).await?.text().await?;
// `body` owns the String. When `body` goes out of scope,
// the memory is freed deterministically.
```

**Why it matters in the weather CLI:** In Rust, when `fetchJson()` returns, the caller *owns* the response body. The compiler ensures you can't use it after it's dropped. In Python, you could accidentally hold a reference to a massive response and not realize when it gets freed.

---

## 2. Error Handling: Explicit vs. Implicit

### Python

Exceptions bubble up unpredictably. You might not know what can fail.

```python
response = requests.get(url)  # Could raise ConnectionError, Timeout, etc.
data = response.json()        # Could raise JSONDecodeError
```

### Go

Explicit but verbose. Every error is a value you must check.

```go
resp, err := http.Get(url)
if err != nil {
    return err
}
body, err := io.ReadAll(resp.Body)
if err != nil {
    return err
}
```

### Rust

The `Result<T, E>` type forces you to handle errors, but the `?` operator makes it concise.

```rust
let body = fetch_json(url).await?;  // If it fails, return the error immediately
let data: WeatherData = serde_json::from_str(&body)?;  // Same here
```

The `?` operator is Rust's sweet spot between Go's verbosity and Python's invisibility. It propagates errors but preserves type safety. The compiler won't let you ignore a `Result`.

---

## 3. The Type System: Static, Strong, and Expressive

### Python

Dynamic types. Fast to write, easy to break at runtime.

```python
def get_forecast(url):
    response = requests.get(url)
    return response.json()  # Who knows what this returns?
```

### Go

Static types but limited expressiveness. Generics were added in Go 1.18 but can feel clunky.

```go
func getForecast(url string) (ForecastPeriod, error) {
    // You need explicit structs for everything
}
```

### Rust

Static types with powerful features like **pattern matching**, **enums with data** (Algebraic Data Types), and **generics**.

```rust
enum WeatherResult {
    Success(Forecast),
    Error(String),
}

match result {
    WeatherResult::Success(forecast) => println!("{:#?}", forecast),
    WeatherResult::Error(msg) => eprintln!("Failed: {}", msg),
}
```

In the weather CLI, `Option<Vec<Place>>` and `Result<ForecastPeriod, anyhow::Error>` let the compiler guarantee you handle missing data and failures.

---

## 4. Concurrency: Async/Await with Zero-Cost Abstractions

### Python

The GIL (Global Interpreter Lock) prevents true parallelism for CPU-bound work. `asyncio` exists but is complex.

```python
async def fetch():
    await session.get(url)  # Only one thread runs Python code at a time
```

### Go

Goroutines are amazing—lightweight, easy to spawn, great scheduler.

```go
go fetchWeather(url)  // Spawns a goroutine effortlessly
```

### Rust

`async/await` with explicit runtimes (like Tokio). No runtime baked into the language, so you pay for only what you use.

```rust
let (geo, points) = tokio::join!(
    geocode_zip(zip),
    get_points(lat, lon)
);  // Runs both requests concurrently
```

**Key difference:** Go's goroutines are managed by the runtime. Rust's async is compiled down to state machines—**zero-cost abstractions**. Your async code has the same performance as hand-written state machines.

---

## 5. Compilation and Performance

| Language | Model | Startup | Runtime Performance |
|----------|-------|---------|---------------------|
| Python | Interpreted | Slow (cold imports) | Slow |
| Go | Compiled to native | Fast | Good |
| Rust | Compiled to native with optimizations | Instant | Excellent (rivals C/C++) |

The Rust weather CLI starts instantly and uses minimal memory. A Python version would spend time importing `requests`, `json`, etc.

---

## 6. Ecosystem and Tooling

| Feature | Python | Go | Rust |
|---------|--------|-----|------|
| Package manager | pip / poetry | go mod | **Cargo** |
| Build tool | setuptools / poetry | go build | **Cargo** |
| Formatter | black | gofmt | **rustfmt** |
| Linter | pylint / mypy | go vet | **clippy** |
| Testing | pytest | `go test` | Built-in + `cargo test` |

**Cargo** is arguably the best in class. One tool manages dependencies, building, testing, docs, and publishing. The `Cargo.toml` in this project is the entire project manifest.

---

## 7. Syntax Comparison: Fetching and Parsing JSON

Let's look at the same concept across all three languages.

### Python

```python
import requests

response = requests.get(url)
data = response.json()
places = data['places']
```

### Go

```go
resp, _ := http.Get(url)
defer resp.Body.Close()

var data ZippoResponse
json.NewDecoder(resp.Body).Decode(&data)
places := data.Places
```

### Rust

```rust
let response = client.get(url).send().await?;
let data: ZippoResponse = response.json().await?;
let places = data.places;
```

Rust requires more ceremony (`await?`, type annotations) but gives you compile-time guarantees that `places` exists and is the right type.

---

## 8. What the Weather CLI Codebase Specifically Demonstrates

Reading the actual `src/main.rs` surfaces a few practical advantages that generic comparisons often miss.

### Single Binary Distribution
For a CLI tool, Rust's most immediate practical advantage over Python is deployment. Running `cargo build --release` produces one statically-linked binary (roughly 5–10 MB). There is no Python runtime to install, no `venv` to activate, no `requirements.txt` to keep in sync, and no import path issues. You can copy the binary to any similar target architecture and run it.

### `anyhow::Context` for Rich Error Traces
The code uses `anyhow` to attach human-readable context at every API boundary without manual `try/except` boilerplate:

```rust
let resp = client
    .get(&url)
    .send()
    .await
    .with_context(|| format!("Failed to send geocoding request to {}", url))?;
```

If this fails, the error chain includes the exact URL that caused the problem. In Python you'd need explicit `try/except` blocks and manual string formatting to match this. In Go, you'd repeat `if err != nil { return fmt.Errorf("... %s: %w", url, err) }` at every step.

### Serde as a Compile-Time API Contract
The NWS API returns field names like `place name`, `state abbreviation`, and `detailedForecast`. In `main.rs`, `serde` renames and validates these at compile time:

```rust
#[derive(Deserialize, Debug)]
struct Place {
    latitude: String,
    longitude: String,
    #[serde(rename = "place name")]
    place_name: String,
    #[serde(rename = "state abbreviation")]
    state: String,
}
```

If the upstream API removes `place name`, the project stops compiling. In Python, the same change surfaces as a `KeyError` at runtime for some users and not others. In Go, you'd write custom struct tags and still handle decoding errors manually.

### Error Propagation in a Chained Workflow
The weather CLI chains three dependent network calls (geocode → points → forecast). Because each step returns `Result<T, E>`, the `?` operator propagates failures without nested indentation:

```rust
let (lat, lon, city, state) = geocode_zip(&args.zip).await?;
let points = get_points(lat, lon).await?;
let current = get_forecast(&points.forecast).await?;
```

The equivalent Go code requires explicit `if err != nil` blocks after every single call, which adds significant visual noise when the happy path is the common case.

### Pattern Matching as a Constraint
Rust doesn't just handle missing data—it forces you to prove you've handled it before accessing a value:

```rust
let place = data.places.into_iter().next().context("No location data found for zip code")?;
```

`Vec::into_iter().next()` returns `Option<Place>`. You cannot accidentally use a null or missing place because the compiler rejects code that doesn't explicitly deal with the `None` case. In Go, a missing element might return a zero-value struct that silently prints empty strings. In Python, it raises an `IndexError` at runtime.

### Honest Trade-Offs
Rust compiles slower than Go and has a steeper learning curve than both Python and Go. For a one-off 20-line script, Python is faster to write. But for a CLI you intend to distribute, run in CI pipelines, or maintain over time, the compile-time guarantees and single-binary deployment are genuinely better.

## Summary: When to Use Which

| Use Case | Best Choice |
|----------|-------------|
| Prototyping, data science, scripting | **Python** |
| Network services, cloud infrastructure | **Go** |
| CLI tools, systems programming, embedded | **Rust** |

The weather CLI is a perfect Rust use case: it's a standalone binary that should start fast, use minimal memory, and never crash from null pointer exceptions or type errors at runtime.

---

## Deeper Topics to Explore

- **Lifetimes and Borrowing:** How Rust prevents data races and use-after-free without a GC
- **Pattern Matching:** How `match` and `enum` in Rust are more powerful than Go's `switch` or Python's `if/elif`
- **Traits:** Rust's answer to interfaces—more flexible than Go's and without Python's duck typing uncertainty
- **Macros:** Code that writes code—more powerful than Go's code generation, safer than Python's metaprogramming
