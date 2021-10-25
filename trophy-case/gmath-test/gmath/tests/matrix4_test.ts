import { assert, assertEquals } from "./deps.ts";
import { Matrix4 } from "../src/matrix4.ts";

Deno.test("Matrix4.transpose", () => {
  assert(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
      .transpose().eq(
        Matrix4.from(1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15, 4, 8, 12, 16),
      ),
  );
});

Deno.test("Matrix4.eq", () => {
  assert(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).eq(
      Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16),
    ),
  );
  assert(
    !Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).eq(
      new Matrix4(),
    ),
  );
});

Deno.test("Matrix4.isFinite", () => {
  assert(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
      .isFinite(),
  );
  assert(
    // deno-fmt-ignore
    !Matrix4.from(
      Infinity, 2, 3, 4,
      5, 6, 7, 8,
      9, 10, 11, 12,
      13, 14, 15, 16,
    ).isFinite(),
  );
});

Deno.test("Matrix4.row", () => {
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).row(
      0,
    ),
    [1, 5, 9, 13],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).row(
      1,
    ),
    [2, 6, 10, 14],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).row(
      2,
    ),
    [3, 7, 11, 15],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).row(
      3,
    ),
    [4, 8, 12, 16],
  );
});

Deno.test("Matrix4.col", () => {
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).col(0),
    [1, 2, 3, 4],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).col(
      1,
    ),
    [5, 6, 7, 8],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).col(
      2,
    ),
    [9, 10, 11, 12],
  );
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16).col(
      3,
    ),
    [13, 14, 15, 16],
  );
});

Deno.test("Matrix4.add", () => {
  assert(
    Matrix4.from(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1).add(
      Matrix4.from(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1),
    ).eq(Matrix4.from(2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2)),
  );
});

Deno.test("Matrix4.sub", () => {
  assert(
    Matrix4.from(2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2).sub(
      Matrix4.from(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1),
    ).eq(Matrix4.from(1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1)),
  );
});

Deno.test("Matrix4.mul", () => {
  // deno-fmt-ignore
  const a = Matrix4.from(
    1, 2, 3, 4,
    5, 6, 7, 8,
    9, 10, 11, 12,
    13, 14, 15, 16
  );
  // deno-fmt-ignore
  const b = Matrix4.from(
    17, 18, 19, 20,
    21, 22, 23, 24,
    25, 26, 27, 28,
    29, 30, 31, 32
  );
  // deno-fmt-ignore
  const c = Matrix4.from(
    538, 612, 686, 760,
    650, 740, 830, 920,
    762, 868, 974, 1080,
    874, 996, 1118, 1240
  );
  // deno-fmt-ignore
  const d = Matrix4.from(
    1, 5, 9, 13,
    2, 6, 10, 14,
    3, 7, 11, 15,
    4, 8, 12, 16
  );
  // deno-fmt-ignore
  const e = Matrix4.from(
    2, 6, 10, 14,
    3, 7, 11, 15,
    4, 8, 12, 16,
    5, 9, 13, 17
  );
  // deno-fmt-ignore
  const f = Matrix4.from(
    100, 228, 356, 484,
    110, 254, 398, 542,
    120, 280, 440, 600,
    130, 306, 482, 658,
  );

  assert(a.mul(b).eq(c));
  assert(d.mul(e).eq(f));
});

Deno.test("Matrix4.toArray", () => {
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
      .toArray(),
    [
      [1, 2, 3, 4],
      [5, 6, 7, 8],
      [9, 10, 11, 12],
      [13, 14, 15, 16],
    ],
  );
});

Deno.test("Matrix4.toFloat32Array", () => {
  assertEquals(
    Matrix4.from(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
      .toFloat32Array(),
    new Float32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
  );
});
