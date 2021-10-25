import { assert, assertEquals } from "./deps.ts";
import { Deg } from "../src/angle.ts";
import { Vector2 } from "../src/vector2.ts";

Deno.test("Vector2.clone", () => {
  const orig = Vector2.one();
  const clone = orig.clone();
  orig.x = 0;
  orig.y = 0;

  assert(orig.eq(Vector2.zero()));
  assert(clone.eq(Vector2.one()));
});

Deno.test("Vector2.mag", () => {
  assertEquals(Vector2.zero().mag(), 0);
  assertEquals(Vector2.right().mag(), 1);
  assertEquals(Vector2.one().mag(), 1.4142135623730951);
  assertEquals(new Vector2(5, 10).mag(), 11.180339887498949);
});

Deno.test("Vector2.mag2", () => {
  assertEquals(Vector2.zero().mag2(), 0);
  assertEquals(Vector2.right().mag2(), 1);
  assertEquals(Vector2.one().mag2(), 2);
  assertEquals(new Vector2(5, 10).mag2(), 125);
});

Deno.test("Vector2.normal", () => {
  assertEquals(Vector2.zero().normal().x, NaN);
  assertEquals(Vector2.zero().normal().y, NaN);
  assert(Vector2.right().normal().eq(Vector2.right()));
  assert(Vector2.one().normal().eq(new Vector2(0.7071067690849304)));
});

Deno.test("Vector2.angle", () => {
  assert(Vector2.zero().angle().eq(0));
  assert(Vector2.one().angle().eq(new Deg(45)));
  assert(Vector2.up().angle().eq(new Deg(90)));
  assert(Vector2.down().angle().eq(new Deg(-90)));
  assert(Vector2.left().angle().eq(new Deg(180)));
  assert(Vector2.right().angle().eq(new Deg(0)));
});

Deno.test("Vector2.clamp", () => {
  assertEquals(Vector2.zero().clamp(1).x, NaN);
  assertEquals(Vector2.zero().clamp(1).y, NaN);
  assert(Vector2.one().clamp(2).eq(new Vector2(Math.sqrt(2))));
  assert(Vector2.right().clamp(2).eq(new Vector2(2, 0)));
});

Deno.test("Vector2.dot", () => {
  assertEquals(Vector2.zero().dot(Vector2.zero()), 0);
  assertEquals(Vector2.one().dot(Vector2.one()), 2);
  assertEquals(new Vector2(1, 2).dot(new Vector2(3, 4)), 11);
});

Deno.test("Vector2.lerp", () => {
  assert(Vector2.zero().lerp(Vector2.one(), 0.5).eq(new Vector2(0.5)));
  assert(Vector2.zero().lerp(Vector2.one(), 0.25).eq(new Vector2(0.25)));
  assert(Vector2.zero().lerp(Vector2.right(), 0.5).eq(new Vector2(0.5, 0)));
});

Deno.test("Vector2.set", () => {
  const vector = new Vector2();
  vector.set(Vector2.one());

  assert(vector.eq(Vector2.one()));
});

Deno.test("Vector2.neg", () => {
  assert(Vector2.up().neg().eq(Vector2.down()));
  assert(Vector2.left().neg().eq(Vector2.right()));
});

Deno.test("Vector2.add", () => {
  assert(Vector2.one().add(Vector2.one()).eq(new Vector2(2)));
});

Deno.test("Vector2.sub", () => {
  assert(Vector2.one().sub(Vector2.one()).eq(Vector2.zero()));
});

Deno.test("Vector2.mul", () => {
  assert(Vector2.one().mul(Vector2.one()).eq(Vector2.one()));
});

Deno.test("Vector2.div", () => {
  assert(Vector2.one().div(Vector2.one()).eq(Vector2.one()));
});

Deno.test("Vector2.eq", () => {
  assert(Vector2.one().eq(Vector2.one()));
  assert(Vector2.left().eq(Vector2.left()));
  assert(!Vector2.left().eq(Vector2.right()));
});

Deno.test("Vector2.isFinite", () => {
  assert(Vector2.zero().isFinite());
  assert(Vector2.one().isFinite());
  assert(!Vector2.negativeInfinity().isFinite());
  assert(!Vector2.positiveInfinity().isFinite());
});

Deno.test("Vector2.toArray", () => {
  assertEquals(Vector2.zero().toArray(), [0, 0]);
  assertEquals(Vector2.one().toArray(), [1, 1]);
  assertEquals(Vector2.left().toArray(), [-1, 0]);
  assertEquals(Vector2.up().toArray(), [0, 1]);
});

Deno.test("Vector2.toFloat32Array", () => {
  assertEquals(Vector2.zero().toFloat32Array(), new Float32Array([0, 0]));
  assertEquals(Vector2.one().toFloat32Array(), new Float32Array([1, 1]));
  assertEquals(Vector2.left().toFloat32Array(), new Float32Array([-1, 0]));
  assertEquals(Vector2.up().toFloat32Array(), new Float32Array([0, 1]));
});
