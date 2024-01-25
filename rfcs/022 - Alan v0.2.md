# 022 - Alan v0.2 RFC

## Current Status

### Proposed

2024-01-24

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <isv.damocles@gmail.com>

## Summary

This is not a normal RFC. There will be no other approvers and this is essentially a reboot of the project. The structure of the remainder of the RFC is more of an essay, with some background on what challenges the existing Alan language is up against, what I think the "good parts" of the language are, some thoughts on the context of the software world this project is in, and finally a proposal for a new Alan.

## Background

Alan was intended to automatically scale across cores and machines for you by restricting your code to a not-quite-Turing-Complete (per event) set of flow control, eschewing arbitrary loops for array-level operations, and blocking recursion entirely. This was so the runtime would be able to look at your loop-like code and decide at runtime if it makes more sense to execute it sequentially on the same thread or spread the work across multiple threads (and if the data lived in the experimental datastore, decide which part of the work should be done on which machine to minimize total latency).

The issue is the "parse problem." For any kind of work you're doing, you can break it up into three phases: parse the inputs, do the business logic, serialize the outputs. General business logic worked just fine with the restrictions we put in place, and we were able to write code that would take any data structure in Alan and serialize it into JSON, but parsing logic for naturally-recursive type data is most naturally written in recursion. It is possible to write a non-recursive [LALR-style parser](https://www.reddit.com/r/programming/comments/z7q5ay/comment/iy8qrhw/?utm_source=share&utm_medium=web2x&context=3), but the code is much more difficult to write and not "natural feeling" compared to its recursive cousin (Alan's own compiler uses a [recursive decent parser](https://en.wikipedia.org/wiki/Recursive_descent_parser) so it feels unfair to keep that power from Alan's users).

However, if we allowed recursive code that would mean the compiler and runtime would not be able to make predictions about the actual running time, and by being recursion, cannot do anything but run it in a single thread. This breaks the autoscaling logic as it cannot make the recursive code parallelize at all, and it can't even model the total running time, so it can no longer tell if parallelizing the remaining code is worth it from a latency standpoint or not. Even the LALR-style parsing will likely produce a poor prediction model, not knowing which branch of the state machine is going to be called how many times, it needs to assume the maximum number of calls of the most expensive branch, which will be a significant overestimate of the sequential runtime so it may under-parallelize concurrent logic because the parse path has a high expected latency.

This may not seem like a "big" deal since this is true of other languages, but you (ideally) use a new language because of what it brings to the table to make your life easier as a developer, and that was the *intended* value, and unlike Rust's safety and the `unsafe` blocks, since *every* non-trivial program has "parse" sections, often interwoven with their business logic, it's a much harder sell to put that into a corner of the syntax space and tell the users to only use it if they know what they're doing.

With that conclusion, we paused work on Alan and explored other possible tools to build (for the next two-and-a-half years).

## What Worked in Alan 0.1

While we struggled to realize the original promise of the language, there were many things *right* or at least in a very promising direction, pretty early on, and we sometimes lamented not having access to them in other programming languages. In no particular order:

* While `alan deploy` depended on a centralized service we ran (because it was much simpler to write that way), it was able to deploy and (coarsely) autoscale a cluster of a service across all three major cloud providers, and across the various datacenters and availability zones within each of them with a singular shared "datastore" that maintained duplication such that the outage of any one node, availability zone, data center, *or* cloud provider would still keep all of the relevant data available, and could do interesting things like grab the "nearest" copy of data, understanding this networking hierarchy, or it could be forced to use the "canonical" copy of the data where writes go to first, or it could *push* the execution context (closure scope and everything) to the node that had the relevant (presumably large) data to perform the computation and then return to the original node the output of the execution. (The goal was to eventually merge all three and let the VM decide which action to take based on the size of the data and execution context.) All of this was as transparent as if the data was stored locally in memory.
* There was a rudimentary but generalized way to mock dependencies built into the module system of Alan itself. This was originally envisioned as a way to allow mocks in unit tests without needing to add expensive (and error-prone and security-brittle) runtime reflection into the language (or an error-prone and mentally-damaging macro system), but we realized could be used to also act as a capabilities management mechanism by replacing all or parts of the standard library itself with stubs in third-party libraries, thereby preventing them from ever accessing features that you don't want them to have access to, or injecting new behavior as desired. The aggressive inlining and dead code removal of the compiler helped with this -- if you don't even use the part of the library that depends on some part of the standard library you don't want it to use, that code won't even be evaluated, so a "busted" stub won't matter.
* The reduction to only types and functions without classes, but making the first argument of a function act like the `self` or `this` property for method-style syntax, the syntax for mapping one and two parameter functions into operators at the desired precedence, coupled with interfaces that could be used as constraints in generic functions and were evaluated on the visible types in a scope against the visible functions in the scope contributed to a very dynamic-feeling language while still being statically typed. If you imported a library and it had this wonderful fluent interface, but you *wished* it had some method, you could just write a function in your own code and then use it mixed in with all of the other fluent methods rather than having to jump between fluent and functional style back and forth, and there would be no ambiguity because you would see that function definition in the file (or you would have explicitly imported it from another file) and with this behavior being known it would be perfectly clear what's going on. If that library provided a super useful type, but they forgot to make it implement the `Sortable` interface, you could similarly just implement the `gt` and `lt` functions for it in your scope and now you can sort an array of them without needing to modify the original library to do so. If you wanted a DSL for operations and within the domain the symbols are unambiguous you could define your own operators and their precedences and then use it. If you want to override the behavior of one of the methods/functions for that library, you just write a new function with the same name within your scope and since your defintion is the "newer" of the two (in ties of matching function types, the most recent definition in terms of scope hierarchy wins) within the context of your own code, that function will be used, even if it is not called directly but is called by one of the other methods that you call. It's like monkey patching and duck typing but all of it statically typed and explicitly defined so the compiler is still making sure you're not screwing up. If we had implemented the implicit interfaces RFC, it would also have been possible for it to all be done without any type annotations at all -- looking *exactly* like a dynamic language but with all of these static guarantees.
* Even at a coarse level, the parallelization story was pretty good. The event system allowed totally independent execution contexts within the program to run, well, independently. The array operations could do array-level parallelization for you, and assuming the closure they were running had no recursion within it, could automatically decide if it should break it up across multiple cores or keep it single threaded, rather than requiring you to explicitly force it to be parallel or single threaded like other parallelization-focused languages require. Then there was the datastore for multi-machine coordination that *could* have been subsumed into the Array and HashMap types to make automatic data distribution possible (but we didn't for a few reasons, the main one being that the "parse problem" was discovered shortly after datastore implementation started, but also some concerns about top-level types having different performance characteristics to types defined within a function). Finally, IO operations like filesystem reads, http requests, etc, could be automatically executed concurrently when they didn't depend on each other (with dependencies being determined by the graph of inputs and outputs of the low-level AVM opcodes) though for that we reversed course a bit and made it an opt-in experience because http requests could be dependent on hidden state in the remote service with nothing in your own code making it clear that you can't execute both HTTP PUTs (for instance) concurrently because the first one written needs to complete before the second one can be accepted even though you don't use the output of the first one in the second one.
* The memory management simply being "throw it away at the end of an event" is not *ideal*, but sure was easier to implement in the AVM and also made concerns about GC pauses irrelevant during execution -- your code should run a predictable amount of time when it has the thread and then it spends some amount of time after the event is done to free its memory. The set of free threads is not fully predictable by the AVM with this setup, but which threads it can use for this new event *are* known to it when that event pops up, so it's a pretty nice set of trade-offs assuming you don't have long-running events in your service. It would probably be better to have a dedicated "sweeper" thread that frees no-longer needed memory and the execution threads can mark unneeded memory if they've been running for a long time, with all of it being marked at the end of the event.
* The `root` scope just being another source file that contains definitions implicitly included in every file would make learning the language simpler. (We didn't include the ability to define your own root scope, though, because that would cause an explosion of incomparable dialects of the language that can't interop with each other.)
* The Rust/Haskell/ML-inspired type system with no implicit nulls and the lack of try-catch is *amazing* and I hate having to work with languages that have either.

## What Fell Flat

As you can see, there are several bits of Alan where it does things "better" for backend services that have many concurrent users, which makes sense as that was our bread-n-butter at prior jobs. While the deployment, cluster management and coordination, and auto-parallelization are the flashiest bits of Alan, they may actually have the least impact overall because:

1. Businesses that run these sorts of backend services tend to already have their own deployment system, managed by an SRE (or equivalent) team, and getting them to give up control of that would be a hard sell, which would lead to the Alan binaries being run as isolated executables in separate Docker containers in a Kubernetes cluster, or whatever is the "standard" way to do this when you're reading this. ;) When run that way, the automatic coordination via the datastore is simply broken, so only the event and array parallelization would work, making it directly comparable to Go in that respect.
2. For most backend services like this, there are *far, far* more users than there are available threads across the cluster, so array-level parallelization is not just not desired, but would tend to be a latency-inducing headache as the set of events that don't need more than one core pile up behind the one that took over all of the cores on the machine. The AVM would recognize this situation and would therefore choose to never parallelize the array operations, but that means all of the work to do that auto-parallelization would be wasted, and the pain put on the developer to push them away from writing looping or recursive code would be unjustified.
3. In the rare cases where a particular event would benefit from array-level parallelization *and* having more threads than users is justifiable, this would likely be written into a specialized service by senior software engineers and monitored separately from the main flow. Even if it could be more efficient for Alan to do this automatically and this extra compute is allocated to the primary cluster to do so, just the *risk* of impacting the latency of the other paths, or accidentally not scaling the cluster enough because this event volume was lower than expected for a time and then running into an underprovisioning situation is probably enough that a company that has these needs would prefer their engineers to handle it explicitly rather than let Alan implicitly handle it.

These points are *not* true for brand new startups, and the automatic scaling logic would carry them far without needing to invest in all of this process, but:

1. Is this a strong enough point that these startups are more likely to survive than startups that do things the "normal" way, such that a new normal can develop over time?
2. Would this stick through the hypergrowth phase and into the larger-stage startup / regular software business phase, where the company grows more conservative (to conserve whatever cash-generating machine they have created) causing engineers to shy from being blamed for problems they can't control, and therefore are unwilling to leave that control to an automatic system, even if it is superior to their own manual efforts?

For most web-based startups, I think the answer to both of these questions is "no." Which startups live or die likely has a low correlation to their programming language of choice. YC has pushed that meme a couple of decades ago with Python and other dynamic languages versus the statically compiled languages of that era, because of the kind of developers you could find and the speed at which they could generate business logic in those languages. That may or may not have been true then, but the developer velocity of Alan versus other languages would be comparable at the beginning, not significantly better, and would only improve *as it scales* not when you can run everything off of a developer's laptop shoved into a closet. Once you're scaling like that, you can probably already afford a team of developers and sticking to more commonly known languages to get a bigger hiring pool is probably the better choice there.

The automatic deployment, scaling, and auto-parallelization work was all incredibly fascinating and definitely *valuable*, but it wasn't what an Alan v0.1 should have been focused on -- it's a project for an Alan v2.0 after Alan v1.0 is a successful language with a growing community. After finding a value proposition that answers those questions with a "yes."

Beyond that, there were the "expected" shortcomings of a prototype-level language. The AVM's performance only barely beat CPython for basic arithmetic test applications. Compiling to Javascript almost always produced actual real-world faster performance. Being a static language, this could certainly be improved, but who would adopt a language designed to scale that has such a terrible baseline performance? Sure, we could use all of your cores across a cluster of machines, but you could just use one core on a single machine in a different language for the same performance, so why adopt it?

We also had some compiler bugs that meant we had to write some code in the standard library in an awkward way to avoid them, and deeply-nested conditionals would occasionally compile *incorrectly* (there's a branch rewriting the first stage of the compiler that resolved that, but had some other parts not quite working yet when we paused development).

Finally, there was no language server, making IDE integration pretty poor (though we did write syntax highlighters for Alan, so readability of the code was decent).

## The Elephant in the Room

[As everyone knows, AI is going to do all the programming for us](https://www.newyorker.com/magazine/2023/11/20/a-coder-considers-the-waning-days-of-the-craft) so why even make a new language right now? We even explored this more deeply with [Marsha](https://github.com/alantech/marsha), a minimalist markdown-based "language" that generates *working* Python code for you by having you describe the function and provide examples to use as a test suite, and then iterating with an actual build-and-test step with the LLM until it passes the tests.

But despite getting more stars on Github in a week than any other project we worked on, we saw zero usage of Marsha in the wild, even just to try it out, demonstrating a very poor product-market fit. Perhaps still modeling things as a kind of source code and going through a CLI-based compilation process is enough to put off the non-programmers, while trust in the output actually working and accelerating their development put off the programmers.

What AI is doing right now is simply reinforcing existing, popular languages, by drawing on the knowledge based built up on the internet of these languages and regurgitating it to users, adding even more headwind to new programming languages. This is on top of the headwinds caused by: minimal-to-nonexistant third party libraries, lack of a language server to add autocomplete/error-checking/etc to their developer tools, lack of a linter to enforce coding standards, possibly even a lack of coding standards having been established for the new language, lack of third-party documentation on Stackoverflow and friends, etc, etc.

To overcome all of this, a new language has to provide a *significant* advantage over its competition. Alan 0.1 brought a lot to the table, but none of it met this standard. The breadth of all of these improvements in all of these different areas also reduced the focus of the language. I liked *all* of them, of course, but some of them could come later after a solid problem has been solved and actual language users *want* these secondary QoL features.

## Proposal for Alan 0.2

The capabilities of the AVM (distributed datastore, auto-parallelization, etc) would make a good foundation for an ACID-compliant distributed database that doesn't suck (can handle partial outages gracefully), but as a *programming language* that would only provide an advantage if you were willing to have your database embedded within your application, which I just don't think people are ready for, yet. ;)

So Alan 0.2 shouldn't have an AVM. It should compile into an actual binary (or other executable artifact like the current JS compilation target).

Dynamic-like syntax with Rust-like safety guarantees during compilation feels like a great target to aim for, but actually not needing to define the type explicitly requires no recursion in the language (as currently exists, though it makes some things more difficult). Why?

Consider the following JS function:

```js
function lispyInt(num) {
  if (num == 0) {
    return undefined;
  }
  const val = lispyInt(num - 1);
  return [ val ];
}
```

This function returns a nested set of Arrays matching the number provided, so `lispyInt(0)` returns `undefined`, `lispyInt(1)` returns `[]`, `lispyInt(2)` returns `[[]]`, etc. So if we tried to write a type for this in typescript, we can easily declare `num: number`, but the function return type would be `undefined | Array<undefined> | Array<Array<undefined>> | Array<Array<Array<undefined>>> | ...` and continue on infinitely. You might be saying it goes up as high as the number type can go, but Javascript gets really weird with super high numbers...

```
damocles@elack:~$ node
Welcome to Node.js v20.7.0.
Type ".help" for more information.
> 2**54
18014398509481984
> 2**54 - 1
18014398509481984
>
```

So even if you though you were being technically correct, you weren't. ;)

But this is just an example to demonstrate the point that the return type could change based on the recursion depth for recursive functions, and it won't always be obvious what it would be until after running. Forcing the developer to specify the return type is how you get out of this, by making this sort of craziness uncompilable in the first place. But then you don't have a dynamic-looking language that is fully statically typed. It will need to do cycle detection (which it already does to just ban recursive functions) and then require these functions to be fully typed before it will compile. (When I say "recursive functions" this would also apply to arbitrary looping, too, so we couldn't include `for` or `while` without also requiring typing for any function using those, too.)

Truly automatic typing requires a non-Turing-Complete type system, since to compile the type inference needs to always halt, and this makes certain kinds of code not representable in the language.

Now, most recursive code doesn't do this. The type of the output is *usually* static, and perhaps recursive functions could be opportunistically represented in the language by testing branches for possible type outputs and ignoring the recursive paths for type data. This could still fail on mutually recursive functions, or could produce type inference that doesn't match reality (the aforementioned recursive type depth example would be typed as just `undefined`), so I'm not fully sold on it. It's better to leave recursion out of Alan 0.2, despite the parse problem, and return to this in 0.3.

While bringing auto-parallelization and a strong, automatically inferred type system to highly concurrent distributed backend systems *would* improve things there, it's a marginal improvement without being able to optimize the data layer. But there are two areas of computing right now that are not-quite Turing complete, one of which deals with massively parallel computation, that same one has a fractured and fragmented ecosystem by hardware type, and both suffer from a high barrier to entry that the other properties of Alan could fit well in: Untrusted kernel extensions (eBPF) and GPGPU.

eBPF allows one to write untrusted code that is guaranteed to halt that the kernel itself runs on behalf of the user -- primarily focused on auditing and network traffic shaping, because being in kernelspace allows these things to be done far more efficiently than the constant back-and-forth between kernel- and userspace that requires pauses via interrupt handlers. Basically if you have a tight loop that involves a syscall, eBPF is a good fit. eBPF is a C-like syntax with [a more limited set of types and operators](https://www.kernel.org/doc/html/latest/bpf/standardization/instruction-set.html), and a different "standard library" of functions you can call exposed by the Linux kernel. It's pretty esoteric and requires much more in-depth knowledge of how the Linux kernel itself works (and is not very stable, the kernel functions, called [kfuncs](https://www.kernel.org/doc/html/latest/bpf/kfuncs.html#core-kfuncs), can be deprecated at any time) and you need to be at a scale where just adding a few more VMs to your cluster *won't* help or is too expensive of a path to take.

A language to compile to eBPF would have very few users due to the rarification of the problem space *and* how little such experts would need an abstraction layer like that. Further, automatic parallelization simply doesn't matter in eBPF because your code is single-threaded and expected to use as little CPU time as possible to accomplish its goal. The documentation goes into depth on [the bytecode output and how to optimize it](https://www.kernel.org/doc/html/latest/bpf/classic_vs_extended.html). For eBPF users, [abstraction is for academics](https://twitter.com/rbranson/status/662118397447049216).

GPGPU is another story entirely, though. There's an explicit language designed for GPGPU called OpenCL, except using it is very difficult because specialized drivers need to be installed for your operating system and it simply doesn't work out of the box in most cases. On top of that it is considered a difficult language to learn. Then there's CUDA and ROCm, two other GPGPU languages, but they only support nVidia and AMD graphics cards, respectively, so you can't write once, run everywhere with either of them.

If you want hardware-independent, platform-independent GPGPU, you've basically been out-of-luck for a while. The best solution was to write a separate GPGPU kernel in the GPU library and shader language that's best supported for each platform you want to compile to (DX12 and HLSL for Windows, Metal and MSL for OSX/iOS, Vulkan and GLSL for Linux/Android, WebGL and a more primitive GLSL for the browser), which is a *lot* of redundant work.

Fortunately, the [wgpu](https://github.com/gfx-rs/wgpu) library for Rust now exists and provides a unified API on top of all of that for you based on the new WebGPU standard. This reduces the amount of work significantly and gives you write once, run everywhere, *but* you have to learn the WGSL shading language to [write your GPGPU code](https://github.com/gfx-rs/wgpu/blob/06e9876adfcdb7b0d99c0d78ac8d2931705e0425/examples/src/hello_compute/shader.wgsl) then you have to [write a lot of boilerplate in Rust, which you may not know](https://github.com/gfx-rs/wgpu/blob/06e9876adfcdb7b0d99c0d78ac8d2931705e0425/examples/src/hello_compute/mod.rs) to initialize the GPU, compile the WGSL shader and load it into the GPU, allocate memory to transmit data to and from, the GPU, grab the input data and copy into that buffer, execute the code, copy the results back to the CPU memory, do some mangling to get the types back the way they should be, and finally return the output to the user.

It's *a lot* of work that the developer has to understand intimately, *and* understand when it is beneficial to go through this expensive process versus just throwing a bunch of CPU cores at it. It also requires knowing two statically typed languages and how to translate between their type systems.

Alan feels like it could fit in that niche *very* well, by abstracting away the decision making of when to use GPGPU and when to just use multi-core, and automatically moving the necessary data between the two. Alan is still typed, but keeping the recursion restrictions the types should be automatically inferrable and there should be equivalent code for any function in both Rust and WGSL due to those restrictions.

A dynamic-looking language that works seamlessly with your GPU is a solid target, and it opens up interesting secondary pathways: the automatic clustering and distributed compute of Alan 0.1 could be revived on top of Alan 0.2 (as 0.3?) to make it trivial to cluster GPGPU workloads, or perhaps the GPGPU could be extended into the classic GPU world and easy-to-use graphics programming coupled with new events for mouse/keyboard/joystick/gamepad inputs and built-in audio outs could make it a solid language for game programming, especially when compiled for the web to avoid app store fees?

Regardless of whichever possible future Alan takes, the immediate target of making Alan the GPGPU language for the average developer feels like a strong niche, but some of the current language features feel like they don't quite fit anymore (besides those related to clustering and deploying, which are already planned to be removed).

First, the language-level event system seems less of a solid fit if much of the software to be written in it won't take advantage of it, and the plan to just clean up memory at the end of each event fails entirely if the whole program logic resides in the `on start` event handler. The memory management logic absolutely needs to be handled differently, but if the plan is to have the compiler generate Rust code, we could rely on Rust's own lifetime management instead of potentially pulling in a GC. But should events themselves leave the language spec? It would make the language easier to learn, but since the language doesn't allow for infinite loops, actually writing a backend service in the language (instead of just calling out to Alan code from another language) would probably be impossible without the event system, so I think it should stay.

But the event system should probably be improved from it's current iteration, making it more like a `match` syntax in Rust; being able to add an event handler for specific HTTP request paths, for instance, currently isn't possible; all of the logic must inevitably be baked into the same global request handler, bloating it up considerably (it is possible to manually separate it by defining sub-events and the main event handler figures out which sub-event to route to, but it's more tedious than just calling the relevant function directly).

The "ternary" operator doesn't behave anything like the C operator of the same structure (it evaluates both branches, puts them into an array, and then chooses which array value to return to you, instead of determining which branch to evaluate and return. Internally the language was converting if statements into functions that potentially return a value, even though that's not how it works syntactically in Alan, but it would probably be better to use Rust-style `if`s and drop the fake ternary.

The type syntax is directly stolen from Java, but a Typescript/Rust-style type syntax would be more legible in most cases. Complicating this by including both feels like a not good idea, though. If automatic type inference works out, though that wouldn't be the focus in Alan 0.2, this *probably* doesn't matter too much, but one uniform, easy-to-understand syntax would be better.

Most languages are nowhere near as flexible with operators as Alan 0.1 is. There are technically zero built-in operators in the compiler; the `root.ln` file defines them, which functions they map to, *and* the operator precedence, but they could only be made up of a small number of ASCII symbols. That last restriction was to make it unambiguous to parse things like `3+4` without spaces because the operator name couldn't be a valid function name nor a constant like a string or number or boolean. This means it's impossible to define an operator that's Pythonic like `3 and 4` that would be read "legibly", and it would encourage potentially inscrutible operators for any library that wanted to make some for their own DSL.

Also, currently only infix and prefix operators are allowed, no postfix operators that would be very useful for representing units.

Just giving up on the operators and using Rust's operators, for instance, could be one way to go, but it feels antithetical to the language to just represent execution as functions and operators and methods are just syntatic sugar. But allowing arbitrary operators could produce unintelligible DSLs that fragment the language harshly, and looser naming of the operators will make parsing very difficult and statements ambiguous. (Is that a variable or an operator? Is it an infix operator or a postfix operator?)

For Alan 0.2 this will be left as-is, but as known tech debt to tackle in the future, since we're not going to 1.0. (In some ways operators with precendence tables are a mental tech debt of how we teach mathematics, but a language that can't do "normal math" statements is viewed as odd.)

The import syntax being incredibly explicit about what is being imported was great for self-documentation, but was painful in practice because of how much typing you had to do (and the lack of multi-line import statements, but that's minor and easily correctable). There was an experiment at the end where importing an interface would also bring over any type and all methods/operators that match said interface implicitly near the end, making it very much like a class but more verbose to define (you have to write the type declaration and functions and then redeclare the functions in the interface and optionally the type properties). Perhaps syntactic sugar (a `class` syntax or something like `derive interface Foo from <type>`) would help here.

The two-root import logic (`modules` and `dependencies`) and how it can be used to override dependencies' own dependencies is interesting but complicated, and using it for mocks in tests was convoluted. Doing this at the syntax level with something like `mock import @foo with @bar` can make the testing situation better and making `~` or similar simply be an alias for the project root (as determined by the root file being compiled) can make utility files easier to work with.

Capabilities-based hardening of dependencies could be done using a `without` syntax, indicating the module that should be replaced with a no-op mock, which could be done automatically for the compiler for any module, eg `import @sketchy/module as lessSketch without @std/http` would disable access to the `@std/http` library *but* give the library a seemingly functional no-op version so it still compiles correctly, but can no longer phone home (or whatever). This could potentially be combined with the `mock` syntax into something like `import @sketchy/module as lessSketch mock @std/http with ./my/fake/http` that actually does some useful logic, such as monitoring if/when it tries to phone home, or only allows certain http operations through while blocking known bad ones.

With these additions, we don't need the two headed FS search and can go back to a more normal single dependency tree structure (with the special carve-out for `@std`).

The `@std/datastore` library had a lot of interesting work done enabling a shared global mutable state across distributed instances, using Rendezvous Hashing to determine ownership of the canonical storage as well as where to put duplicates for latency reduction in reads. The work to transfer execution scope between AVM instances so it can execute on the same machine as the source data and then transfer only the result back seems particularly relevant to passing work back and forth between the CPU and GPU.

The way `@std/datastore` implemented things exposed underlying details to the user and didn't really bake it into the language itself, but it was an experiment to demonstrate viability (and at that it succeeded very well). Pushing the mutation logic to the "owner" of the data could allow for mutable global variables that are serialized per variable but parallel across a large number of them, eliminating some of the downsides to parallelization with global mutable state. There would be complexities where multiple global variables are mutated in the same function but perhaps that could fall back to a more classic lock mechanism under the hood? But only if the read-to-write interval of statements overlaps between the two, otherwise the code that reads until it finally writes something back to the global variable can be plucked out as a closure and fed to the execution queue for that global.

This sort of closure scope transferance may be much more complicated without the AVM, but presumably a similar process will be necessary when transferring execution to the GPU and back again. Also, because of the cost of moving back and forth between the two boundaries, avoiding a bunch of CPU RAM -> GPU RAM (and vice versa) memcpys will be key. It *may* make sense for Alan 0.2 to allow explicit statements to run things on GPU vs CPU, and perhaps automatically deciding which to do is written in Alan itself as part of the root scope code, but the goal will be to infer this automatically and without issue. If that is unreachable in the short term, then so be it.

Finally, the Typescript/Rust split between the compiler and runtime needs to go. We did a good job of getting the compiler to run fast (faster than ANTLR in Java, even), but we could go faster if it was in Rust, as well, and with the new focus of compiling into Rust, it would make more sense for it to be written in Rust, as well.

### Alan 0.2 Proposal Summary

Alan 0.2 will:

1. Rewrite the compiler in Rust.
2. Compile Alan into Rust, first single threaded, then reviving CPU parallelism, then wgpu-powered GPU parallelism, then choosing the correct one at runtime.
3. Change the import logic to bake in mocking at the language level rather than being a weird trick during module resolution.
4. Bake `@std/datastore`-like logic to allow global mutable variables that can pass closures between threads (and hopefully demonstrate CPU->GPU closure transferance).
5. Minor syntax/type changes, though the biggest would possibly be Rust/TS-style types instead of Java-style (eg, `A | B` vs `Either<A, B>`)

Alan 0.2 will *not*:

1. Tackle the "parse problem"; unbounded recursion isn't good for *automatic* GPGPU code generation. Making parsing easier will come later.
2. Implement FFI or other binding logic is also not included here. It would be tempting to just add a way to automatically expose Rust functions and types in Alan, but if you can pull it in anywhere, you break the automatic parallelization logic again since it can't do that to any of that code.
3. Implement type inference. It should hopefully be a "fast follow" for 0.3, but it's not necessary to demonstrate value with GPGPU programming.
4. Fix the situation with operators.
5. Add any syntactic sugar, the new implementation may in fact remove some sugar if necessary.
6. Bootstrap the compiler. This *may* be something Alan 0.3 does, or it may not.

## Expected Timeline

Unknown. ;) But the goal is by Summer 2024.

