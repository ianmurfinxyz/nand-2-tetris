### Notes

bool: true == -1, false == 0

### push segment index

```
@index
D=A       // D=index
@segment
A=M+D     // A=segment[index]
D=M       // D=*(segment[index])
@SP
A=M       // A=*SP
M=D       // **SP=*(segment[index])
@SP
M=M+1     // *SP=*SP+1 (i.e. increment stack pointer)
```

from studying the Pong.asm code there is a better way to do this which reduces the total instructions
by 1,

```
@index
D=A       // D=index
@segment
A=M+D     // A=segment[index]
D=M       // D=*(segment[index])
@SP
AM=M+1
A=A-1
M=D
```

9 instructions instead of 10

Further, this can be optimised for the cases where index == 0 and index == 1

```
// index == 0
@segment
A=M     // A=segment[index]
D=M     // D=*(segment[index])
@SP
AM=M+1
A=A-1
M=D

// index == 1
@segment
A=M+1     // A=segment[index]
D=M       // D=*(segment[index])
@SP
AM=M+1
A=A-1
M=D
```

### pop segment index

```
@SP
M=M-1   // *SP = *SP - 1 (i.e. decrement stack pointer)
A=M     // A = *SP
D=M     // D = **SP (pop value)
@LCL
D=D+M   // D = **SP + *LCL (i.e. pop value + segment base memory address)
@16
D=D+A   // D= **SP + LCL[16] (i.e. pop value + address to pop to)
@SP
A=M     // A = *SP
A=M     // A = **SP (pop value)
A=D-A   // A = (**SP + LCL[16]) - **SP (i.e. address to pop to)
M=D-A   // *(LCL[16]) = (**SP + LCL[16]) - LCL[16] (i.e. address to pop to = pop value)
```

generic version,

```
@SP
M=M-1   // *SP = *SP - 1 (i.e. decrement stack pointer)
A=M     // A = *SP
D=M     // D = **SP (pop value)
@segment
D=D+M   // D = **SP + *segment (i.e. pop value + segment base memory address)
@index
D=D+A   // D= **SP + segment[index] (i.e. pop value + address to pop to)
@SP
A=M     // A = *SP
A=M     // A = **SP (pop value)
A=D-A   // A = (**SP + segment[index]) - **SP (i.e. address to pop to)
M=D-A   // *(segment[index]) = (**SP + segment[index]) - segment[index] (i.e. address to pop to = pop value)
```

### pop constant index

```
@index
D=A
@SP
A=M
M=D
@SP
M=M+1
```

this can actually be done more efficiently with 1 less instruction,

```
@index
D=A
@SP
AM=M+1
A=A-1
M=D
```

or equivalently,

```
@index
D=A
@SP
M=M+1
A=M-1
M=D
```

this can be optimised for the index == 0 and index == 1 cases,

```
// index == 0
@SP
M=M+1
A=M-1
M=0

// index == 1
@SP
M=M+1
A=M-1
M=1
```

### add (x + y)

```
 stack
 -----
 | x |    [initial stack state]
 |---|
 | y |
 |---|
 |   | <- SP
 |---|

@SP
AM=M-1  // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M     // D = **SP (i.e. y)
A=A-1   // A = *SP - 1 (i.e. &x)
M=D+M   // x = x + y
```

### sub (x - y)

```
@SP
AM=M-1  // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M     // D = **SP (i.e. y)
A=A-1   // A = *SP - 1 (i.e. &x)
M=M-D   // x = x - y
```

### neg

```
@SP
A=M-1
M=-M
```

### eq

```
@SP
AM=M-1      // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M         // D = **SP (i.e. y)
A=A-1       // A = *SP - 1 (i.e. &x)
D=M-D       // D = x - y
@TRUE
D;JEQ
@SP
A=M-1
M=0
@END
0;JMP
(TRUE)
@SP
A=M-1
M=-1
(END)
```

from Pong.asm the following EQ routine can be found:

```
 stack
 -----
 | x |    [initial stack state]
 |---|
 | y |
 |---|
 |   | <- SP
 |---|

// backup return address
@R15
M=D

// D = y
@SP
AM=M-1
D=M

// D = x - y
A=A-1
D=M-D

// x = 0
M=0

@END_EQ
D;JNE

// false, so override x to x = -1
@SP
A=M-1
M=-1

// return to caller
(END_EQ)
@R15
A=M
0;JMP
```

an example of a call to the EQ routine,

```
// push argument 1
@ARG
A=M+1
D=M
@SP
AM=M+1
A=A-1
M=D

// push constant 0
@SP
M=M+1
A=M-1
M=0

// D = return address
@RET_ADDRESS_EQ0
D=A

// jump to common eq routine
@6
0;JMP
(RET_ADDRESS_EQ0)
```

hence note that upon entry of @6 D == return address.

The Pong.asm example is better since it reduces code bloat by using a common call routine. My initial
design would of required inlining in every call site.

Note that each call to the common routine needs a unique return label, which is generated simply by
incrememting a postfix index.

### gt (x > y)

```
@SP
AM=M-1      // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M         // D = **SP (i.e. y)
A=A-1       // A = *SP - 1 (i.e. &x)
D=M-D       // D = x - y
@TRUE
D;JGT
@SP
A=M-1
M=0
@GT_END
0;JMP
(TRUE)
@SP
A=M-1
M=-1
(END)
```

Pong.asm gt, (routine @22)

```
@R15
M=D
@SP
AM=M-1
D=M
A=A-1
D=M-D
M=0
@END_GT
D;JLE
@SP
A=M-1
M=-1
@R15
A=M
0;JMP
```

### lt (x < y)

```
@SP
AM=M-1      // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M         // D = **SP (i.e. y)
A=A-1       // A = *SP - 1 (i.e. &x)
D=M-D       // D = x - y
@TRUE
D;JLT
@SP
A=M-1
M=0
@GT_END
0;JMP
(TRUE)
@SP
A=M-1
M=-1
(END)
```

Pong.asm lt, (routine @38)

```
@R15
M=D
@SP
AM=M-1
D=M
A=A-1
D=M-D
M=0
@END_LT
D;JGE
@SP
A=M-1
M=-1
@R15
A=M
0;JMP
```

### and

```
@SP
AM=M-1  // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M     // D = **SP (i.e. y)
A=A-1   // A = *SP - 1 (i.e. &x)
M=D&M   // x = x + y
```

### or

```
@SP
AM=M-1  // A= *SP - 1; *SP = *SP - 1; (i.e. decrement stack pointer)
D=M     // D = **SP (i.e. y)
A=A-1   // A = *SP - 1 (i.e. &x)
M=D|M   // x = x + y
```

### not

```
@SP
A=M-1
M=!M
```

### call name nArgs

Studying Pong.asm

```
// set R13 = nArgs
@N
D=A
@R13
M=D

// R14 = ROM address of function to call
@ball.new
D=A
@R14 
M=D

// D = ROM address to return to after call
@RET_ADDRESS_CALL56
D=A

// jump to common call routine
@95
0;JMP
```

The start of the Pong.asm file contains common routines that are used throughout the
Pong.asm program. @95 is the call routine that branches to another function. These common
routines clearly are intended to avoid having to duplicate these common code snippets
everywhere (avoid inlining), which reduces code bloat.

Before calling @95, the following is set:
R13 = n args passed to callee
R14 = address of routine to call
D   = return address

In the above the label RET_ADDRESS_CALL56 should be generated for each function call, the book
actually recommends a different naming convention, which is ```fileName.functionName$ret.i```, where
**functionName** is that of the caller not the callee, and **fileName** is the file within which **functionName**
is defined (without file extension).

The following is @95, (common call routine)

```
// push return address onto the stack
@SP
A=M
M=D

// push LCL onto the stack
@LCL
D=M
@SP
AM=M+1
M=D

// push ARG onto the stack
@ARG
D=M
@SP
AM=M+1
M=D

// push THIS onto the stack
@THIS
D=M
@SP
AM=M+1
M=D

// push THAT onto the stack
@THAT
D=M
@SP
AM=M+1
M=D

// calculate address of callee's ARG
@4
D=A
@R13
D=D+M
@SP
D=M-D

// set callee's ARG
@ARG
M=D

// increment stack point
@SP
MD=M+1

// set callee's LCL
@LCL
M=D

// jump to callee
// note: SP == LCL at this point
@R14
A=M
0;JMP
```

stack model:

```
 |------|
 | arg0 | <- ARG [of callee]
 |------|
 | reta |
 |------|
 | LCL  | [of caller]
 |------|
 | ARG  | [of caller]
 |------|
 | THIS | [of caller]
 |------|
 | THAT | [of caller]
 |------|
 | lcl0 | <-- LCL [of callee]
 |------|
 | lcl1 |
 |------|
 |      | <-- SP
 |------|
```

### function name nLcls

Steps:
- label function
- push n local vars on stack, each set to 0

An example in Pong.asm

```
(ball.bounce)
@5
D=A
(LOOP_ball.bounce)
D=D-1
@SP
AM=M+1
A=A-1
M=0
@LOOP_ball.bounce
D;JGT
```

Clearly they are using a loop here; there is likely a range of arg counts for which inlining each push 
is more efficient.

```
// 0 args
(functionName)

// 1 arg
(functionName)
@SP
AM=M+1
A=A-1
M=0

// 2 arg
(functionName)
@SP
AM=M+1
A=A-1
M=0
@SP
AM=M+1
A=A-1
M=0
```

a 3rd arg will clearly take more instructions, so it is worth inlining the pushes for nArgs <= 2.

functionName label should have the format ```fileName.functionName```.

### return

return routine by me,

```
// frame = LCL (frame is a temp)
@LCL
D=M
@R13
M=D

// *ARG = pop(), in other words SP -= 1, and ARG = return value (top of stack)
// this works because ARG[0] of the callee, will be the top of the stack for the caller when returned.
@SP
AM=M-1
D=M
@ARG
M=D

// SP = ARG + 1
@ARG
D=M
@SP
M=D+1

// THAT=*(frame-1)
@R13
AM=M-1
D=M
@THAT
M=D

// THIS=*(frame-2)
@R13
AM=M-1
D=M
@THIS
M=D

// ARG=*(frame-3)
@R13
AM=M-1
D=M
@ARG
M=D

// LCL=*(frame-4)
@R13
AM=M-1
D=M
@LCL
M=D

// goto return address
@R13
A=M
0;JMP
```

return routine found in Pong.asm

```
// R13 = return address
@5
D=A
@LCL
A=M-D
D=M
@R13
M=D

// *ARG=pop()
@SP
AM=M-1
D=M
@ARG
A=M
M=D

// SP=ARG+1
D=A
@SP
M=D+1

// THAT=*(frame-1)
@LCL
D=M
@R14
AM=D-1
D=M
@THAT
M=D

@R14
AM=M-1
D=M
@THIS
M=D

@R14
AM=M-1
D=M
@ARG
M=D

@R14
AM=M-1
D=M
@LCL
M=D

// goto return address
@R13
A=M
0;JMP
```

So theres is different...pretty sure it is because of what happens if nArgs == 0, it means that
ARG points to RAM location which stores the return address, hence if you don't backup the return
address, when you do ```*ARG=pop()```, you will overwrite the return address. Hence the Pong.asm
solution is correct.

calling the common return routine,



### label label

simple inserts a label in the assembly, at the point the vm command is found.

- only labeled locations can be jumped to
- only labels within the same function as the goto can be jumped to
- label format is: ```functionName$label```
- labels consist of chars: [a-zA-Z_.:]; $ separates from function name, so cannot be in label part

```
(file.functionName$label)
```

### goto label

unconditional jump to a label in the same function; does not require a return be setup.

```
@file.functionName$label
0;JMP
```

### if-goto label

conditional jump, based on if the value at the top of the stack is true (-1) or false (0).

op:
- pop value off stack
- if value is true, jump to label, else continue

```
@SP
AM=M-1
D=M
@file.functionName$label
D;JNE
```

### Bootstrap

Pong.asm also has the bootstrap code at ROM 0,

```
@256
D=A
@SP
M=D
@133
0;JMP
```

which jumps to, (line 133)

```
// R13 = 0
@0
D=A
@R13
M=D

// R14 = sys.init routine ROM address
@sys.init
D=A
@R14
M=D

// call sys.init
@RET_ADDRESS_CALL0
D=A
@95
0;JMP
```

sys.init is defined at the very bottom of the Pong.asm file with all the other OS routines, like
sys.wait, sys.error etc, along with utilities like string.newline. But that is foreshadowing for
the OS chaptor.