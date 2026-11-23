# Ryan's working notes behind the product docs (2026-11-22)

Raw thinking, kept for the detail the distilled docs leave out. VISION.md, EXPERIENCE.md and MILESTONE-1.md win where they differ.

- how to think about building a software factory
    - i think it’s best to have one agent own one task
    - different types of tasks require different involvement from me
    - quick brainstorm
        - testing
        - reviewing code
        - infra
            - making sure CI is green
        - code quality and architecture
        - tech debt
        - writing out a spec of what we want to build
            - iterating on this spec
        - taste
        - iterating with user feedback
        - high level planning
        - monitoring
        - requirements, success criteria, milestone
        - there’s various types of work and things to align on
            - processes
            - code architecture
            - user experience
            - language design
        - some notion of writing down goals and trying to let the agent make reasonable decisions themselves?
        - we want to break down something very complicated and align on parts independently
        - ok maybe a lot of this is pretty related to organizing humans
            - we need an org chart that’s aligned to the product
            - we need metrics and goals and planning for each team
        - high level planning
            - what are all the components we’re working on
            - ok so there’s some horizontal layers
                - processes
                    - workflows
                    - how to keep track of notes
                - infra
                    - ci
                    - deploys
                - code architecture
                    - codebase structure
                    - just general high level design
                    - probably some number of shared libraries and api surfaces
                    - the core to answering this requires us to decide on what to build
                    - some of this is horizontal, some of this is vertical as well
            - what’s all the stuff we need people to work on
                - infra: ci, deploys, release, justfile, mise
                - compiler core
                    - requirements
                        - we match the specs and are correct
                        - horizontal layer that serves everyone else
                    - components
                        - cpl check: it just works
                        - range analysis
                        - diagnostics: incredibly high quality error messages that are super intuitive
                - interpreter
                    - debug mode is within 5x of c++
                - codegen + perf
                    - under 32kb across our entire corpus. submittable to all main online judges
                    - perf: p50 <1.1x, p95 <1.2x, max <1.5x, max with constant optimization <1.1x
                    - solid benchmark suite that we can use to hill climb on
                    - eventually: be able to do things like world class simd auto vectorization
                - std
                    - we match the spec, are correct, and are performant (within 1.2x of optimized c++ code on librarychecker)
                - spec
                    - tastefully designed language that’s super pleasant to use
                    - solid test suite that we can use to hill climb on
                    - idioms
                        - skill for agents to write idiomatic code
                        - well documented in a horizontal format that everyone else can use
                        - fully translated into lint rules
                - lint
                    - catches all unintuitive things
                    - serves as a tutorial for the stdlib since we have a lint for everything that teaches you how to write it more cleanly
                    - highly configurable, easy to disable certain rules that you don’t like
                - fmt
                    - it just works. we can make it configurable but we should have very opinionated defaults
                - tools
                    - the cpl cli, how one actually uses it and structures their files, competitive companion integration
                - editors
                    - our lsp editor integrations just work
                    - time travel debugging + things like setting up competitive companion and native uis in vscode and jetbrains
                - debugging
                    - stress testing: need a design here but basically it should just be a super smooth experience
                    - trace + trace viewing
                        - at most like +20% to interpreter speeds
                        - super high quality ux
                - website
                    - home page
                    - learn
                        - through guides
                        - through practice problems
                    - docs (we own cpl docs as well)
                    - playground
            - higher level pieces / tactical stuff today
                - how smooth is the language to use?
                - how easy is it to onboard and learn the language? is there something i can share with friends for them to try out
                - how good is the tooling to actually use it in a contest and solve problems with it
                - i think the first milestone we want to work towards is: good enough for me to share with a friend and for them to be able to meaningfully try it out and give meaningful feedback
- ontology from chat
    - layers
        - direction: why are we doing this?
        - product: what should exist from the user’s perspective
        - system: how will the product work?
        - work: what needs to change
        - organization: who is accountable for what
        - execution: how do we turn intentions into shipped changes
        - evidence: how do we know what is true
    - product
        - direction: mission, principle, goal, metric, milestone
        - product: user journey, capability, requirement, specification, quality attribute
        - examples
            
            
            | Concept | Meaning | CPL example |
            | --- | --- | --- |
            | Mission | The overarching purpose | Make turning a contest idea into an accepted solution fast and reliable |
            | Principle | A durable decision rule | Avoid hidden performance costs |
            | Goal | A desired outcome | Users can meaningfully solve problems using CPL |
            | Metric | A quantitative indicator | Time from completed algorithm idea to accepted submission |
            | Milestone | A recognizable, testable state of progress | Ready to share with a friend |
            | User journey | An end-to-end sequence of user actions | Install → learn → implement → test → submit |
            | Capability | Something the product enables | Stress testing |
            | Requirement | A property that must hold | Generated submissions fit judge limits |
            | Specification | The precise contract for behavior | The exact semantics of ranges |
            | Quality attribute | A system-wide dimension of goodness | Performance, usability, reliability, debuggability |
        - vertical decomposition: user journeys, use cases, and capabilities
        - horizontal decomposition: domains, components, platforms, and interfaces
    - system
        - domain, component, interface, architecture, design, invariant
            
            
            | Concept | Definition |
            | --- | --- |
            | Domain | A cohesive area of technical or product responsibility |
            | Component | A concrete implementation unit with a defined purpose |
            | Interface / contract | The promises components make to one another |
            | Architecture | The global organization of components, responsibilities, and dependencies |
            | Design | A proposed solution to a specific technical or product problem |
            | Invariant | A property that must always remain true |
            
        - examples
            - Domain: compiler infrastructure.
            - Components: parser, type checker, typed IR, diagnostic engine.
            - Interface: how the LSP obtains type information.
            - Architecture: the LSP and codegen consume shared semantic analysis rather than reimplementing it.
            - Design: the representation of source positions in the IR.
            - Invariant: a program accepted by the type checker must not be rejected by codegen merely because of an unsupported type combination.
    - work
        - lots of diff work types
    - organization
        - roles, ownership, agents
        - an agent can fill multiple roles
        - intersecting structure of domain ownership and initiative ownership
        - responsibilities relating to a decision
            - Propose: Develop options and a recommendation.
            - Decide: Choose the direction.
            - Execute: Implement the decision.
            - Verify: Establish whether it worked.
            - Maintain: Take responsibility for its ongoing consequences.
- main takeaways
    - we should separate out product from system basically
    - the questions are like
        - what product experience do we want to deliver
        - how will we do that
            - the domains here
- ok so now i basically at a high level want to go through the exercise of
    - what product experience do we want to deliver
    - how will we do that
- the product experience
    - relevant concepts: mission, principle, goal, metric, milestone, user journey, capability, requirement, specification, quality attribute
    - broken down a bit
        
        
        | Layer | Central question | Key concepts |
        | --- | --- | --- |
        | 1. Purpose | Why should this product exist? | Mission, target users, principles |
        | 2. Outcomes | What should users be able to achieve, and what does success look like? | Goals, metrics |
        | 3. Experience | What should using the product actually feel and look like? | User journeys, workflows, UX principles |
        | 4. Capabilities | What must the product enable to deliver those experiences? | Features, capabilities, requirements, specifications |
        | 5. Quality | How good must everything be? | Correctness, performance, reliability, ergonomics, quality attributes |
    - exercises
        
        ### I. Define what we're trying to achieve
        
        1. Who are we building for, and in what contexts?
        
        Who are our target users? When and why would they use CPL? What are their existing workflows, constraints, and alternatives?
        
        2. What value are we trying to create?
        
        What problems are we solving? What outcomes do users care about? Why should someone choose CPL over C++ or Python? What would make CPL genuinely transformative rather than incrementally nicer?
        
        3. What product principles should guide our decisions?
        
        What properties should CPL consistently prioritize? What tradeoffs are we willing or unwilling to make? What are our non-goals?
        
        ### II. Design the actual experience
        
        4. What are the major user journeys?
        
        Enumerate the end-to-end activities users want to accomplish: learning CPL, writing solutions, debugging, stress testing, submitting, etc. What are the important contexts, edge cases, and failure scenarios?
        
        5. What would an exceptional experience look like for each journey?
        
        Walk through the ideal workflow from the user's perspective. What should be effortless? What should feel magical? Where should friction disappear? What happens when things go wrong? What would make users love the experience?
        
        6. What capabilities and behaviors are required to deliver those experiences?
        
        Derive the necessary features, language behaviors, tools, and interactions. What exactly must the product support? What are the requirements and specifications? Stay focused on what it should do, not the implementation architecture.
        
        ### III. Define success and delivery
        
        7. What quality bar must the experience meet?
        
        How correct, fast, reliable, intuitive, discoverable, and polished does each capability need to be? What is merely acceptable versus exceptional? Which properties are non-negotiable?
        
        8. How will we know whether we've succeeded?
        
        What quantitative metrics, benchmark suites, example programs, usability tests, and real-world observations can evaluate our goals? How will we capture feedback and identify what needs improvement?
        
        9. What should we deliver first, and in what sequence?
        
        What milestones represent meaningful, complete product experiences? What is the minimum quality and capability set for each? What can be deferred? What are the highest-value unknowns to resolve first?
        
    - defining what we are trying to achieve
        - questions
            - who are we building for and in what context?
            - what value are we trying to create?
            - what product principles should guide our decisions?
        - my initial answers
            
            ok let's start one by one. first, defining what we're trying to achieve.
            
            we are building for competitive programmers.
            
            for now, we are targeting folks who are already familiar with competitive programming and fluent in a language. our target audience is probably CF 1500+. in the future we might want to target beginners and provide tutorials and stuff, but we are currently focusing on intermediate to advanced to really advanced CPers. they will use it basically as their primary language for upsolving, training, and doing contests. their current main alternative is c++ with homegrown tools
            
            the value prop we offer to users / why they should use CPL. ok just throwing out ideas with no particular ordering for now
            
            - it's a genuinely nice and pleasant experience. like things just work. the tooling is great
            - powerful standard library
            - the language itself is just nice to use and much more expressive than any other language
            - allows you to focus on just the idea and very easily and fluently translate that idea into code
            - the language is designed in a way that makes it hard to misuse and code that compiles is likely correct. you spend time fighting the compiler, not the judge that gives you WA
            - really solid debugging experience that blows everyone else out of the water with stress testing and time stepped debugging
            
            product principles and properties. tbh idk what the border between this and e.g. capabilities, requirements, etc are. but here are some ideas
            
            - we have to be reliable
                - we have to be correct
                - any valid cpl code has to compile and generate code that is submittable (fits under the submission limit) and also the code has to be performant. if users don't trust us they'll never use us in a competition
            - the language should feel coherent and well designed
                - it should just feel super nice to use
                - there's something deeply satisfying about typing a piece of code that just elegantly does what you want it to do. the language should constnatly feel like this
                - everything about it should jsut feel well designed
            - the tooling should just work
            
            what would be additional questions that you might have in this axis? and/or if you feel we're at the 80/20, how would you synthesize these responses?
            
        - mission statement
            
            > Make translating a competitive programming idea into a correct, efficient, accepted solution as fast, effortless, and enjoyable as possible.
            > 
            
            > CPL should be a language people love writing.
            > 

    - designing the actual product experience
        - questions
            - what are the major user journeys and contexts?
            - what would the ideal experience look like?
            - what capabilities, behaviors, and guarantees are required?
        - my answer
            
            ok i'll start with major user journeys and contexts. just throwing ideas out there
            
            - learning
            - setup
            - solving a problem
            - testing the problem
            - debugging
            - submitting
            - maintaining libraries
            - maintaining their contests / spinning up a new contest
            
            what would the ideal experience look like
            
            - learning
                - there's a guide they can read in ~30 mins which basically teaches them all the important syntax of the language
                - on our website, in the browser, they can then just try it out for a bunch of competitive programming style problems (think div 2 a / b, but intentionally selected) and along the way just pick up a bunch of the syntax
                    - probably the experience should be like, a lot of this is about trying to learn how to do things the CPL way
                        - we make it super easy to just search for stuff you're looking for so this teaches you that pattern
                        - you can view hints which links you to std docs of relevant pages that would be helpful
                        - we lint your code so as you write the code you'll get auto suggestions for what you should've typed instead
                - from here, we have more advanced guides that people can read
                    - btw the guides can be pretty interactive. like inline code and maybe have you go implement smth yourself at the end of a guid eif you want
                - and also more advanced coding you can do
                - but basically super low friction to get started and try it out. we want to optimize time to value, and after that you can learn much more advanced features through a combination of reading + doing. we can think about what all the domains here are. it's like, core language syntax, std, writing idomatic code, and stuff
                - and lint always suggests cleaner ways to do what you're trying to do so in the editor and in the browser you're just constantly learning passively
                - and probably we also have an experience where we make it super easy for you to ask AI about how you could code what you currently have more idiomatically
                    - we just have to be a bit careful of people using AI during contest and stuff
            - setup
                - one bash script + click minimal number of buttons to install into your favorite editor and everything just works
                - easy to find the install page and stuff
            - solving a problem
                - i think this kinda gets into the language design / this is smth that i've thought a lot about / one of the clear focus areas that i'm not too worried abou tit / i'm sure it'll get worked on
                - things liek reading input and interactive problems just magically work
            - testing the problem
                - we should have automatically pulled the samples for the problem from competitive companion
                - you just do like, cpl run and it just automatically runs all the samples for you. if you want you can super easily write a checker too. you can also run an indivdual testcase and things like that
                - interactive problems just work. like you can be the interactor yourself, or we make it super super easy for you to write an interactor
            - debugging
                - if you WA, you can one click / one command open up a view where you can time step through your entire program to see exactly what went wrong. you can easily focus and narrow down to a specific testcase in a multitested problem / a specific function call even. and you can like edit that funciton, rerun, directly in the browser (and we have this same experience integrated with your editor as well)
                - also if you want, you can just write a brute force solver super easily (ideally you don't even need to write a gen / the gen ges automatically written as you write the code) and then cpl stress will automatically run on multiple threads and in like <15s find you the smallest test case that fails with very very smart and intelligent narrowing and shrinking algorithms
            - submitting
                - you just cpl build or cpl build --copy (with shorthands for everythign ofc so this becomes cpl b -c) and just directly one click submit
                - i think eventually you jus directly cpl submit where we have integrations with every single online judge on the face of the planet, but this is pretty down the line
            - maintaining libraries
                - the set up of like, you use std + you use your own local library (where you'll want intelligent versioning and so updating your library doesn't break your old code) + you might pull libraries from other people (probably we don't need a full blown package manager / can just use github to start) and it just works
            - maintaining their contests / spinning up a new contest
                - we have a default recommended file system set up but are flexible enough to map to any file structur eyou want. but it's just easy to have cpl automatically maintain your contests and you just not have to think about it
            
            capabilities: hmm i kinda feel like this is somewhat of a function of everything i said above + all the other contest i've sent + my initial notes hwer ei just wrote down all the components
            
            hmm ok do you have any questions so far ig and/or do you think this is enough to write a v0 of like, designing the product experience
            

    - success and delivery
        - questions
            - what quality bar must the experience meet
            - how will we know whether we’ve succeeded
            - what should we deliver first, and in what sequence?
        - ok honestly i feel like this stuff isn’t that important
        - key thing is to decide on our milestone: something i can send to a friend and it’s easy for them to learn and use for new problems and in contest
        - milestone draft
