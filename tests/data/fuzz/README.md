# Fuzz regression corpus

Inputs that once crashed the compiler, kept as files so each case survives
independently of the generator that produced it. `tests/property_tests.rs`
replays every `.md` here and asserts only that parsing returns.

Add a file whenever a generated input finds a crash, named after the cause.

| File | Case |
|---|---|
| `merge_conflict_eq_variable.md` | object name collides with one inside the built-in `Equation` type |
| `merge_conflict_enum_fence.md` | same collision, reached through an enumeration fence |
| `merge_conflict_term_angle.md` | same collision, with a truncated `Term:` and an invalid option value |
