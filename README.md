## memconstruct
A crate to safely initialize memory anywhere. Memory can be initialized directly on the heap inside
a box or a `MaybeUninit` could be safely initialized.
The types initialization is checked via typestate.

This is currently an absolute mvp and a work in progress.

