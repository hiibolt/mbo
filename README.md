## Steps Chosen
I chose to do all four major targets and all engineering requirements, I had nothing better to do with my day and figured I'd enjoy the rare challenge.

I started with **Order Book Reconstruction** to gain a better understanding of how the library worked, as it looked to be the most difficult comparatively. I was correct in this thought, and succeeded as a result. **CR** = Core Requirement, **PE** = Production Requirement, **SI** = Self-Imposed
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
- 13. (SI) **Final Testing** - Approx. 1hr, as I grappled with learning about crossed markets to try and undestand how a negative spread could make sense in this context.
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
    - Needed help from Sonnet to debug SSE connections not dropping cleanly - ended up developing custom RAII guard solution, which was really cool! 