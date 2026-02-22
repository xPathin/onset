# Onset

GTK4/libadwaita application for managing XDG autostart entries on Linux.

## Build & Test

```bash
cargo build --release
cargo test
```

## Conventions

- **Error handling**: `anyhow::Result` everywhere, `.with_context()` for path-related errors
- **File I/O**: Always use `write_atomic()` (temp file + rename) for writing .desktop files
- **GTK state**: `Rc<RefCell<T>>` for shared mutable state in callbacks — this is standard GTK-rs
- **Lazy statics**: `once_cell::sync::Lazy` for compiled regexes and XDG paths
- **Desktop entry format**: INI-like with `[Desktop Entry]` section header, key=value pairs
- **Startup delay**: Wraps exec as `sh -c 'sleep N && exec CMD'`, parsed back with regex
- **Field codes**: `%u`, `%U`, `%f`, `%F`, `%i`, `%c`, `%k` are stripped from Exec lines (meaningless for autostart)

## Important Constraints

- Must remain XDG Base Directory compliant
- Desktop entry files must follow the freedesktop.org Desktop Entry Specification
- Atomic writes prevent corruption if the process is interrupted
- The `Hidden=true` field disables entries without deleting the file
- `OnlyShowIn`/`NotShowIn` and `TryExec` affect `EffectiveState` but don't prevent display
