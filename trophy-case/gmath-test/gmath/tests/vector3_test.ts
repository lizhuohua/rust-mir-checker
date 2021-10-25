import { assert, assertEquals } from "./deps.ts";
import { Vector3 } from "../src/vector3.ts";

Deno.test("Vector3.clone", () => {
  const orig = Vector3.one();
  const clone = orig.clone();
  orig.x = 0;
  orig.y = 0;
  orig.z = 0;

  assert(orig.eq(Vector3.zero()));
  assert(clone.eq(Vector3.one()));
});

Deno.test("Vector3.mag", () => {
  assertEquals(Vector3.zero().mag(), 0);
  assertEquals(Vector3.right().mag(), 1);
  assertEquals(Vector3.one().mag(), 1.7320508075688772);
  assertEquals(new Vector3(5, 10, 15).mag(), 18.708286933869704);
});

Deno.test("Vector3.mag2", () => {
  assertEquals(Vector3.zero().mag2(), 0);
  assertEquals(Vector3.right().mag2(), 1);
  assertEquals(Vector3.one().mag2(), 3);
  assertEquals(new Vector3(5, 10, 15).mag2(), 350);
});

Deno.test("Vector3.normal", () => {
  assertEquals(Vector3.zero().normal().x, NaN);
  assertEquals(Vector3.zero().normal().y, NaN);
  assertEquals(Vector3.zero().normal().z, NaN);
  assert(Vector3.right().normal().eq(Vector3.right()));
  assert(Vector3.one().normal().eq(new Vector3(0.5773502691896258)));
});

Deno.test("Vector3.clamp", () => {
  assertEquals(Vector3.zero().clamp(1).x, NaN);
  assertEquals(Vector3.zero().clamp(1).y, NaN);
  assertEquals(Math.round(Vector3.one().clamp(2).mag()), 2);
  assert(Vector3.right().clamp(2).eq(new Vector3(2, 0, 0)));
});

Deno.test("Vector3.dot", () => {
  assertEquals(Vector3.zero().dot(Vector3.zero()), 0);
  assertEquals(Vector3.one().dot(Vector3.one()), 3);
  assertEquals(new Vector3(1, 2, 3).dot(new Vector3(4, 5, 6)), 32);
});

Deno.test("Vector3.cross", () => {
  assert(
    new Vector3(1, 2, 3).cross(new Vector3(4, 5, 6)).eq(new Vector3(-3, 6, -3)),
  );
});

Deno.test("Vector3.lerp", () => {
  assert(Vector3.zero().lerp(Vector3.one(), 0.5).eq(new Vector3(0.5)));
  assert(Vector3.zero().lerp(Vector3.one(), 0.25).eq(new Vector3(0.25)));
  assert(Vector3.zero().lerp(Vector3.right(), 0.5).eq(new Vector3(0.5, 0, 0)));
});

Deno.test("Vector3.set", () => {
  const vector = new Vector3();
  vector.set(Vector3.one());

  assert(vector.eq(Vector3.one()));
});

Deno.test("Vector3.neg", () => {
  assert(Vector3.up().neg().eq(Vector3.down()));
  assert(Vector3.left().neg().eq(Vector3.right()));
});

Deno.test("Vector3.add", () => {
  assert(Vector3.one().add(Vector3.one()).eq(new Vector3(2)));
});

Deno.test("Vector3.sub", () => {
  assert(Vector3.one().sub(Vector3.one()).eq(Vector3.zero()));
});

Deno.test("Vector3.mul", () => {
  assert(Vector3.one().mul(Vector3.one()).eq(Vector3.one()));
});

Deno.test("Vector3.div", () => {
  assert(Vector3.one().div(Vector3.one()).eq(Vector3.one()));
});

Deno.test("Vector3.eq", () => {
  assert(Vector3.one().eq(Vector3.one()));
  assert(Vector3.left().eq(Vector3.left()));
  assert(!Vector3.left().eq(Vector3.right()));
});

Deno.test("Vector3.isFinite", () => {
  assert(Vector3.zero().isFinite());
  assert(Vector3.one().isFinite());
  assert(!Vector3.negativeInfinity().isFinite());
  assert(!Vector3.positiveInfinity().isFinite());
});

Deno.test("Vector3.toArray", () => {
  assertEquals(Vector3.zero().toArray(), [0, 0, 0]);
  assertEquals(Vector3.one().toArray(), [1, 1, 1]);
  assertEquals(Vector3.left().toArray(), [-1, 0, 0]);
  assertEquals(Vector3.up().toArray(), [0, 1, 0]);
});

Deno.test("Vector3.toFloat32Array", () => {
  assertEquals(Vector3.zero().toFloat32Array(), new Float32Array([0, 0, 0]));
  assertEquals(Vector3.one().toFloat32Array(), new Float32Array([1, 1, 1]));
  assertEquals(Vector3.left().toFloat32Array(), new Float32Array([-1, 0, 0]));
  assertEquals(Vector3.up().toFloat32Array(), new Float32Array([0, 1, 0]));
});
