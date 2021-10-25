import { assert, assertEquals } from "./deps.ts";
import { Vector3 } from "../src/vector3.ts";
import { Matrix3 } from "../src/matrix3.ts";
import { Matrix4 } from "../src/matrix4.ts";

Deno.test("Matrix3.transpose", () => {
  assert(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).transpose().eq(
      Matrix3.from(1, 4, 7, 2, 5, 8, 3, 6, 9),
    ),
  );
});

Deno.test("Matrix3.eq", () => {
  assert(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).eq(
      Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9),
    ),
  );
  assert(!Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).eq(new Matrix3()));
});

Deno.test("Matrix3.isFinite", () => {
  assert(Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).isFinite());
  assert(!Matrix3.from(Infinity, 2, 3, 4, 5, 6, 7, 8, 9).isFinite());
});

Deno.test("Matrix3.row", () => {
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).row(0),
    [1, 4, 7],
  );
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).row(1),
    [2, 5, 8],
  );
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).row(2),
    [3, 6, 9],
  );
});

Deno.test("Matrix3.col", () => {
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).col(0),
    [1, 2, 3],
  );
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).col(1),
    [4, 5, 6],
  );
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).col(2),
    [7, 8, 9],
  );
});

Deno.test("Matrix3.add", () => {
  assert(
    Matrix3.from(1, 1, 1, 1, 1, 1, 1, 1, 1).add(
      Matrix3.from(1, 1, 1, 1, 1, 1, 1, 1, 1),
    ).eq(Matrix3.from(2, 2, 2, 2, 2, 2, 2, 2, 2)),
  );
});

Deno.test("Matrix3.sub", () => {
  assert(
    Matrix3.from(2, 2, 2, 2, 2, 2, 2, 2, 2).sub(
      Matrix3.from(1, 1, 1, 1, 1, 1, 1, 1, 1),
    ).eq(Matrix3.from(1, 1, 1, 1, 1, 1, 1, 1, 1)),
  );
});

Deno.test("Matrix3.mul", () => {
  assert(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).mul(
      Matrix3.from(10, 11, 12, 13, 14, 15, 16, 17, 18),
    ).eq(Matrix3.from(138, 171, 204, 174, 216, 258, 210, 261, 312)),
  );

  assert(
    Matrix3.from(1, 4, 7, 2, 5, 8, 3, 6, 9).mul(
      Matrix3.from(2, 5, 8, 3, 6, 9, 4, 7, 10),
    ).eq(Matrix3.from(36, 81, 126, 42, 96, 150, 48, 111, 174)),
  );
});

Deno.test("Matrix3.toMatrix4", () => {
  assert(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).toMatrix4().eq(
      Matrix4.from(1, 2, 3, 0, 4, 5, 6, 0, 7, 8, 9, 0, 0, 0, 0, 1),
    ),
  );
});

Deno.test("Matrix3.toArray", () => {
  assertEquals(Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).toArray(), [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
  ]);
});

Deno.test("Matrix3.toFloat32Array", () => {
  assertEquals(
    Matrix3.from(1, 2, 3, 4, 5, 6, 7, 8, 9).toFloat32Array(),
    new Float32Array([1, 2, 3, 4, 5, 6, 7, 8, 9]),
  );
});
