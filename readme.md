# Lindenmayer Systems Interpreter and Examples

To skip straight to the results: [click here](http://htmlpreview.github.io/?https://github.com/bertdouglas/Lindenmayer/blob/master/rust_svg/lsys_examples.html)

I happened to encounter a reference to the book, "The Algorithmic Beauty of Plants",
and I became interested in Lindenmayer Systems.  I saw how the L-system
could abstract the essence of numerous space filling curves, which had previously
been treated separately.

Years ago, when a student, I had written code to draw the Sierpinski and
Hilbert space filling curves.  This was an exercise in recursion.
I followed the examples from chapter 3 of the book by Niklaus Wirth,
"Algorithms + Data Structures = Programs".  At first I redid these curves
using L-systems.  Then I added a handful of other curves.

L-systems are a merger of concepts from turtle graphics and programming language grammars.
Grammatical productions are applied recursively to generate drawing primitives.

Originally, I implemented an interpreter for L-Systems, written in python and producing
output in postscript.  Later I implemented another version, written in Rust and
producing output in SVG wrapped in Html.

A few references:
- https://en.wikipedia.org/wiki/L-system
- https://en.wikipedia.org/wiki/The_Algorithmic_Beauty_of_Plants
- https://en.wikipedia.org/wiki/Algorithms_%2B_Data_Structures_%3D_Programs

Copyright: 2019-2023 Bert Douglas
SPDX-License-Identifier: MIT
