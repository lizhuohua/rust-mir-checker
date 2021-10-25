import { assert, assertEquals } from "./deps.ts";
import { Deg, Rad } from "../src/angle.ts";
import { Matrix2 } from "../src/matrix2.ts";

Deno.test("Rad.sin", () => {
  assertEquals(new Rad(0).sin(), 0);
  assertEquals(new Rad(Rad.turn).sin(), -2.4492935982947064e-16);
});

Deno.test("Rad.cos", () => {
  assertEquals(new Rad(0).cos(), 1);
  assertEquals(new Rad(Rad.turn).cos(), 1);
});

Deno.test("Rad.tan", () => {
  assertEquals(new Rad(0).tan(), 0);
  assertEquals(new Rad(Rad.turn).sin(), -2.4492935982947064e-16);
});

Deno.test("Rad.sincos", () => {
  assertEquals(new Rad(0).sincos(), [0, 1]);
  assertEquals(new Rad(Rad.turn).sincos(), [-2.4492935982947064e-16, 1]);
});

Deno.test("Rad.csc", () => {
  assertEquals(new Rad(0).csc(), Infinity);
});

Deno.test("Rad.cot", () => {
  assertEquals(new Rad(0).cot(), Infinity);
});

Deno.test("Rad.sec", () => {
  assertEquals(new Rad(0).sec(), 1);
  assertEquals(new Rad(Rad.turn).sec(), 1);
});

Deno.test("Rad.asin", () => {
  assertEquals(new Rad(0).asin(), 0);
  assertEquals(new Rad(Rad.turn).asin(), NaN);
});

Deno.test("Rad.acos", () => {
  assertEquals(new Rad(0).acos(), 1.5707963267948966);
  assertEquals(new Rad(Rad.turn).acos(), NaN);
});

Deno.test("Rad.atan", () => {
  assertEquals(new Rad(0).atan(), 0);
  assertEquals(new Rad(Rad.turn).atan(), 1.4129651365067377);
});

Deno.test("Rad.add", () => {
  assertEquals(new Rad(0).add(1).value, 1);
  assertEquals(new Rad(Rad.turn).add(Rad.turn).value, Rad.turn + Rad.turn);
});

Deno.test("Rad.sub", () => {
  assertEquals(new Rad(0).sub(1).value, -1);
  assertEquals(new Rad(Rad.turn).sub(Rad.turn).value, 0);
});

Deno.test("Rad.mul", () => {
  assertEquals(new Rad(0).mul(1).value, 0);
  assertEquals(new Rad(Rad.turn).mul(Rad.turn).value, Rad.turn * Rad.turn);
});

Deno.test("Rad.div", () => {
  assertEquals(new Rad(0).div(1).value, 0);
  assertEquals(new Rad(Rad.turn).div(Rad.turn).value, 1);
});

Deno.test("Rad.neg", () => {
  assertEquals(new Rad(0).neg().value, -0);
  assertEquals(new Rad(Rad.turn).neg().value, -Rad.turn);
});

Deno.test("Rad.eq", () => {
  assert(new Rad(0).eq(new Deg(0)));
  assert(new Rad(Rad.turn).eq(new Deg(Deg.turn)));
  assert(new Rad(0).eq(0));
  assert(new Rad(Rad.turn).eq(Rad.turn));
});

Deno.test("Rad.normal", () => {
  assertEquals(new Rad(Rad.turn).normal().value, 0);
  assertEquals(new Rad(Rad.turn * 1.5).normal().value, Rad.turn / 2);
  assertEquals(new Rad(Rad.turn * -1.5).normal().value, Rad.turn / 2);
  assertEquals(new Rad(Rad.turn * 2.0).normal().value, 0);
});

Deno.test("Rad.normalize", () => {
  assertEquals(new Rad(Rad.turn).normalize().value, 0);
  assertEquals(new Rad(Rad.turn * 1.5).normalize().value, Rad.turn / 2);
  assertEquals(new Rad(Rad.turn * -1.5).normalize().value, Rad.turn / 2);
  assertEquals(new Rad(Rad.turn * 2.0).normalize().value, 0);
});

// TODO: move to matrix2_test
// Deno.test("Rad.toMatrix2", () => {
//   assert(new Rad(0).toMatrix2().eq(Matrix2.from(1, 0, -0, 1)));
//   assert(
//     new Rad(1).toMatrix2().eq(
//       Matrix2.from(
//         0.5403023058681398,
//         0.8414709848078965,
//         -0.8414709848078965,
//         0.5403023058681398,
//       ),
//     ),
//   );
//   assert(
//     new Rad(Rad.turn).toMatrix2().eq(
//       Matrix2.from(1, -2.4492935982947064e-16, 2.4492935982947064e-16, 1),
//     ),
//   );
// });

Deno.test("Rad.toRad", () => {
  assertEquals(new Rad(0).toRad(), new Rad(0));
  assertEquals(new Rad(0).toRad().value, new Rad(0).value);
  assertEquals(new Rad(Rad.turn).toRad(), new Rad(Rad.turn));
  assertEquals(new Rad(Rad.turn).toRad().value, new Rad(Rad.turn).value);
});

Deno.test("Rad.toDeg", () => {
  assertEquals(new Rad(0).toDeg(), new Deg(0));
  assertEquals(new Rad(Rad.turn).toDeg(), new Deg(Deg.turn));
});

Deno.test("Rad.toString", () => {
  assertEquals(new Rad(0).toString(), "0 rad");
  assertEquals(new Rad(Rad.turn).toString(), "6.283185307179586 rad");
});

Deno.test("Deg.sin", () => {
  assertEquals(new Deg(0).sin(), 0);
  assertEquals(new Deg(Deg.turn).sin(), -2.4492935982947064e-16);
});

Deno.test("Deg.cos", () => {
  assertEquals(new Deg(0).cos(), 1);
  assertEquals(new Deg(Deg.turn).cos(), 1);
});

Deno.test("Deg.tan", () => {
  assertEquals(new Deg(0).tan(), 0);
  assertEquals(new Deg(Deg.turn).sin(), -2.4492935982947064e-16);
});

Deno.test("Deg.sincos", () => {
  assertEquals(new Deg(0).sincos(), [0, 1]);
  assertEquals(new Deg(Deg.turn).sincos(), [-2.4492935982947064e-16, 1]);
});

Deno.test("Deg.csc", () => {
  assertEquals(new Deg(0).csc(), Infinity);
});

Deno.test("Deg.cot", () => {
  assertEquals(new Deg(0).cot(), Infinity);
});

Deno.test("Deg.sec", () => {
  assertEquals(new Deg(0).sec(), 1);
  assertEquals(new Deg(Deg.turn).sec(), 1);
});

Deno.test("Deg.asin", () => {
  assertEquals(new Deg(0).asin(), 0);
  assertEquals(new Deg(Deg.turn).asin(), NaN);
});

Deno.test("Deg.acos", () => {
  assertEquals(new Deg(0).acos(), 1.5707963267948966);
  assertEquals(new Deg(Deg.turn).acos(), NaN);
});

Deno.test("Deg.atan", () => {
  assertEquals(new Deg(0).atan(), 0);
  assertEquals(new Deg(Deg.turn).atan(), 1.4129651365067377);
});

Deno.test("Deg.add", () => {
  assertEquals(new Deg(0).add(1).value, 1);
  assertEquals(new Deg(Deg.turn).add(Deg.turn).value, Deg.turn + Deg.turn);
});

Deno.test("Deg.sub", () => {
  assertEquals(new Deg(0).sub(1).value, -1);
  assertEquals(new Deg(Deg.turn).sub(Deg.turn).value, 0);
});

Deno.test("Deg.mul", () => {
  assertEquals(new Deg(0).mul(1).value, 0);
  assertEquals(new Deg(Deg.turn).mul(Deg.turn).value, Deg.turn * Deg.turn);
});

Deno.test("Deg.div", () => {
  assertEquals(new Deg(0).div(1).value, 0);
  assertEquals(new Deg(Deg.turn).div(Deg.turn).value, 1);
});

Deno.test("Deg.neg", () => {
  assertEquals(new Deg(0).neg().value, -0);
  assertEquals(new Deg(Deg.turn).neg().value, -Deg.turn);
});

Deno.test("Deg.eq", () => {
  assert(new Deg(0).eq(new Rad(0)));
  assert(new Deg(Deg.turn).eq(new Rad(Rad.turn)));
  assert(new Deg(0).eq(0));
  assert(new Deg(Deg.turn).eq(Deg.turn));
});

Deno.test("Deg.normal", () => {
  assertEquals(new Deg(Deg.turn).normal().value, 0);
  assertEquals(new Deg(Deg.turn * 1.5).normal().value, Deg.turn / 2);
  assertEquals(new Deg(Deg.turn * -1.5).normal().value, Deg.turn / 2);
  assertEquals(new Deg(Deg.turn * 2.0).normal().value, 0);
});

Deno.test("Deg.normalize", () => {
  assertEquals(new Deg(Deg.turn).normalize().value, 0);
  assertEquals(new Deg(Deg.turn * 1.5).normalize().value, Deg.turn / 2);
  assertEquals(new Deg(Deg.turn * -1.5).normalize().value, Deg.turn / 2);
  assertEquals(new Deg(Deg.turn * 2.0).normalize().value, 0);
});

// TODO: move to matrix2_test
// Deno.test("Deg.toMatrix2", () => {
//   assert(new Deg(0).toMatrix2().eq(Matrix2.from(1, 0, -0, 1)));
//   assert(
//     new Deg(1).toMatrix2().eq(
//       Matrix2.from(
//         0.9998476951563913,
//         0.01745240643728351,
//         -0.01745240643728351,
//         0.9998476951563913,
//       ),
//     ),
//   );
//   assert(
//     new Deg(Deg.turn).toMatrix2().eq(
//       Matrix2.from(1, -2.4492935982947064e-16, 2.4492935982947064e-16, 1),
//     ),
//   );
// });

Deno.test("Deg.toDeg", () => {
  assertEquals(new Deg(0).toDeg(), new Deg(0));
  assertEquals(new Deg(0).toDeg().value, new Deg(0).value);
  assertEquals(new Deg(Deg.turn).toDeg(), new Deg(Deg.turn));
  assertEquals(new Deg(Deg.turn).toDeg().value, new Deg(Deg.turn).value);
});

Deno.test("Deg.toDeg", () => {
  assertEquals(new Deg(0).toDeg(), new Deg(0));
  assertEquals(new Deg(Deg.turn).toDeg(), new Deg(Deg.turn));
});

Deno.test("Deg.toString", () => {
  assertEquals(new Deg(0).toString(), "0 deg");
  assertEquals(new Deg(Deg.turn).toString(), "360 deg");
});
