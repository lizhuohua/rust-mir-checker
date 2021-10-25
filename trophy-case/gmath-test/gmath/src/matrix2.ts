import {
  alloc,
  matrix2add,
  matrix2determinant,
  matrix2mul,
  matrix2sub,
  memory,
} from "../wasm/mod.ts";
import { Angle } from "./angle.ts";
import { Matrix3 } from "./matrix3.ts";
import { Matrix4 } from "./matrix4.ts";
import { Vector2 } from "./vector2.ts";

export class Matrix2 {
  readonly ptr: number;
  #internal: Float32Array;

  get [0](): [number, number] {
    return new Proxy([this.#internal[0], this.#internal[1]], {
      set: (_target, prop, value) => {
        if (prop === "0" || prop === "1") {
          this.#internal[prop as unknown as number] = value;
          return true;
        }
        return false;
      },
    });
  }

  set [0](val: [number, number]) {
    this.#internal[0] = val[0];
    this.#internal[1] = val[1];
  }

  get [1](): [number, number] {
    return new Proxy([this.#internal[2], this.#internal[3]], {
      set: (_target, prop, value) => {
        if (prop === "2" || prop === "3") {
          this.#internal[2 + prop as unknown as number] = value;
          return true;
        }
        return false;
      },
    });
  }

  set [1](val: [number, number]) {
    this.#internal[3] = val[0];
    this.#internal[4] = val[1];
  }

  get x(): Vector2 {
    return new Vector2(...this[0]);
  }

  set x(val: Vector2) {
    this.#internal[0] = val.x;
    this.#internal[1] = val.y;
  }

  get y(): Vector2 {
    return new Vector2(...this[1]);
  }

  set y(val: Vector2) {
    this.#internal[2] = val.x;
    this.#internal[3] = val.y;
  }

  /** Constructs a Matrix2 from individual elements */
  // deno-fmt-ignore
  static from(
    c0r0: number, c0r1: number,
    c1r0: number, c1r1: number,
  ) {
    return new Matrix2(new Vector2(c0r0, c0r1), new Vector2(c1r0, c1r1));
  }

  static fromAngle(theta: Angle): Matrix2 {
    const [s, c] = theta.sincos();
    // deno-fmt-ignore
    return Matrix2.from(
      c, s,
      -s, c
    );
  }

  static identity(): Matrix2 {
    // deno-fmt-ignore
    return Matrix2.from(
      1, 0,
      0, 1,
    );
  }

  static lookAt(dir: Vector2, up: Vector2): Matrix2 {
    const basis1 = dir.normal();
    const basis2 = up.x * dir.y >= up.y * dir.x
      ? new Vector2(basis1.y, -basis1.x)
      : new Vector2(-basis1.y, basis1.x);

    return new Matrix2(basis1, basis2);
  }

  constructor();
  constructor(ptr: number);
  constructor(x: Vector2, y: Vector2);
  constructor(x?: Vector2 | number, y?: Vector2) {
    this.ptr = typeof x === "number" ? x : alloc(16);
    this.#internal = new Float32Array(memory.buffer, this.ptr, 4);

    if (typeof x !== "number" && x !== undefined) {
      this.x = x ?? Vector2.zero();
      this.y = y ?? Vector2.zero();
    }
  }

  /** Creates a new Matrix2 with the same values */
  clone(): Matrix2 {
    return new Matrix2(this.x, this.y);
  }

  transpose(): Matrix2 {
    // deno-fmt-ignore
    return Matrix2.from(
      this[0][0], this[1][0],
      this[0][1], this[1][1],
    );
  }

  eq(other: Matrix2): boolean {
    return this[0][0] === other[0][0] && this[0][1] === other[0][1] &&
      this[1][0] === other[1][0] && this[1][1] === other[1][1];
  }

  isFinite(): boolean {
    return this.x.isFinite() && this.y.isFinite();
  }

  row(n: 0 | 1): [number, number] {
    return [this[0][n], this[1][n]];
  }

  col(n: 0 | 1): [number, number] {
    return this[n];
  }

  diag(): [number, number] {
    return [this[0][0], this[1][1]];
  }

  trace(): number {
    return this[0][0] + this[1][1];
  }

  determinant(): number {
    return matrix2determinant(this.ptr);
  }

  invert(): Matrix2 | undefined {
    let det = this.#internal[0] * this.#internal[3] -
      this.#internal[2] * this.#internal[1];
    if (det === 0) {
      return undefined;
    }

    const mat = new Matrix2();
    det = 1 / det;

    mat.#internal[0] = this.#internal[3] * det;
    mat.#internal[1] = -this.#internal[1] * det;
    mat.#internal[2] = -this.#internal[2] * det;
    mat.#internal[3] = this.#internal[0] * det;

    return mat;
  }

  add(other: Matrix2 | number): Matrix2 {
    if (typeof other === "number") {
      return new Matrix2(
        this.x.add(other),
        this.y.add(other),
      );
    }

    return new Matrix2(matrix2add(this.ptr, other.ptr));
  }

  sub(other: Matrix2 | number): Matrix2 {
    if (typeof other === "number") {
      return new Matrix2(
        this.x.sub(other),
        this.y.sub(other),
      );
    }

    return new Matrix2(matrix2sub(this.ptr, other.ptr));
  }

  mul(other: Matrix2 | number): Matrix2 {
    if (typeof other === "number") {
      return new Matrix2(
        this.x.mul(other),
        this.y.mul(other),
      );
    }

    return new Matrix2(matrix2mul(this.ptr, other.ptr));
  }

  toMatrix3(): Matrix3 {
    // deno-fmt-ignore
    return Matrix3.from(
      this[0][0], this[0][1], 0,
      this[1][0], this[1][1], 0,
      0, 0, 1,
    );
  }

  toMatrix4(): Matrix4 {
    // deno-fmt-ignore
    return Matrix4.from(
      this[0][0], this[0][1], 0, 0,
      this[1][0], this[1][1], 0, 0,
      0, 0, 1, 0,
      0, 0, 0, 1,
    );
  }

  toArray(): [[number, number], [number, number]] {
    return [this[0], this[1]];
  }

  toFloat32Array(): Float32Array {
    return this.#internal;
  }
}
