# AgenticConnect FFI Reference

## C-Compatible Interface

AgenticConnect provides FFI bindings via `agentic-connect-ffi` (cdylib + staticlib).

### Functions

#### `acnx_version()`
Returns the library version as a null-terminated C string.

```c
const char* acnx_version(void);
```

**Returns:** Version string (e.g., "0.1.0"). Caller must free with `acnx_free_string`.

#### `acnx_free_string(ptr)`
Free a string returned by an `acnx_*` function.

```c
void acnx_free_string(char* ptr);
```

### Language Examples

#### C
```c
#include <stdio.h>
const char* acnx_version(void);
void acnx_free_string(char* ptr);

int main() {
    char* version = (char*)acnx_version();
    printf("AgenticConnect %s\n", version);
    acnx_free_string(version);
    return 0;
}
```

#### Python (ctypes)
```python
import ctypes
lib = ctypes.CDLL("libagentic_connect_ffi.dylib")
lib.acnx_version.restype = ctypes.c_char_p
print(lib.acnx_version().decode())
```

### Building

```bash
cargo build -p agentic-connect-ffi --release
# Output: target/release/libagentic_connect_ffi.{dylib,so,dll}
```
