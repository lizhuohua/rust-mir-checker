import { assert, assertEquals } from "./deps.ts";
import { Matrix2 } from "../src/matrix2.ts";
import { Matrix3 } from "../src/matrix3.ts";
import { Matrix4 } from "../src/matrix4.ts";

Deno.test("Matrix2.transpose", () => {
  assert(
    Matrix2.from(1, 2, 3, 4).transpose().eq(Matrix2.from(1, 3, 2, 4)),
  );
});

Deno.test("Matrix2.eq", () => {
  assert(Matrix2.from(1, 2, 3, 4).eq(Matrix2.from(1, 2, 3, 4)));
  assert(!Matrix2.from(1, 2, 3, 4).eq(new Matrix2()));
});

Deno.test("Matrix2.isFinite", () => {
  assert(Matrix2.from(1, 2, 3, 4).isFinite());
  assert(!Matrix2.from(Infinity, 2, 3, 4).isFinite());
});

Deno.test("Matrix2.row", () => {
  assertEquals(Matrix2.from(1, 2, 3, 4).row(0), [1, 3]);
  assertEquals(Matrix2.from(1, 2, 3, 4).row(1), [2, 4]);
});

Deno.test("Matrix2.col", () => {
  assertEquals(Matrix2.from(1, 2, 3, 4).col(0), [1, 2]);
  assertEquals(Matrix2.from(1, 2, 3, 4).col(1), [3, 4]);
});

Deno.test("Matrix2.add", () => {
  assert(
    Matrix2.from(1, 1, 1, 1).add(Matrix2.from(1, 1, 1, 1)).eq(
      Matrix2.from(2, 2, 2, 2),
    ),
  );
});

Deno.test("Matrix2.sub", () => {
  assert(
    Matrix2.from(2, 2, 2, 2).sub(Matrix2.from(1, 1, 1, 1)).eq(
      Matrix2.from(1, 1, 1, 1),
    ),
  );
});

Deno.test("Matrix2.mul", () => {
  assert(
    Matrix2.from(1, 2, 3, 4).mul(Matrix2.from(5, 6, 7, 8)).eq(
      Matrix2.from(23, 34, 31, 46),
    ),
  );

  assert(
    Matrix2.from(1, 3, 2, 4).mul(Matrix2.from(2, 4, 3, 5)).eq(
      Matrix2.from(10, 22, 13, 29),
    ),
  );
});

Deno.test("Matrix2.toMatrix3", () => {
  assert(
    Matrix2.from(1, 2, 3, 4).toMatrix3().eq(
      Matrix3.from(1, 2, 0, 3, 4, 0, 0, 0, 1),
    ),
  );
});

Deno.test("Matrix2.toMatrix4", () => {
  assert(
    Matrix2.from(1, 2, 3, 4).toMatrix4().eq(
      Matrix4.from(1, 2, 0, 0, 3, 4, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1),
    ),
  );
});

Deno.test("Matrix2.toArray", () => {
  assertEquals(Matrix2.from(1, 2, 3, 4).toArray(), [[1, 2], [3, 4]]);
});

Deno.test("Matrix2.toFloat32Array", () => {
  assertEquals(
    Matrix2.from(1, 2, 3, 4).toFloat32Array(),
    new Float32Array([1, 2, 3, 4]),
  );
});
