// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// Put your code here.
(CHK_KBD)
@SCREEN
D=A
@CURRENT_SCREEN
M=D
@KBD
D=M
@CLEAR_SCREEN
D;JEQ
@FILL_SCREEN
0;JMP

(CLEAR_SCREEN)
@R0
M=0
@UPDATE_SCREEN
0;JMP

(FILL_SCREEN)
@R0
M=-1

(UPDATE_SCREEN)
@R0
D=M
@CURRENT_SCREEN
A=M
M=D
@CURRENT_SCREEN
MD=M+1
@SCREEN
D=D-A
@8192
D=A-D
@UPDATE_SCREEN
D;JGT
@CHK_KBD
0;JMP
