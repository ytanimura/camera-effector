// Post effect assortment
// Copyright © 2022 Dr. Yoshinori Tanimura
// MIT License (photograph texture is not included)

/********************* About bonus effects *********************/
/* I was going to put them on the website, but decided not to  */
/* because one cannot concentrate on the contents with a man's */
/* face in the background. I have compiled a list of various   */
/* techniques, and if you want to activate them, please remove */
/* comment out of the macro BONUS_TRACK.                       */
/***************************************************************/

// Hash without Sine https://www.shadertoy.com/view/4djSRW
vec2 hash23(vec3 p3) {
    p3 = fract(p3 * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}
float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

vec4 getColor(in vec2 U) {
    U.x = (U.x - 0.5 * iResolution.x / iResolution.y)
        / iChannelResolution[0].x
        * iChannelResolution[0].y
        + 0.5;
    return textureLod(iChannel0, fract(U), 0.0);
}

vec2 getNearestSample(in vec2 U) {
    const float TILE_RESOLUTION = 40.0;
    float time = iTime * 0.2;
    float it = floor(time), ft = fract(time);

    vec2 V = U * TILE_RESOLUTION,
    V0 = floor(V) + 0.5;
    float miniDist = 100.0;
    vec2 Vmini = vec2(0);
    for (int i = 0; i < 25; i++) {
        vec2 V0 = V0 + vec2(i % 5, i / 5) - 2.0;
        vec2 disp = mix(
            hash23(vec3(V0, it)),
            hash23(vec3(V0, it + 1.0)),
            ft
        );
        V0 += 4.0 * disp - 2.0;
        float dist = distance(V, V0);
        if (miniDist > dist) {
            miniDist = dist;
            Vmini = V0;
        }
    }
    return Vmini / TILE_RESOLUTION;
}

// voronoi mosaic
vec4 voronoi(in vec2 U) {
    U = getNearestSample(U);
    vec3 e = vec3(1, 1, 0) / iResolution;
    return (
          getColor(U)
        + getColor(U - e.xz - e.zy)
        + getColor(U - e.xz)
        + getColor(U - e.xz + e.zy)
        + getColor(U - e.zy)
        + getColor(U + e.zy)
        + getColor(U + e.xz - e.zy)
        + getColor(U + e.xz)
        + getColor(U + e.xz + e.zy)
    ) / 9.0;
}

float luminance(in vec3 c) {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

vec4 laplacianFilter(in vec2 U) {
    vec2 disp = mix(
        hash23(vec3(U, floor(iTime))),
        hash23(vec3(U, floor(iTime) + 1.0)),
        fract(iTime)
    );
    U += 0.002 * disp - 0.001;
    vec3 e = vec3(1,1,0) / iResolution;
    vec4 col = -8.0 * getColor(U)
            + getColor(U - e.xz - e.zy)
            + getColor(U - e.xz)
            + getColor(U - e.xz + e.zy)
            + getColor(U - e.zy)
            + getColor(U + e.zy)
            + getColor(U + e.xz - e.zy)
            + getColor(U + e.xz)
            + getColor(U + e.xz + e.zy);
    return 1.0 - abs(col);
}

vec4 pointillism(in vec2 U) {
    float r = hash23(vec3(floor((U * iResolution.y + iTime * 5.0) / 2.0), iTime)).x;
    float c = luminance(getColor(U).xyz);
    return vec4(vec3(r > c ? 0.0 : 1.0), 1.0);
}

vec4 color64(in vec2 U) {
    vec3 col = getColor(U).xyz;
    col = floor(col * 4.0 + 0.5 * fract(iTime * 4.0)) / 4.0;
    return vec4(col, 1);
}

vec2 noiseDeform(in vec2 U) {
    vec2 e = vec2(1, 0), iu = floor(U * 100.0), fu = fract(U * 100.0);
    float noise = mix(
        mix(hash12(iu       ), hash12(iu + e.xy), fu.x),
        mix(hash12(iu + e.yx), hash12(iu + e.yy), fu.x),
        fu.y
    );
    return U + (2.0 * noise - 1.0) / 500.0;
}

// https://www.shadertoy.com/view/MsGSRd
vec4 oil(in vec2 U) {
    vec2 d = vec2(1, 0) / iResolution.y;
    vec2 grad = vec2(
        luminance(getColor(noiseDeform(U + d.xy)).xyz)
            - luminance(getColor(noiseDeform(U - d.xy)).xyz),
        luminance(getColor(noiseDeform(U + d.yx)).xyz)
            - luminance(getColor(noiseDeform(U - d.yx)).xyz)
    ) / d.x;

    vec3 n = normalize(vec3(-grad, 150.0));
    vec3 light = normalize(vec3(cos(iTime), sin(iTime), 2));
    float diff = clamp(dot(n, light), 0.5, 1.0);
    float spec = clamp(dot(reflect(light, n), vec3(0, 0, -1)), 0.0, 1.0);
    spec = pow(spec, 36.0) * 2.0;
	return vec4(clamp(getColor(U).xyz * diff + spec, 0.0, 1.0), 1);
}

vec4 contrast(in vec2 U) {
    vec3 col = getColor(U).xyz;
    float th = 0.5 + 0.1 * sin(iTime);
    col = step(col, vec3(th)) * col / 2.0 + step(vec3(th), col) * (1.0 + col) / 2.0;
    return vec4(col, 1);
}

void mainImage(out vec4 O, in vec2 U) {
    U /= iResolution.y;
    int h = int(iTime / 4.0) % 6;
    switch (h) {
        case 0: O = voronoi(U); break;
        case 1: O = pointillism(U); break;
        case 2: O = laplacianFilter(U); break;
        case 3: O = color64(U); break;
        case 4: O = contrast(U); break;
        default: O = oil(U); break;
    }
}
