//// This file is part of www.nand2tetris.org
//// and the book "The Elements of Computing Systems"
//// by Nisan and Schocken, MIT Press.
//// File name: projects/07/MemoryAccess/StaticTest/StaticTest.vm
//
//// Executes pop and push commands using the static segment.
//push constant 111
@111
D=A
@SP
A=M
M=D
@SP
M=M+1
//push constant 333
@333
D=A
@SP
A=M
M=D
@SP
M=M+1
//push constant 888
@888
D=A
@SP
A=M
M=D
@SP
M=M+1
//pop static 8
@SP
M=M-1
A=M
D=M
@STATICTEST.8
M=D
//pop static 3
@SP
M=M-1
A=M
D=M
@STATICTEST.3
M=D
//pop static 1
@SP
M=M-1
A=M
D=M
@STATICTEST.1
M=D
//push static 3
@STATICTEST.3
D=M
@SP
A=M
M=D
@SP
M=M+1
//push static 1
@STATICTEST.1
D=M
@SP
A=M
M=D
@SP
M=M+1
//sub
@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=M-D
@SP
M=M+1
//push static 8
@STATICTEST.8
D=M
@SP
A=M
M=D
@SP
M=M+1
//add
@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=M+D
@SP
M=M+1
//
(STATICTEST_1)
@STATICTEST_1
0;JMP
