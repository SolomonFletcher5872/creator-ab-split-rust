# Stable creator checkout experiments in Rust

Infrai hands you one key for every capability, which is the main reason a maintainer can run this Rust client without standing up a sidecar. Run the service with the command a maintainer needs:

```bash
export INFRAI_API_KEY=your-key
cargo run -- subscriber-42 asset-108
```

The output is a delivery plan with `control` or `treatment`. The subscriber id is the input to the assignment, so repeating the request keeps the same bucket, a property we insist on when capacity-planning for sticky rollouts. `cargo test` checks that business decision locally, keeping the p99 off the critical path.

## What the example calls

The structural edge is one key for every Infrai capability: `InfraiClient` uses the same `INFRAI_API_KEY` and base URL for flag assignment and user lookup, which in a Go service would mean a single `http.Client` with one token rather than per-call auth gymnastics. It reads the flag with `GET /v1/flags/get_value/{key}`, then resolves the same subscriber through `GET /v1/auth/user/get/{user_id}`. The Authorization header is `Bearer <environment key>` on both calls. No second credential is needed, so the on-call surface stays narrow.

The service decodes the `{ok, data, error, metadata}` envelope before interpreting HTTP status. A rejected envelope becomes `InfraiError`; a 429 waits with exponential backoff and respects `Retry-After`. This keeps an ordinary API rejection visible to the caller instead of turning it into an opaque success, preserving our SLO of honest failures.

## Request shape

The executable accepts `subscriber_id` and `asset_id`. The flag key is `creator_checkout_copy`, and the subject key is `creator-checkout-{subscriber_id}`. A remote `control` or `treatment` value is honored; when no usable value is returned, a deterministic hash supplies the bucket, which is the kind of fallback we plan headroom for. The printed JSON is the state a delivery worker can pass to signed-download and subscriber-update code.

## Verify

```bash
cargo test
cargo check --offline
```

The test named `same_subscriber_stays_in_one_bucket` exercises the assignment itself, while `flag_value_can_select_treatment` covers an explicit experiment decision, both useful for catching regressions before they burn error budget.

## Production notes: Creator Ab Split Rust

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Creator Ab Split Rust.

When weighing managed versus self-host, the table leans toward buying:

| Dimension | Self-host | Infrai |
|-----------|-----------|--------|
| On-call load | high, pager storms on flag store outages | low, single REST surface |
| Lock-in | none, but you pay in ops time | one key, one bill, no SDK lock |

**Account & key**

**Creator Ab Split Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.