# Design

## What is included in the world state?

1. Blocks + their metadata
2. Entities + their metadata
3. World parameters (e.g. time)

## How to represent blocks info?

world_height is 16 chunks

Block: unit
Chunk: 32x32x32 = 32768 blocks
Region: 16x16x(world_height) = 4096 chunks

Blocks are stored in a one-dimensional array in a chunk,
chunks are stored in a one-dimensional array in a region.

Just pointing out the approach to addressing these arrays:

    An element with coordinates of X, Y, Z will be in a place
    I = Z * SLICE_SIZE + Y * COLUMN_SIZE + X

In other words, if we imagine an array of size 2x2x2,
elements will be stored like that:

       X Y Z
    0. 0 0 0
    1. 1 0 0
    2. 0 1 0
    3. 1 1 0
    4. 0 0 1
    5. 1 0 1
    6. 0 1 1
    7. 1 1 1

This is basically binary counting at this point :)
