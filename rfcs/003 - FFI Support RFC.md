# 003 - FFI Support RFC

## Current Status

### Proposed

2020-06-10

### Accepted

YYYY-MM-DD

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author

- David Ellis <david@alantechnologies.com>

## Summary

While `alan` should be perfectly suitable for the vast majority of backend application development, sometimes a lower-level language where you can be more precise on the exact execution is necessary for higly-critical code where performance is necessary.

Because `alan` itself is automatically optimizing execution to take advantage of the available hardware, a traditional FFI interacts poorly with the `alan` runtime. By isolating FFI calls by default to a single "IO-like" thread, FFI can work without negatively impacting the runtime's estimators, and with special annotations it can be "promoted" to being able to be run in parallel in the IO threadpool.

## Expected SemBDD Impact

This would be a patch-level change to the project assuming we were at 1.0.0+. New code would be possible, while existing code should work unchanged.

## Proposal

The underlying truth behind the `alan` language is that different programming languages are more suitable to certain tasks than others. The vast majority of commercial backends would be better suited as `alan` as it was very much designed with their needs in mind, but sometimes the compiler and runtime will not be able to figure out the best performing set of operations for something that has critical performance needs, or you need access to existing code from another language and wrapping it in a network-aware service for consumption is not the right approach. That is when an FFI is neeeded.

The main issue with an FFI in the context of `alan` is that it introduces code that the compiler and runtime cannot reason about from a performance or resource perspective. This can interact poorly with automatic parallelization features in the runtime as it cannot estimate how long the underlying FFI function call will take, how many resources it requires, or if it is safe to make the calls in parallel.

So by default, the FFI system needs to assume that the execution time will be unpredictable and that it should not be executed in parallel. This calls for an "IO-like" thread that FFI calls are serialized onto, so no FFI call can be executing at the same time as any other. This "resolves" the issue but puts a significant bottleneck on the optimization possible -- any code that uses an FFI is no longer parallelizable, though it will not have any adverse effects on any other code running.

However, it should be possible, with some light annotations, for the FFI bindings to be "understandable" to the runtime. Short FFI calls can be specifed as "sync" to be inlined like a CPU opcode, FFI calls that are safely parallelizable can be specified as "par" similarly allowing them to be used in parallel computation.

This can decompose into four opcodes, `ffi` (FFI with no annotations, must be isolated), `ffis` (ffi with the sync flag only, must also run on the isolated thread but can yank the entire execution fragment with it for better performance), `ffip` (FFI with the parallel flag only, can execute like a "normal" IO opcode), and `ffisp` (ffi with both flags, can execute like a "normal" CPU opcode).

Assuming a C-like API, they could have many arguments (more than the two the runtime has for its own opcodes) but this can be resolved by converting the argument list into an array and passing the array of argument values to the relevant opcode. Similarly a C function can only return a single value that can fit within our own fixed value space, but it may represent a pointer to data so a special set of opcodes to convert to and from poitner data to HandlerMemory objects will be necessary, but fixed data should be usable without any conversion necessary. Some C APIs assume that already-allocated blocks of memory are passed to it, where it may be possible to give the C code access to a new HandlerMemory's internal u8 vector and be able to avoid memory copies at all there.

All of these opcodes involving memory translation to-and-from the C FFI should be safe to execute in parallel as the runtime will guarantee that the relevant C code operating on it has finished executing before it makes changes to it. However, this assumption is invalid if the FFI library itself spawns an OS thread and continues to make changes to the memory in question, this assumption is invalid and could crash the runtime. It will be the responsibility of the FFI binding author to make sure memory is used safely if they intend to bind such a library, but FFI binding is not done lightly, so we can assume the developer knows that "Here Be Dragons" the moment they embark on this project.

The loading of the library would use the standard OS-specific logic, with resolution of the shared library and its functions done at runtime, not compile time, though the binding code wrapping it would present a static API that other authors can rely on.

To make sure that this is true, the FFI binding author would need to register an event handler for a `loadLibrary` event that provides it with access to a `load` function that the library needed by the bindings. This event would precede even the `start` event (though the runtime doesn't need to know this, more on this later), immediately terminating execution if a specified library could not be loaded -- the compiler would know how many `loadLibrary` handlers there are and the `load` function could trigger the `start` event after the total number of loads has been reached (by checking a value added to the global memory against an internal counter).

If a `loadLibrary` event is found by the compiler, it can rewrite that to be the `start` event from the runtime's perspective, and the `load` function would trigger a `normalStart` custom event that the code registered on the `start` event has been rewritten to listen to, instead.

A sort of "FFI" could also be defined for the js-runtime, but it would by definition have to be very different from the C-style FFI described above, so it is out of scope for this RFC.

### Alternatives Considered

The primary alternative is the "null" alternative: no FFI -- just use sockets to communicate to a sidecar process or web service written in another language to access the needed functionality. This is not a solid long-term solution as it requires a mixed backend and you have to make a decision of what functionality should live where, producing suboptimal results for the developers (and arguments against usage of `alan`).

A second alternative is to make it easy to write extensions to the runtime, allowing the user to add new opcodes accessible to a custom `@std` library. This would be simpler for us, but would require the developer to know Rust (when most code you want to bind will still be C-like), know the internals of the runtime itself to create opcodes that don't break the contracts of the rest of the runtime, then write a binding-like custom `@std` library in alan, so it puts a heavy burden on the binding author. It also would require that anyone using the binding recompile their own runtime, which may not compose well with any other binding, if at all, and would also be an AGPL violation if done for their own company but not published as open source, which would likely cause legal to ban use of the language.

Providing a "classic" FFI binding without annotations could also work, but would be very limiting in performance since nothing can be assumed about it, which would *encourage* the use of OS threads in the binding to get back parallelism possibilities, but those (potentially multiple per binding in the same runtime) threads would compete with the runtime for resources and throw off any attempts at optimization by the scheduler as progress on the CPU code fragments would lose any level of predictability.

## Affected Components

This would require a new `@std/ffi` library, extra logic in the compiler for rewriting events (or extra logic in the runtime for different event behavior), and significant work in the runtime to do dynamic library loading and create all of the opcodes necessary to call into it safely and transfer data into and out of it.

## Expected Timeline

This project will likely take at least a month to get right and done safely. The majority of the work would be in the runtime to first use Rust's own FFI functionality to load external libraries and then test with hand-crafted test files of the new opcodes, ignoring the `start` event issue by simply inserting a 1 second `wait` before using the opcodes.

Once it seems to be working assuming calls to it are delayed until after loading, the work on the compiler to rewrite the events can be done.

