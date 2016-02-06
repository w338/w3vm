Builtin verbs form the "instruction set" of the virtual machine.
Many verbs have a single, numerical argument, written `verb(arg)` or `verb` to
use the default argument. Some verbs index from the frame pointer, others from
top of stack.

Phrases:
  * Numbers, strings, and symbols are just loaded onto the stack.
  * Pointers are loaded, then executed.
  * `{code}` Represents an unquoted vector. It is executed.
  * `[code]` Represents a quoted vector. It desugars into `load [code]`.
  * `$(var1 var2 ...)` Represents some variable labels, at the current top of stack.
  * `$var` is equivalent to `dupv($var)`
  * `dupv($var)` Duplicates `$var`, as measured from the frame pointer.
  * `swapv($var)` Switch the top of stack with var, as measured from the frame pointer.
  * `popv(index=1)` Pop some number of elements off the stack, up to and including var.
  * `dup(index=0)` Duplicate a stack location to top of stack.
  * `swap(index=1)` Switch the top of stack with another stack entry. Switches with the second element by default.
  * `pop(index=1)` Pop some number of elements off the stack. Pop's one element by default.
  * `+ - * / %` The usual arithmetic operators. They pop the top two elements and push the result onto the stack.
  * `dupi(index=0)` Duplicate a stack location to top of stack, where index is first looked up in the stack.
  * `swapi(index=0)` Switch the top of stack with another stack entry. Looks at the top of stack (by default) to determine where to switch with.
  * `popi(index=0)` Pop some number of elements off the stack. Looks at the top of stack (by default) to determine how much to pop.
  * `go(label)` Sets the program counter to a constant.
    * Note that this can be used to make a loop: `$x start: 1 + go(start:) /* Add 1 to x forever. */`
  * `return(offset)` Return from the current function, adjusting the stack by offset relative to frame pointer.
  * `args(count)` Duplicate the count elements from the top of the parent function's frame.
  * `load(offset=1)` Takes the element at pc + offset and pushes it onto the stack. This is
    usually used to load `{}` vectors by name, instead of executing them.
  * `exec` Executes the vector that's on the top of the stack.
  * `branch(offset)` Pop's the top value on the stack. If it's truthy, continues executing. Otherwise, jumps to pc + offset.
    * This can be used to easily make a `do ... until` loop: `label: code condition branch(label:)`
  * `? (code) (code)` Equivalent to `branch(label:) code label: code`
  * `-> $var` is equivalent to `swapv($var) pop`

Examples:

```

exec_in: {
  /* We're setting the frame pointer here, so we can't actually use these. */
  args(2) /* $(fp v) */
  dup(1) /* dup fp on top */
  fp /* set and pop frame pointer */
  exec_in_magic
  swap
  exec /* execute v, which must return only one value */
  swap
  exec_in_magic 
  is ? () ("exec_in received wrong number of return values" error)
  swap fp /* restore our frame pointer */
  return(-2)
  exec_in_magic: []
}

if: {
  $(pfp ppc _ _)
  $pfp $ppc 1 + vindex /* Get the condition */
  $pfp swap exec_in /* Load the frame pointer, and call exec_in */
  ? ($pfp $ppc 2 + vindex $pfp swap exec_in)
    ($pfp $ppc 3 + vindex load else == ? ($pfp $ppc 4 + vindex $pfp swap exec_in) ())
}

else: {
  "No matching if." error
}

while: {
  $(pfp ppc _ _)
  start:
  $pfp $ppc 1 + vindex /* Get the condition */
  $pfp swap exec_in /* Load the frame pointer, and call exec_in */
  ? ($pfp $ppc 2 + vindex $pfp swap exec_in pop go(start:))
    (return(0))
}

fib: {
  $(x)
  0 1 $(a b)
  loop:
    $x 0 == ? ()
    (
      /* ($a $b +) ($b) -> $a -> $b */
      $a $b + 
        $b swapv($a) pop
      swapv($b) pop
      $x 1 - swapv($x) pop
    )
  pop swap return
}

fib_nice: {
  $(x)
  0 1 $(a b)
  0 $(c)
  while ($x 0 !=) {
    $a $b + -> $c
    $b -> $a
    $c -> $b
    $x 1 - -> $x
  }
  $a -> $x
  return
}
```
