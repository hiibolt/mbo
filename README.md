
## Steps Chosen
I chose to do all four major targets, I had nothing better to do tonight.

I started with **Order Book Reconstruction** to gain a better understanding of how the library worked, as it looked to be the most difficult comparatively. I was correct in this thought, and succeeded as a result.
- 1. **Boilerplate**: Approx. 5-10m to select a stack and solid dependency foundation.
- 2. **Order Book Reconstruction**: Approx. 25m to read and comprehend `databento` (first-time user) and 20m to implement full data reconstruction
    - 2.1. I diverted from their documentation to implement logging when an `MboMsg` does something impossible like cancelling a non-existant order (for example, `MboMsg` #2). When such a thing occurs, it is safey ignored and tracing logs are emitted.
- 3. **Recollecting**: Approx. 10m to drink water and close-read requirements to begin planning the future structure of the repository, server, and frontend.
- 4. **Data Streaming**: Approx. 10m to convert repository to a webserver, 10m to write JSON export endpoint, and 15m to write a failure-safe TCP streaming endpoint.
- 5. **Endpoint Documentation**: Approx 20m to learn how `utopia` (Swagger/OpenAPI docgen tooling for Rust) works and implement

## AI Usage
I used Claude Opus 4.1 for dense, difficult tasks requiring heavy verification and Claude Sonnet 4.5 for less intense tasks such as test verification, by-line documentation, and rapid templating.

List of usage:
- Moving example `databento` code to use more verbose `anyhow` reporting
    - A simple matter of asking it to inspect and replace all `unwrap` and `expect` calls
- Adding full `tokio_tracing` coverage
    - An otherwise tedious task, made simple with AI!
    - Required following AI cursor and inspecting all changes to verify there's no secret leakage