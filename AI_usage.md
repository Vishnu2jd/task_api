1.  Tools Used

  - Claude: Used as a coding co-pilot for rapid prototyping and boilerplate
    scaffolding.
  - IDE and Coding Assistance: RustRover and rust-analyzer for autocompletion,
    imports, and real-time compiler checks.

2.  AI-Assisted Code (Scaffolded)

  - Struct definitions and payload models in src/models.rs.
  - JWT extraction logic and custom extractor traits in src/main.rs.
  - Basic database querying logic inside the core handlers (login, verify_2fa,
    create_task, assign_tasks, view_my_tasks).

3.  Manually Written / Modified Code

  - Cargo Setup: Initialized the project and configured dependencies in
    Cargo.toml.
  - Zero-Dependency 2FA Generation: Replaced the rand crate completely with
    native chrono nanoseconds (timestamp_subsec_nanos()) to resolve version
    conflicts.
  - Axum Trait Compiler Fix: Added #[axum::async_trait] to the FromRequestParts
    extractor block to resolve compilation errors (error[E0195]).
  - Schema Validation Mapping: Manually mapped database task items to
    TaskResponseItem inside the handlers to strictly match the requested JSON
    validation schema (e.g., stringifying IDs and renaming assigned_to).
  - Manual Server Wiring: Wrote AppState structures, registered router
    endpoints, implemented the local email log handler, and wrote the main entry
    point.
