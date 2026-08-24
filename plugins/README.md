# SecureForge Plugin System

## Adding File Signatures

### Simple (TOML)

Create a `.toml` file in `plugins/signatures/`:

```toml
[[signature]]
name = "My Custom Format"
category = "Documents"
header = "\\x00\\x01MAGIC"
footer = "\\x00\\x00END"
max_size = "100MB"
```

### Advanced (Lua)

Create a `.lua` file in `plugins/scripts/`:

```lua
signature {
    name     = "My Custom Format",
    category = "Documents",
    header   = "\x00\x01MAGIC",
    max_size = "100MB",
    validate = function(data)
        -- Custom validation logic
        return data:byte(7) == 0x42
    end
}
```

Plugins are hot-reloaded — no restart required.
