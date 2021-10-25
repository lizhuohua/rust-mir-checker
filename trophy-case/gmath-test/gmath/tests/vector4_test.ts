import { assert, assertEquals } from "./deps.ts";
import { Vector4 } from "../src/vector4.ts";

Deno.test("Vector4.clone", () => {
  const orig = Vector4.one();
  const clone = orig.clone();
  orig.x = 0;
  orig.y = 0;
  orig.z = 0;
  orig.w = 0;

  assert(orig.eq(Vector4.zero()));
  assert(clone.eq(Vector4.one()));
});

Deno.test("Vector4.mag", () => {
  assertEquals(Vector4.zero().mag(), 0);
  assertEquals(Vector4.one().mag(), 2);
  assertEquals(new Vector4(5, 10, 15, 20).mag(), 27.386127875258307);
});

Deno.test("Vector4.mag2", () => {
  assertEquals(Vector4.zero().mag2(), 0);
  assertEquals(Vector4.one().mag2(), 4);
  assertEquals(new Vector4(5, 10, 15, 20).mag2(), 750);
});

Deno.test("Vector4.normal", () => {
  assertEquals(Vector4.zero().normal().x, NaN);
  assertEquals(Vector4.zero().normal().y, NaN);
  assertEquals(Vector4.zero().normal().z, NaN);
  assertEquals(Vector4.zero().normal().w, NaN);
  assert(Vector4.one().normal().eq(new Vector4(0.5)));
});

Deno.test("Vector4.clamp", () => {
  assertEquals(Vector4.zero().clamp(1).x, NaN);
  assertEquals(Vector4.zero().clamp(1).y, NaN);
  assertEquals(Math.round(Vector4.one().clamp(2).mag()), 2);
  assert(new Vector4(1, 0, 0, 0).clamp(2).eq(new Vector4(2, 0, 0, 0)));
});

Deno.test("Vector4.dot", () => {
  assertEquals(Vector4.zero().dot(Vector4.zero()), 0);
  assertEquals(Vector4.one().dot(Vector4.one()), 4);
  assertEquals(new Vector4(1, 2, 3, 4).dot(new Vector4(5, 6, 7, 8)), 70);
});

Deno.test("Vector4.lerp", () => {
  assert(Vector4.zero().lerp(Vector4.one(), 0.5).eq(new Vector4(0.5)));
  assert(Vector4.zero().lerp(Vector4.one(), 0.25).eq(new Vector4(0.25)));
  assert(
    Vector4.zero().lerp(new Vector4(1, 0, 0, 0), 0.5).eq(
      new Vector4(0.5, 0, 0, 0),
    ),
  );
});

Deno.test("Vector4.set", () => {
  const vector = new Vector4();
  vector.set(Vector4.one());

  assert(vector.eq(Vector4.one()));
});

Deno.test("Vector4.neg", () => {
  assert(Vector4.one().neg().eq(new Vector4(-1)));
});

Deno.test("Vector4.add", () => {
  assert(Vector4.one().add(Vector4.one()).eq(new Vector4(2)));
});

Deno.test("Vector4.sub", () => {
  assert(Vector4.one().sub(Vector4.one()).eq(Vector4.zero()));
});

Deno.test("Vector4.mul", () => {
  assert(Vector4.one().mul(Vector4.one()).eq(Vector4.one()));
});

Deno.test("Vector4.div", () => {
  assert(Vector4.one().div(Vector4.one()).eq(Vector4.one()));
});

Deno.test("Vector4.eq", () => {
  assert(Vector4.one().eq(Vector4.one()));
});

Deno.test("Vector4.isFinite", () => {
  assert(Vector4.zero().isFinite());
  assert(Vector4.one().isFinite());
  assert(!Vector4.negativeInfinity().isFinite());
  assert(!Vector4.positiveInfinity().isFinite());
});

Deno.test("Vector4.toArray", () => {
  assertEquals(Vector4.zero().toArray(), [0, 0, 0, 0]);
  assertEquals(Vector4.one().toArray(), [1, 1, 1, 1]);
  assertEquals(new Vector4(1, 2, 3, 4).toArray(), [1, 2, 3, 4]);
});

Deno.test("Vector4.toFloat32Array", () => {
  assertEquals(Vector4.zero().toFloat32Array(), new Float32Array([0, 0, 0, 0]));
  assertEquals(Vector4.one().toFloat32Array(), new Float32Array([1, 1, 1, 1]));
  assertEquals(
    new Vector4(1, 2, 3, 4).toFloat32Array(),
    new Float32Array([1, 2, 3, 4]),
  );
});
