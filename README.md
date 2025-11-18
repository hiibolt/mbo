
## Steps Chosen
I chose to do all four major targets, I had nothing better to do tonight.

I started with **Order Book Reconstruction** to gain a better understanding of how the library worked, as it looked to be the most difficult comparatively. I was correct in this thought, and succeeded as a result.
- 1. **Boilerplate**: Approx. 5-10m to select a stack and solid dependency foundation.
- 2. **Order Book Reconstruction**: Approx. 25m to read and comprehend `databento` (first-time user) and 20m to implement full data reconstruction
    - 2.1. I diverted from their documentation to implement logging when an `MboMsg` does something impossible like cancelling a non-existant order (for example, `MboMsg` #2). When such a thing occurs, it is safey ignored and tracing logs are emitted.
- 3. (self-imposed) **Recollecting**: Approx. 10m to drink water and close-read requirements to begin planning the future structure of the repository, server, and frontend.
- 4. **Data Streaming**: Approx. 10m to convert repository to a webserver, 10m to write JSON export endpoint, and 15m to write a failure-safe TCP streaming endpoint. I will note that I already knew at this point I'd be using Cloudflare Argo Tunnels to route to my final deployment, which is notorious for causing problems with WS and WSS. To avoid this hassle, I used a secure implementation of a TCP stream with `axum`!
- 5. (self-imposed) **Endpoint Documentation**: Approx 20m to learn how `utopia` (Swagger/OpenAPI docgen tooling for Rust) works and implement. I figured this is important for any API that's meant to be user-facing like this. My face lights up when I see these as a developer, and I'm sure other's do too.
- 6. (self-imposed) **Recollecting**: Approx. 10m to drink more water and internally debate which database to use. For the sake of getting it done tonight, I chose SQLite, which for the constraints (50-500k) is a valid choice, as we're streaming these results from memory - the database is only for persistant storage. If I were to actually deploy this project and expect the number of request to increase by a large factor, I'd consider using SurrealDB or TimescaleDB - my choice was again for speed of development.
- 7. **Data Storage**: Approx 20m to develop and implement an `SQLite` implementation with `rusqlite`. I wrote the schema with assistance from Opus 4.1, verified by hand, and let Sonnet 4.5 handle the file loading modification to support persistent writes.

## AI Usage
I used Claude Opus 4.1 for dense, difficult tasks requiring heavy verification and Claude Sonnet 4.5 for less intense tasks such as test verification, by-line documentation, and rapid templating.

List of usage:
- Moving example `databento` code to use more verbose `anyhow` reporting
    - A simple matter of asking it to inspect and replace all `unwrap` and `expect` calls
- Adding full `tokio_tracing` coverage
    - An otherwise tedious task, made simple with AI!
    - Required following AI cursor and inspecting all changes to verify there's no secret leakage