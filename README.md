# `mbo`
> Databento Market Simulation for Batonics built with Rust, Svelte, and Nix.

## [Application](https://mbo.hiibolt.com)
## [Monitoring](https://mbo-grafana.hiibolt.com)

## Implementation Details
I chose to do all four major targets and all but one engineering requirement, I had nothing better to do with my day and figured I'd enjoy the rare challenge.

I'd also like to note that document may have used to contain an instruction to allow live data (presumably core requirement #5), and while I would have loved to do it, it's a paid API that requires approval. If you'd like to see this implemented, I designed the database, backend, and frontend with it in the back of my head - all I require is a pre-approved API key.

### Core Requirements
1. **Data Streaming**: Stream the MBO file at 50k-500k messages/second over TCP (design for scalability)
    - This was done quickly thanks to the absolute monster that is Rust's asynchronus ecosystem. Tokio and Axum are love-letters to backend developers worldwide who're fed up with NodeJS try/catch.
    - I used TCP streams as they're less complex, easier to debug, and most importantly, compatible with Cloudflare Zero Trust Tunnels.
2. **Order Book Reconstruction**: Build an accurate order book with p99 latency <50ms
and output as JSON
    - Thanks to Axum, this was trivial - during testing, p99 never went above 5ms. Even under 100 concurrent connections in a container with 2vCPUs and 2GB RAM, it didn't break a sweat.
    - I was unable to find the upper limit because my development machine would die trying to hold more than 130~ connections exhausting its `fork` limits, but CPU usage on the production server didn't even flicker.
    - I suspect it could handle multiple thousand concurrent requests thanks to the extremely fast request turnaround time of 30~ms for a full upload (limited by my machine's connection, not the backend!) and p50 of 300 microseconds. 
3. **Data Storage**: Persist to a time-series database
    - I chose to implement this with `rustqlite`, a safe wrapper on SQLite. I added functionality for further expansion should the backend want to serialize live data - it wasn't a requirement, but the option was there. It primarily serves as a persistent backup.
4. **Deployment**: Dockerized application with clear setup instructions 
    - I went hardcore with the CI/CD for this. Fully automated testing, frontend/backend builds, and multiple tiers of Docker Compose deployment for development, observability, and production.
    - However, for production itself, I wrote an extensive Kubernetes deployment, as that's the true enterprise standard. I deployed it on my local K8s cluster, and used Cloudflare's Zero Trust suite to point my domain at it.

### Production Engineering
6. **API Layer**: REST or WebSocket API supporting 10-100+ concurrent clients reading the order book
    - Done. I've confirmed by hand the backend can easily handle over 100 concurrent SSE connections at once. It's incredibly difficult to capture this via Grafana, because of how quickly the backend operates. The connections are completely satiated in under 35ms~ on average (30ms DL, limitation of my networking, ~5ms p99 latency), so Grafana can't even catch them without some luck. 
7. **Frontend**: Web interface visualizing live order book updates
    - Done, I chose to build using Svelte, Skeleton, TailwindCSS Bun, and Nix. I went with a black on white color scheme because I think it looks cool.
8. **Configuration Management**: Externalized config with no hardcoded credentials
    - Done, all variables are flexible and can be specified with `.env` or environment variables. The repo contains a `.env.example` for reference. I use `direnv` personally, as the project already uses Nix Flakes for development environments (Docker Compose is an option for those without).
9. **Reproducible Builds**: Dependency locking and documented build process
    - Yes, absolutely. Bun and Cargo are known for near-perfect dependency locking with their `Cargo.lock` and `bun.lockb` files. Additionally, Nix dev environments are locked with `flake.lock`.
    - The self-documenting and human readable nature of Dockerfiles and GitHub Action YAML files means it's easy to have visibility into the build process, and you can see it happen live on GitHub!
10. **Testing**: Unit tests, integration tests, or correctness proofs for order book logic
    - All of the above are implemented using `cargo test`. As part of the CI/CD, before a package is pushed to the registry, it must pass all Rust backend tests. All of the above mentioned are included.
11. **Performance Optimization**: Achieve higher throughput (targeting 500K msg/sec with p99 <10ms)
    - Absolutely. I was able to run a 10s test that accrued 170M messages total, after which my machine promptly gave out - from the number of open connections (from the test, the backend was fine all throughout). All throughout, p99 never went above 5ms thanks to Axum.
12. **Observability**: Metrics (latency percentiles, throughput), structured logging, or distributed tracing
    - All of the above. There's container-level tracing with `tokio-tracing`, structured logging with environment filtering, and full metric deployment with Prometheus and Grafana. 
13. **Infrastructure as Code**: Terraform, Pulumi, or similar for deployment automation
    - I haven't done IaC work just yet, but my K8s mentor plans to teach me within the month - it does interest me!
14. **Multi-Environment Setup**: Dev/staging/prod configurations with CI/CD pipeline
    - Absolutely, there are multiple levels of containerization:
        - Full-local: Nix Development Shells
        - Dev/Observability/Psuedo-Prod: Multiple tiered Docker Compose YAMLs
        - Full-prod: Declarative Kuberenetes YAMLs
15. **Resilience Testing**: Demonstrate graceful handling of failures (connection drops, pod kills, etc.)
    - Yes. Graceful shutdowns, custom RAII guards for handling dropped connections kindly, and thanks to the `Drop` trait Rust weilds, it was mostly done for me in advance.
    - The production Kubernetes deployments are all self-healing and let K8s do its thing to bring actual to desired state.
16. **API Reliability**: Idempotency, retry logic, proper error handling
    - Absolutely. All operations are atomic and idempotent, as I made use of Rust's `Arc`, `RwLock`, and other concurrency types.
    - All error handling is done with a contextual error handling crate I love called `anyhow`, which makes RCAs an delightfully simple. There isn't a single `unwrap` or `expect` call in the entire core backend.
    - The frontend has type verification, Toast-based error handling and reporting, and absolutely zero chance of an unreported issue.
17. **Security**: Supply chain verification, dependency auditing, SBOM generation
    - I have `dependabot` set up in the repository, which automatically runs `cargo-audit` and emails me if a dependency has a major CVE or issue, and prompts for major updates.
    - `cargo-sbom` is extremely straightforward and gives the SBOM visibility with one command. The output is verbose so I omitted it, but the tool exists, works well, and one command.
18. **Correctness Verification**: Prove the order book never violates exchange rules (price-time priority, valid quantities)
    - This was the most interesting for me, because I've very new to `databento`. I made two major decisions (in a real environment, I wouldn't have made either and would immediately reach out for advice!)
        - MBO messages that operate on things that don't yet or never exist are discarded with a `warn`-level log/trace event. I decided to do so since it's simply a limitation of having only a snapshot of the market.
        - No crossed markets. Leaving it as allowed, there was a negative spread, which makes sense programmatically because of how these events work, but on the other hand it mades no real-world sense. To compensate, I added a correction layer that filters to only allow properly matched operations.

## Full Process
I started with **Order Book Reconstruction** to gain a better understanding of how the library worked, as it looked to be the most difficult comparatively. I was correct in this thought, and succeeded as a result. 

**CR** = Core Requirement, **PE** = Production Requirement, **SI** = Self-Imposed
- 1. (SI) **Boilerplate**: Approx. 5-10m to select a stack and solid dependency foundation.
- 2. (CR) **Order Book Reconstruction**, **Configuration Management**: Approx. 25m to read and comprehend `databento` (first-time user) and 20m to implement full data reconstruction
    - 2.1. I diverted from their documentation to implement logging when an `MboMsg` does something impossible like cancelling a non-existant order (for example, `MboMsg` #2). When such a thing occurs, it is safey ignored and tracing logs are emitted.
- 3. (SI) **Recollecting**: Approx. 10m to drink water and close-read requirements to begin planning the future structure of the repository, server, and frontend.
- 4. (CR, PE) **Data Streaming**, **API Layer**: Approx. 10m to convert repository to a webserver, 10m to write JSON export endpoint, and 15m to write a failure-safe TCP streaming endpoint. I will note that I already knew at this point I'd be using Cloudflare Argo Tunnels to route to my final deployment, which is notorious for causing problems with WS and WSS. To avoid this hassle, I used a secure implementation of a TCP stream with `axum`!
- 5. (SI) **Endpoint Documentation**: Approx. 20m to learn how `utopia` (Swagger/OpenAPI docgen tooling for Rust) works and implement. I figured this is important for any API that's meant to be user-facing like this. My face lights up when I see these as a developer, and I'm sure other's do too.
- 6. (SI) **Recollecting**: Approx. 10m to drink more water and internally debate which database to use. For the sake of getting it done tonight, I chose SQLite, which for the constraints (50-500k) is a valid choice, as we're streaming these results from memory - the database is only for persistant storage. If I were to actually deploy this project and expect the number of request to increase by a large factor, I'd consider using SurrealDB or TimescaleDB - my choice was again for speed of development.
- 7. (CR) **Data Storage**: Approx. 40m to develop and implement an `SQLite` implementation with `rusqlite`. I wrote the schema with assistance from Opus 4.1, verified by hand, and let Sonnet 4.5 handle the file loading modification to support persistent writes.
- 8. (PE) **Frontend**: Approx. 2hrs to build a Svelte, Skeleton, Tailwind, and Bun-based frontend. I use all but Skeleton regularly in personal work, and let Sonnet handle templating the site with Skeleton for sake of rapid iteration. It only actually took around an hour, but I went out for dinner halfway through.
    - 8.1. I'm a big fan of Svelte, it's somewhat Rust-like in that it behaves in an opinionated but predictable manner (and has great docs!). One you internalize how development is meant to be done with modern Runes, it's a cinch.
    - 8.2. Bun is amazing. It's funny, they actually don't have a Bun caching action on GitHub Actions because Bun installs faster than the overhead of launching a GitHub Action. If you want a laugh, take a look at [one of my favorite GitHub replies of all time](https://github.com/oven-sh/setup-bun/issues/14#issuecomment-1714116221). It also makes using TypeScript extremely easy, which is nice for coordinating types with verification between the frontend and backend.
    - 8.3. Tailwind needs no introduction. Lightweight, minimal, and LLMs are great at quickly iterating quality CSS - Tailwind training data is vastly better than raw CSS training data.
    - 8.4. Skeleton is commonly used for pages like this, and Opus recommended it during brainstorming, so I viewed the docs and quickly agreed.
- 9. (PE) **Testing**, **Correctness Verification** - Approx. 15m to develop, verify, and implement a series of tests based on the provided file. I chose to use the standard `cargo` testing suite because it's what I'm most familiar with. There are more comprehensive suites that can test both the frontend/backend simultaneously, but I was planning to use GitHub Actions to test seperately, since you get a pretty nice dopamine hit from seeing that `X/X` with a checkmark on the GitHub UI.
- 10. (CR, PE) **Deployment**, **Reproducible Builds**, **Multi-Environment Setup** - Approx. 1hr45m. I decided to use GitHub Actions for builds and CI/CD, which is notoriously slow but also free.
    - 10.1. Since this is a Nix-based project, it could have also been possible to do with the Hydra build system, but that has a lengthy drop-in time. 
    - 10.2. I chose to use Docker for containerization, it's standard and I know it well. 
    - 10.3. I chose to split the Docker Compose into base/prod/dev, although Compose has mostly been superceded by K8s, it's a nod to homelabbers who enjoy its simplicity.
- 11. (PE) **Observability**: I decided to set up a full-blown Prometheus + Grafana dashboard. Usually, this is major overkill, but I've deployed it for so many work, research, and personal projects at this point that it's become a bit trivial.
    - 11.1. Prometheus is undeniably one the best tools for the job. I selected it for its wide-spread usage, ease of setup (past the initial learning curve), and great integration with Rust via the `prometheus` crate.
- 12. (PE) **API Reliability**, **Performance Optimization**, **Resilience Testing** - Approx. 2hrs, I put everything I'd built in previous steps to the test with this task.
    - 12.1. The test works, but run at your own risk.
    - 12.2. Great success! Exteremly performant, even with terrible specs (2vCPUs, 2GB RAM)
    - 12.3. There were a huge number of tweaks that I made to push performances to its limits, which ended up being the bulk of my time.
    - 12.4. My major limiting factor ended up being unable to properly stress test because the number of open connections was hitting Linux fork usage caps - the backend never once went down during this.
- 13. (SI) **Final Testing**, **Final Documentation** - Approx. 1hr, as I grappled with learning about crossed markets to try and undestand how a negative spread could make sense in this context.
    - 13.1. My initial thoughts all the way back at the start were to accept a negative spread, but I decided that conflicts with the idempotency ask of #16.
    - 13.2. I changed my approach, and modified the `Book`keeping implementation that `databento` provided. In a real situation, I would immediately consult others for advice and wait to proceed for solid information.

## AI Usage
I used Claude Opus 4.1 for dense, difficult tasks requiring heavy verification and Claude Sonnet 4.5 for less intense tasks such as test verification, by-line documentation, and rapid templating.

Absolutely no LLMs were used in the writing of this `README.md` out of principle.

List of usage:
- Moving example `databento` code to use more verbose `anyhow` reporting
    - A simple matter of asking it to inspect and replace all `unwrap` and `expect` calls
- Adding full `tokio_tracing` coverage
    - An otherwise tedious task, made simple with AI!
    - Required following AI cursor and inspecting all changes to verify there's no secret leakage
- Frontend Templating with Skeleton and Tailwind
    - I carefully watched it design the frontend structure and gave feedback to progress it to a visually appealing end result.
- Docker Compose
    - Writing `.dockerignore` files was done with Sonnet, as it's an otherwise tedious task. I was careful to monitor for secret leaks and repo ballooning before confirming.
    - I contracted Opus to build the Dockerfiles, giving it examples from past projects and carefully monitoring to ensure quality output.
- Prometheus
    - Both of Anthropic's models are highly skilled at writing Prometheus dashboards, so with supervision I felt confident to let it write the YAML spread. 
- Stress Testing
    - I asked Opus to build a neat stress-test that has 1M msg/s, 500 concurrent connections, and consistent stress.
    - Needed help from Sonnet to debug SSE connections not dropping cleanly - it ended up helping developing custom a RAII guard solution, which was really cool to supervise and learn from.
- Kubernetes
    - My final deployment namespace for production was massive (13 YAML files). I asked Sonnet to help me consolidate them into concise, readable documents, which it did a wonderful job with.