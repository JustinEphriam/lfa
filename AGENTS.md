Do not leave comments in the code.

Do not use unsafe blocks, if you need unsafe delegate it to an appropriate library.

Favor offsets and arena/pool/bump allocators over pointers and lifetimes. 

Favor cache-friendly patterns.

Do not make core algorithm changes without consulting me and testing the performance of the changes
with hyperfine prior to and after the change.
