use rmsim::*;

#[test]
fn square() {
    let source_code = b"registers 3
# r-1 is reserved for GOTO, r-2 will have a copy of r0 in it.
# First, copy r0 to r-2.
1stcopy1: decjz r0 1stcopy2
inc r-2
inc r-9
decjz r-1 1stcopy1
1stcopy2: decjz r-9 3rdcopy1
inc r0
decjz r-1 1stcopy2

# Now, r0 will be copied to the following registers:
# r-3 represents increment,
# r-4 represents addition,
# r-5 represents multiplication.

3rdcopy1: decjz r0 3rdcopy2
inc r-4
inc r-9
decjz r-1 3rdcopy1
3rdcopy2: decjz r-9 4thcopy1
inc r0
decjz r-1 3rdcopy2

4thcopy1: decjz r0 4thcopy2
inc r-5
inc r-9
decjz r-1 4thcopy1
4thcopy2: decjz r-9 drain
inc r0
decjz r-1 4thcopy2

# Set r0 to 0.
drain: decjz r0 increment
decjz r-1 drain

# First, r-3 will be depleted. Then, r-4 gets decremented and r-3 gets reset to the initial value,
# which is saved in r-2.
# When r-4 gets depleted, r-5 gets decremented and r-4 gets reset (and in turn, r-3).
# Once r-4 is depleted, we are done.



# Now that we've finished incrementing, decrement the addition counter:
addition: decjz r-4 HALT
# Restore the increment counter:
additioncopy1: decjz r-2 additioncopy2
inc r-3
inc r-9
decjz r-1 additioncopy1
additioncopy2: decjz r-9 finished_addition_copy
inc r-2
decjz r-1 additioncopy2
# and go back to incrementing:
finished_addition_copy: decjz r-1 increment

increment: decjz r-3 addition
inc r0
decjz r-1 increment";
    let mut prog: Program = read_input(&source_code[..]).unwrap();
    prog.execute();
    assert_eq!(prog.get_state(), "registers 9")
}
