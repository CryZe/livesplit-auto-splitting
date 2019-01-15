# TODO

- Pointer Path type isn't properly pushed yet
- We may not actually need zext / sext operations for non-casts and instead can just mask.
  - And that is because overflown registers with dirty uper bits don't affect
    the sign at all, so we only need to mask out the relevant bits and we are
    fine. However I'm not actually fully sure if this is true. This optimization
    can certainly be done sometimes but probably not all the time.
  - I'm not sure if it makes sense to change this though, as long term we want
    to use the proper sign extension wasm ops, as those are definitely shorter
    than both our current encoding of sexts, but also shorter than zexts. So we
    may want to just keep the sexts. Binaryen seems to optimize those sexts into
    zexts anyway.
- Show unused variables and functions as warnings
  - Don't codegen or error out on typecheck on those
  - We probably want some kind of reachability pass on that
    - Does the never type play into this? Or is that its own pass?
- CTRL + Z after using the semantic replace is very broken.
  - Might be because we using an exclusive end column. I think Monaco excpects
    one more.
- Lots of panics around wrongly calling a function
  - Assigning a function to a variable panics
  - Calling something that isn't a function as a function
  - Unused functions panic easily due to not having all the types inferred.
    Probably unrelated to functions themselves actually.
- Vars automatically allocate registers at the moment, which we don't want
  - Make sure the fix doesn't break for loops and co. which at the moment
    don't use VarDecl for their temporaries.
  - It probably should be VarDecls instead, so we can declare multiple.
    However if we can declare them in multiple entities, then VarDecl may
    actually be fine.
  - VarDecl is used to push named vars into scope in name resolution. So
    we need a solution that doesn't screw with name resolution. Maybe
    extend it to an enum `VarDecl::Named(...), VarDecl::Anonymous(...)`
  - This has been an issue in reg alloc anyways, where we would just
    create a hash set of all the variables we find, instead of properly
    tracking actual declarations, so that's not just limited to functions.
- Implement tuple literals `(a, b, c)`
  - Types are getting inferred correctly
  - Tuple types can be specified
  -
- Struct literals `{ a, b, c }` and `{ a: expr, b: expr, c: expr }`
- Try removing semicolons from statements by redefining blocks as `{ StmtOrExpr* }`
  - That should at least allow removing the semicolons from `if`
    - Apparently even doing that causes weird edge cases in Rust apparently.
- Boolean comparisons don't short circuit at the moment
  - Is that something we even want?
    - It's definitely something that is the default assumption
    - Codegen is worse, not sure if binaryen can optimize this
      - Maybe optimize manually if not?
- Introduce codespan for nicer error messages, especially in the CLI
- The comparisons in for loops and matches don't seem to extend the variables
  properly
- For type errors we may want to have a notion of "type anchors". These would
  represent the base nodes that refined the types. At the moment we show the
  error where the flow of the two conflicting types meets, which isn't ideal as
  first of all that isn't deterministic (especially once we go parallel) and
  second of all it just isn't meaningful if that place is far away from where
  the actual type definitions are. So preferrably we'd show those "type anchors"
  in the error message.
