#ifndef SDF_H
#define SDF_H

// https://iquilezles.org/articles/distfunctions2d/

float sdf_box(in vec2 p, in vec2 b)
{
    const vec2 d = abs(p) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float sdf_rounded_box( in vec2 p, in vec2 b, in vec4 r )
{
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x  = (p.y > 0.0) ? r.x : r.y;
    const vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

#endif