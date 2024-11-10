#ifndef RADIX_SORT_H
#define RADIX_SORT_H

const uint RADIX_BITS = 8;
const uint RADIX_DIGITS = 1 << RADIX_BITS;
const uint RADIX_MASK = RADIX_DIGITS - 1;

const uint RADIX_WGP_SIZE = 256;
const uint RADIX_ITEMS_PER_INVOCATION = 16;
const uint RADIX_ITEMS_PER_WGP = RADIX_WGP_SIZE * RADIX_ITEMS_PER_INVOCATION;

layout(buffer_reference, std430, buffer_reference_align = 4) coherent buffer FinishedRef {
    coherent uint value;
};

layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer CountRef {
    uint value;
};

#endif
