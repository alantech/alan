## Frequently Asked Questions

### Could you combine an ELI5 (explain it like I'm five) and a sales pitch to try and convince me?

Computers have increasingly more CPUs/cores, as opposed to more powerful ones. Multithreaded code is hard to reason about and error prone, but it is also necessary today to take advantage of all the computing resources in a machine. The goal of Alan is to be similar to multithreaded Go or Java in performance, but similar to Python brevity and simplicity.

### Did you originally intend to make a non-Turing-complete language?

No, originally just pondering why existing large scale backend deployments and data processing frameworks require so much engineering effort to keep them running in production, then realizing that Turing completeness was the cause of the complexity and many distributed systems' outages and that it wouldn't be possible to solve that problem with a "better framework." That's when working on a new language started.

### Which parts of Alan make it Turing Incomplete, specifically?

Classical loops and recursion is not possible. This does not mean that you can't loop over data or write recursive algorithms. Alan encourages expressing iteration using [arrays](https://docs.alan-lang.org/builtins/array_api.html) when possible because it can be parallelized by the AVM.

However, there are algorithms people need to write that are inherently recursive or sequential. For that Alan reintroduces a way of [writting Sequential Algorithms](https://docs.alan-lang.org/std_seq.html) in controlled manner that still allows the AVM to be able to plan around these algorithms and provide the guarantee of termination.

### What would would you recommend *not* trying to write with Alan?

Generally programs where one needs more control over how code the is parallelized, even if it is less convenient, should probably use Go, Rust or Erlang. This is akin to how you might prefer C or C++ over Go or Java if you really need the memory management to be more performant or precise.

### Have you published any papers with the research on compile time parallelization?

Most of it is synthesis of existing ideas that just haven't been applied yet, with a couple of things we believe are [our insights](https://alan-lang.org/alan_overview.html#parallel-computation-and-the-problem-of-turing-completeness) (but we haven't exhaustively read the literature to confirm/deny that).

### Do you have a brief description about how Alan is different (besides syntax) from Rust and Erlang?

**Erlang:**
Alan went with an event-based model instead of an actor-based model. They are two sides of the same coin (you can represent one with the other) but the event-based model is more well-known and understood to developers as a whole than the actor model due to its use in Javascript, VisualBasic, Logo, etc. In both Erlang, parallelism has to be done across actors which makes data-level parallelism more complex. Alan has support via the array methods to perform that sort of parallelization right now, and the compiler computes a dependency graph of all instructions making up an event handler. We hope the runtime will be able to dig up even more parallelization options from this DAG in the future.

**Rust:**
Rust gives the full power to developers to determine how and where they want concurrency and parallelism, and gives them escape hatches from their constraints with the unsafe blocks. However Alan provides neither, but uses the constraints it has to automatically find these concurrency and parallelism opportunities for you. Alan's memory story is something in-between Rust and Java; Alan lacks GC pauses because deallocations only happen outside the event loop once an event handler has run its course. This means Alan's memory management frees up data less frequently than Rust's memory model and with a frequency similar to that of a traditional GC.

### How does Alan get rid of array out of bounds and other common runtime errors?

We built the Alan VM in Rust and borrow from the Rust syntax a bit too. We do [error handling](https://docs.alan-lang.org/error_handling.html) like Rust and solve array-out-of-bounds by returning Result<T> types which forces the user to handle this at compile time. This shouldn't be too tedious due to shorthand notation around error handling and Alan supporting static [multiple dispatch](https://en.wikipedia.org/wiki/Multiple_dispatch) which allows functions to have additional implementations that accept a Result type as an argument.

### Where do you want to go with this language? Do you want to grow a community around it?

Our goal is use Alan to build backends in production that require concurrent or asynchronous execution. We want to work with codebases for concurrent programs that are nimbler and easier to reason about than codebases that use multithreading constructs.

Yes, we would like to create a community of contributors to work on the Alan runtime and compiler with us, or to build a healthy ecosystem of third party libraries for the language.

### How (do you plan to) distribute workloads on different machines?

So the short-term answer to this question is simply fronting Alan processes with a load balancer. But we do intend to make that story better over time. First priority is working on getting @std/datastore to coordinate shared, mutable state within a cluster. This is the [RFC](https://github.com/alantech/alan/blob/main/rfcs/008%20-%20Safe%20Global%20Data%20Storage%20RFC.md) to get it working across cores in a single machine.

Once that's done, we can add TLS support and then pull in a load balancing layer based on the same balancing logic @std/datastore uses for data balancing which would also make directing traffic to the nodes likely to have the data locally possible. We also hope to eventually be able to have the cluster move the compute automatically to the relevant nodes in a more fine-grained fashion than through load balancing.

In the long-term, there's nothing semantically restricting the language from actually performing a single logical compute distributed across multiple machines (a map on a logical array larger than local memory) making out-of-memory bugs a thing of the past if coupled with an autoscaling backend, but there's going to be a *lot* of code to write and architecture to design to detect when that's necessary and switch to it.

### How (do you plan to) do automatic GPGPU delegation or other heterogeneous computing cores?

Alan already encourages writing code that can be parallelized through arrays and events, so it is possible to implement automatic GPGPU delegation in the future. That said, the state of GPGPU is such a disaster, OpenCL/CUDA/OpenGL/DirectX<=11/DirectX12/Vulkan/Metal, all with strengths and weaknesses and platform issues and no way to avoid one or more of them since GPU drivers don't allow direct access to hardware and have compilers baked into them these days...

Still thinking about the best approach to take on the GPGPU front: do it "right" up-front but support the main backends relatively early, or prove it out with a single universal backend that might literally fudge numbers on you depending on your hardware. Also, not a priority until we've got the language really working, so post-v0.2, at least. If you have experience in this realm and want to contribute, please reach out!

### Is there any interoperability?

We are still scoping out [FFI support](https://github.com/alantech/alan/pull/60/files) for bindings that plays nice with the auto parallelization that happens in the VM. Alan (transpiles to Javascript)[https://docs.alan-lang.org/transpile_js.html] which still offers IO concurrency but without the parallelization.

### The "source installation" necessary to contribute to Alan requires Python, Node.js and Rust. Why are there so many dependencies?

We went with tools we were familiar with or believed would accelerate our ability to prove to ourselves that this could work, and that is reflected in the language implementation as it exists today, but we have always intended to rewrite the Alan compiler in Alan. Then only the runtime(s) would be in different languages.

Currently, the compiler is written in Typescript, the main runtime in Rust and the secondary runtime in Javascript. Python is required due to a limitation of a dependency to [nexe](https://github.com/nexe/nexe) and not something we are using within the project. The upcoming nexe 4.0 release will hopefully resolve this. That said our dependency on nexe is intended to be temporary.