# FocusOS Coding Standards

## Rust

Feature-based modules; no global mutable state; strong typing; errors via `Result`, never `unwrap()`/`panic!()` outside of tests and truly unreachable branches; logging via `tracing`.

## React

Functional components only; hooks only, no class components; strict TypeScript (no `any` without a comment explaining why); reusable components live in `packages/ui`; feature-based folder structure, not type-based (i.e. `features/timeline/`, not a giant `components/` dumping ground).

## Database

Every schema change is a migration, never a hand edit; parameterized queries only, no string-built SQL; index every column used in a `WHERE` or `ORDER BY` in the queries from the analytics spec.
