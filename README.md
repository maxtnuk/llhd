An attempt at writing a low-level hardware description toolkit. First up is a
parser and code generator for VHDL.

## Design Guidelines

- source files have suffix `.cpp`
- header files have suffix `.hpp`
- sources and headers both live in the `llhd` directory
- everything lives in the `llhd` namespace
- files may be grouped into directories for better readability and structure
- files containing sub-namespaces of `llhd` must be placed in a directory with
  the same name as the namespace (e.g. `llhd::vhdl::ast` classes are to reside
  in the `llhd/vhdl/ast` directory)
- directory names are lowercase
- includes are sorted by
  - origin (LLHD headers first, then third party, then standard headers)
  - hierarchy level (`llhd/compiler.hpp` before `llhd/allocator/Allocator.hpp`)
  - capitalization (lowercase before uppercase)
  - alphabet
- `using ...` and `using namespace ...` only in sources, absolutely not in the
  headers (except for rare cases of `namespace ... = ...` applicable to the
  entire codebase)
- type names are capitalized (e.g. `TokenContext`)
- function names are camel-cased and start with a verb (e.g.
  `allocateMultipleObjects`)
- variable names are camel-cased (e.g. `allocatedObjects`)
- accessor functions are named `set...` and `get...`
- there are to be absolutely no underscores in type, function, or member names
