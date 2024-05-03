// Translation of Sean Barrett's stb_perlin.h
//
// Public domain
//
// LICENSE
//
//   See end of file.
//
// Contributors:
//    Jack Mott - additional noise functions
//    Jordan Peck - seeded noise
//

use crate::lerp;

// Not same permutation table as Perlin's reference to avoid copyright issues.
// Perlin's table can be found at http://mrl.nyu.edu/~perlin/noise/
const PERMUTE: [u8; 512] = [
    23, 125, 161, 52, 103, 117, 70, 37, 247, 101, 203, 169, 124, 126, 44, 123, 152, 238, 145, 45,
    171, 114, 253, 10, 192, 136, 4, 157, 249, 30, 35, 72, 175, 63, 77, 90, 181, 16, 96, 111, 133,
    104, 75, 162, 93, 56, 66, 240, 8, 50, 84, 229, 49, 210, 173, 239, 141, 1, 87, 18, 2, 198, 143,
    57, 225, 160, 58, 217, 168, 206, 245, 204, 199, 6, 73, 60, 20, 230, 211, 233, 94, 200, 88, 9,
    74, 155, 33, 15, 219, 130, 226, 202, 83, 236, 42, 172, 165, 218, 55, 222, 46, 107, 98, 154,
    109, 67, 196, 178, 127, 158, 13, 243, 65, 79, 166, 248, 25, 224, 115, 80, 68, 51, 184, 128,
    232, 208, 151, 122, 26, 212, 105, 43, 179, 213, 235, 148, 146, 89, 14, 195, 28, 78, 112, 76,
    250, 47, 24, 251, 140, 108, 186, 190, 228, 170, 183, 139, 39, 188, 244, 246, 132, 48, 119, 144,
    180, 138, 134, 193, 82, 182, 120, 121, 86, 220, 209, 3, 91, 241, 149, 85, 205, 150, 113, 216,
    31, 100, 41, 164, 177, 214, 153, 231, 38, 71, 185, 174, 97, 201, 29, 95, 7, 92, 54, 254, 191,
    118, 34, 221, 131, 11, 163, 99, 234, 81, 227, 147, 156, 176, 17, 142, 69, 12, 110, 62, 27, 255,
    0, 194, 59, 116, 242, 252, 19, 21, 187, 53, 207, 129, 64, 135, 61, 40, 167, 237, 102, 223, 106,
    159, 197, 189, 215, 137, 36, 32, 22, 5,
    // and a second copy so we don't need an extra mask or static initializer
    23, 125, 161, 52, 103, 117, 70, 37, 247, 101, 203, 169, 124, 126, 44, 123, 152, 238, 145, 45,
    171, 114, 253, 10, 192, 136, 4, 157, 249, 30, 35, 72, 175, 63, 77, 90, 181, 16, 96, 111, 133,
    104, 75, 162, 93, 56, 66, 240, 8, 50, 84, 229, 49, 210, 173, 239, 141, 1, 87, 18, 2, 198, 143,
    57, 225, 160, 58, 217, 168, 206, 245, 204, 199, 6, 73, 60, 20, 230, 211, 233, 94, 200, 88, 9,
    74, 155, 33, 15, 219, 130, 226, 202, 83, 236, 42, 172, 165, 218, 55, 222, 46, 107, 98, 154,
    109, 67, 196, 178, 127, 158, 13, 243, 65, 79, 166, 248, 25, 224, 115, 80, 68, 51, 184, 128,
    232, 208, 151, 122, 26, 212, 105, 43, 179, 213, 235, 148, 146, 89, 14, 195, 28, 78, 112, 76,
    250, 47, 24, 251, 140, 108, 186, 190, 228, 170, 183, 139, 39, 188, 244, 246, 132, 48, 119, 144,
    180, 138, 134, 193, 82, 182, 120, 121, 86, 220, 209, 3, 91, 241, 149, 85, 205, 150, 113, 216,
    31, 100, 41, 164, 177, 214, 153, 231, 38, 71, 185, 174, 97, 201, 29, 95, 7, 92, 54, 254, 191,
    118, 34, 221, 131, 11, 163, 99, 234, 81, 227, 147, 156, 176, 17, 142, 69, 12, 110, 62, 27, 255,
    0, 194, 59, 116, 242, 252, 19, 21, 187, 53, 207, 129, 64, 135, 61, 40, 167, 237, 102, 223, 106,
    159, 197, 189, 215, 137, 36, 32, 22, 5,
];

// Perlin's gradient has 12 cases so some get used 1/16th of the time
// and some 2/16ths. We reduce bias by changing those fractions
// to 5/64ths and 6/64ths

// This array is designed to match the previous implementation
// of gradient hash: indices[stb__perlin_randtab[i]&63]
const INDICES: [u8; 512] = [
    7, 9, 5, 0, 11, 1, 6, 9, 3, 9, 11, 1, 8, 10, 4, 7, 8, 6, 1, 5, 3, 10, 9, 10, 0, 8, 4, 1, 5, 2,
    7, 8, 7, 11, 9, 10, 1, 0, 4, 7, 5, 0, 11, 6, 1, 4, 2, 8, 8, 10, 4, 9, 9, 2, 5, 7, 9, 1, 7, 2,
    2, 6, 11, 5, 5, 4, 6, 9, 0, 1, 1, 0, 7, 6, 9, 8, 4, 10, 3, 1, 2, 8, 8, 9, 10, 11, 5, 11, 11, 2,
    6, 10, 3, 4, 2, 4, 9, 10, 3, 2, 6, 3, 6, 10, 5, 3, 4, 10, 11, 2, 9, 11, 1, 11, 10, 4, 9, 4, 11,
    0, 4, 11, 4, 0, 0, 0, 7, 6, 10, 4, 1, 3, 11, 5, 3, 4, 2, 9, 1, 3, 0, 1, 8, 0, 6, 7, 8, 7, 0, 4,
    6, 10, 8, 2, 3, 11, 11, 8, 0, 2, 4, 8, 3, 0, 0, 10, 6, 1, 2, 2, 4, 5, 6, 0, 1, 3, 11, 9, 5, 5,
    9, 6, 9, 8, 3, 8, 1, 8, 9, 6, 9, 11, 10, 7, 5, 6, 5, 9, 1, 3, 7, 0, 2, 10, 11, 2, 6, 1, 3, 11,
    7, 7, 2, 1, 7, 3, 0, 8, 1, 1, 5, 0, 6, 10, 11, 11, 0, 2, 7, 0, 10, 8, 3, 5, 7, 1, 11, 1, 0, 7,
    9, 0, 11, 5, 10, 3, 2, 3, 5, 9, 7, 9, 8, 4, 6, 5,
    // and a second copy so we don't need an extra mask or static initializer
    7, 9, 5, 0, 11, 1, 6, 9, 3, 9, 11, 1, 8, 10, 4, 7, 8, 6, 1, 5, 3, 10, 9, 10, 0, 8, 4, 1, 5, 2,
    7, 8, 7, 11, 9, 10, 1, 0, 4, 7, 5, 0, 11, 6, 1, 4, 2, 8, 8, 10, 4, 9, 9, 2, 5, 7, 9, 1, 7, 2,
    2, 6, 11, 5, 5, 4, 6, 9, 0, 1, 1, 0, 7, 6, 9, 8, 4, 10, 3, 1, 2, 8, 8, 9, 10, 11, 5, 11, 11, 2,
    6, 10, 3, 4, 2, 4, 9, 10, 3, 2, 6, 3, 6, 10, 5, 3, 4, 10, 11, 2, 9, 11, 1, 11, 10, 4, 9, 4, 11,
    0, 4, 11, 4, 0, 0, 0, 7, 6, 10, 4, 1, 3, 11, 5, 3, 4, 2, 9, 1, 3, 0, 1, 8, 0, 6, 7, 8, 7, 0, 4,
    6, 10, 8, 2, 3, 11, 11, 8, 0, 2, 4, 8, 3, 0, 0, 10, 6, 1, 2, 2, 4, 5, 6, 0, 1, 3, 11, 9, 5, 5,
    9, 6, 9, 8, 3, 8, 1, 8, 9, 6, 9, 11, 10, 7, 5, 6, 5, 9, 1, 3, 7, 0, 2, 10, 11, 2, 6, 1, 3, 11,
    7, 7, 2, 1, 7, 3, 0, 8, 1, 1, 5, 0, 6, 10, 11, 11, 0, 2, 7, 0, 10, 8, 3, 5, 7, 1, 11, 1, 0, 7,
    9, 0, 11, 5, 10, 3, 2, 3, 5, 9, 7, 9, 8, 4, 6, 5,
];

fn grad(index: usize, x: f32, y: f32, z: f32) -> f32 {
    const BASIS: [[f32; 3]; 12] = [
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 1.0],
        [-1.0, 0.0, 1.0],
        [1.0, 0.0, -1.0],
        [-1.0, 0.0, -1.0],
        [0.0, 1.0, 1.0],
        [0.0, -1.0, 1.0],
        [0.0, 1.0, -1.0],
        [0.0, -1.0, -1.0],
    ];

    // Unfortunately the compiler does not track the range of values in INDICES,
    // and so cannot see a bounds check is superflous here. Do it ourselves.
    //
    // SAFETY: const loop asserts all values in INDICES are in bounds for BASIS.
    let &[a, b, c] = unsafe {
        const _: () = {
            let mut i = 0;
            while i < INDICES.len() {
                assert!((INDICES[i] as usize) < BASIS.len());
                i += 1;
            }
        };
        BASIS.get_unchecked(INDICES[index] as usize)
    };
    a * x + b * y + c * z
}

/// Computes a random value at the coordinate (x,y,z)
///
/// Adjacent random values are continuous but the noise fluctuates its
/// randomness with period 1, i.e. takes on wholly unrelated values at integer
/// points. Specifically, this implements Ken Perlin's revised noise function
/// from 2002.
///
/// The `wrap` parameters can be used to create wraparound noise that wraps at
/// powers of two. The numbers MUST be powers of two. Specify `0` to mean
/// "don't care". The noise always wraps every 256 due to details of the
/// implementation, even if you ask for larger or no wrapping.
///
/// `seed`` selects from multiple different variations of the noise function.
/// The current implementation only uses the bottom 8 bits of `seed`.
pub fn perlin_noise3_wrap_seed(
    x: f32,
    y: f32,
    z: f32,
    x_wrap: usize,
    y_wrap: usize,
    z_wrap: usize,
    seed: u8,
) -> f32 {
    let seed = seed as usize;
    let x_mask = x_wrap.wrapping_sub(1) & 255;
    let y_mask = y_wrap.wrapping_sub(1) & 255;
    let z_mask = z_wrap.wrapping_sub(1) & 255;
    let px = x.floor();
    let py = y.floor();
    let pz = z.floor();
    let x0 = px as i32 as usize & x_mask;
    let x1 = (px + 1.0) as i32 as usize & x_mask;
    let y0 = py as i32 as usize & y_mask;
    let y1 = (py + 1.0) as i32 as usize & y_mask;
    let z0 = pz as i32 as usize & z_mask;
    let z1 = (pz + 1.0) as i32 as usize & z_mask;

    #[inline(always)]
    fn ease(a: f32) -> f32 {
        a.mul_add(6.0, -15.0).mul_add(a, 10.0) * a * a * a
    }

    let x = x - px;
    let u = ease(x);
    let y = y - py;
    let v = ease(y);
    let z = z - pz;
    let w = ease(z);

    let r0 = PERMUTE[x0 + seed] as usize;
    let r1 = PERMUTE[x1 + seed] as usize;

    let r00 = PERMUTE[r0 + y0] as usize;
    let r01 = PERMUTE[r0 + y1] as usize;
    let r10 = PERMUTE[r1 + y0] as usize;
    let r11 = PERMUTE[r1 + y1] as usize;

    let n000 = grad(r00 + z0, x, y, z);
    let n001 = grad(r00 + z1, x, y, z - 1.0);
    let n010 = grad(r01 + z0, x, y - 1.0, z);
    let n011 = grad(r01 + z1, x, y - 1.0, z - 1.0);
    let n100 = grad(r10 + z0, x - 1.0, y, z);
    let n101 = grad(r10 + z1, x - 1.0, y, z - 1.0);
    let n110 = grad(r11 + z0, x - 1.0, y - 1.0, z);
    let n111 = grad(r11 + z1, x - 1.0, y - 1.0, z - 1.0);

    let n00 = lerp(w, n000, n001);
    let n01 = lerp(w, n010, n011);
    let n10 = lerp(w, n100, n101);
    let n11 = lerp(w, n110, n111);

    let n0 = lerp(v, n00, n01);
    let n1 = lerp(v, n10, n11);

    lerp(u, n0, n1)
}

/// Computes a random value at the coordinate (x,y,z)
///
/// Adjacent random values are continuous but the noise fluctuates its
/// randomness with period 1, i.e. takes on wholly unrelated values at integer
/// points. Specifically, this implements Ken Perlin's revised noise function
/// from 2002.
///
/// The `wrap` parameters can be used to create wraparound noise that wraps at
/// powers of two. The numbers MUST be powers of two. Specify `0` to mean
/// "don't care". The noise always wraps every 256 due to details of the
/// implementation, even if you ask for larger or no wrapping.
pub fn perlin_noise3_wrap(
    x: f32,
    y: f32,
    z: f32,
    x_wrap: usize,
    y_wrap: usize,
    z_wrap: usize,
) -> f32 {
    perlin_noise3_wrap_seed(x, y, z, x_wrap, y_wrap, z_wrap, 0)
}

/// Computes a random value at the coordinate (x,y,z)
///
/// Adjacent random values are continuous but the noise fluctuates its
/// randomness with period 1, i.e. takes on wholly unrelated values at integer
/// points. Specifically, this implements Ken Perlin's revised noise function
/// from 2002.
pub fn perlin_noise3(x: f32, y: f32, z: f32) -> f32 {
    perlin_noise3_wrap_seed(x, y, z, 0, 0, 0, 0)
}

/*
------------------------------------------------------------------------------
This software is available under 2 licenses -- choose whichever you prefer.
------------------------------------------------------------------------------
ALTERNATIVE A - MIT License
Copyright (c) 2017 Sean Barrett
Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
------------------------------------------------------------------------------
ALTERNATIVE B - Public Domain (www.unlicense.org)
This is free and unencumbered software released into the public domain.
Anyone is free to copy, modify, publish, use, compile, sell, or distribute this
software, either in source code form or as a compiled binary, for any purpose,
commercial or non-commercial, and by any means.
In jurisdictions that recognize copyright laws, the author or authors of this
software dedicate any and all copyright interest in the software to the public
domain. We make this dedication for the benefit of the public at large and to
the detriment of our heirs and successors. We intend this dedication to be an
overt act of relinquishment in perpetuity of all present and future rights to
this software under copyright law.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
------------------------------------------------------------------------------
*/
