# Bounded Channels Learning Example

## What is this?

This example demonstrates **bounded channels** in Rust, which are fundamental for the Praxis architecture.

## Concept

A **channel** is a communication pipe:
- **Producer** (Node) sends events → Channel → **Consumer** (Gateway) receives events

A **bounded** channel has a maximum capacity. When full, the producer **waits** until space is available.

## Why Bounded?

**Problem without bounds:**
```
Fast LLM generates 1000 tokens/sec
Slow client receives 100 tokens/sec via network
→ Queue grows infinitely → Memory explodes → Server crashes
```

**Solution with bounds:**
```
Channel capacity = 1000 events
When full, LLM waits until client consumes some events
→ Automatic backpressure → Memory controlled → System stable
```

## Run the Example

```bash
cd learning/channels-example
cargo run
```

## What to Observe

1. **Two producers** (simulating Nodes) send events FAST (100ms)
2. **One consumer** (simulating Gateway/SSE) receives events SLOW (300ms)
3. Channel capacity is **5 events**

**Expected behavior:**
- Producers will send 5 events quickly
- Then they'll **WAIT** for consumer to catch up
- You'll see gaps in "Sent" logs (producers blocked)
- This is **backpressure** in action!

## Code Walkthrough

### Creating a bounded channel
```rust
let (tx, rx) = bounded::<String>(5);  // capacity = 5
```

### Producer (Node in Praxis)
```rust
tx.send(event).await;  // Waits if channel is full
```

### Consumer (Gateway in Praxis)
```rust
while let Ok(event) = rx.recv().await {
    send_to_client(event);  // Stream via SSE
}
```

## Relevance to Praxis

In Praxis:
- **Node** = Producer (emits StreamEvents during execution)
- **Gateway** = Consumer (receives events, streams to client via SSE)
- **Bounded channel** = Protects server from slow clients

```
LLMNode → [bounded channel: 1000] → Gateway → SSE → Client
         ↑
    If Gateway is backed up (slow client),
    LLMNode waits automatically
```

## Experiment

Try changing values in `main.rs`:

1. **Increase capacity**: `bounded(100)` → Producers won't block as often
2. **Speed up consumer**: `consumer(rx, 50)` → Less backpressure
3. **More producers**: See how multiple Nodes compete for channel space

## Key Takeaways

1. Bounded channels prevent memory overflow
2. Backpressure is automatic (no manual throttling needed)
3. Essential for scaling to millions of users
4. Protects server from slow/malicious clients

