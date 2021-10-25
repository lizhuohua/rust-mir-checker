import {
  alloc,
  matrix3add,
  matrix3determinant,
  matrix3mul,
  matrix3sub,
  memory,
} from "../wasm/mod.ts";
import { Vector3 } from "./vector3.ts";
import { Vector2 } from "./vector2.ts";
import { Matrix2 } from "./matrix2.ts";
import { Matrix4 } from "./matrix4.ts";
import { Angle } from "./angle.ts";
import { Quaternion } from "./quaternion.ts";
import { Decomposed2 } from "./decomposed.ts";

export class Matrix3 {
  readonly ptr: number;
  #internal: Float32Array;

  get [0](): [number, number, number] {
    return new Proxy(
      [this.#internal[0], this.#internal[1], this.#internal[2]],
      {
        set: (_target, prop, value) => {
          if (prop === "0" || prop === "1" || prop === "2") {
            this.#internal[prop as unknown as number] = value;
            return true;
          }
          return false;
        },
      },
    );
  }

  set [0](val: [number, number, number]) {
    this.#internal[0] = val[0];
    this.#internal[1] = val[1];
    this.#internal[2] = val[2];
  }

  get [1](): [number, number, number] {
    return new Proxy(
      [this.#internal[3], this.#internal[4], this.#internal[5]],
      {
        set: (_target, prop, value) => {
          if (prop === "0" || prop === "1" || prop === "2") {
            this.#internal[3 + prop as unknown as number] = value;
            return true;
          }
          return false;
        },
      },
    );
  }

  set [1](val: [number, number, number]) {
    this.#internal[3] = val[0];
    this.#internal[4] = val[1];
    this.#internal[5] = val[2];
  }

  get [2](): [number, number, number] {
    return new Proxy(
      [this.#internal[6], this.#internal[7], this.#internal[8]],
      {
        set: (_target, prop, value) => {
          if (prop === "0" || prop === "1" || prop === "2") {
            this.#internal[6 + prop as unknown as number] = value;
            return true;
          }
          return false;
        },
      },
    );
  }

  set [2](val: [number, number, number]) {
    this.#internal[6] = val[0];
    this.#internal[7] = val[1];
    this.#internal[8] = val[2];
  }

  get x(): Vector3 {
    return new Vector3(...this[0]);
  }

  set x(val: Vector3) {
    this.#internal[0] = val.x;
    this.#internal[1] = val.y;
    this.#internal[2] = val.z;
  }

  get y(): Vector3 {
    return new Vector3(...this[1]);
  }

  set y(val: Vector3) {
    this.#internal[3] = val.x;
    this.#internal[4] = val.y;
    this.#internal[5] = val.z;
  }

  get z(): Vector3 {
    return new Vector3(...this[2]);
  }

  set z(val: Vector3) {
    this.#internal[6] = val.x;
    this.#internal[7] = val.y;
    this.#internal[8] = val.z;
  }

  /** Constructs a Matrix3 from individual elements */
  // deno-fmt-ignore
  static from(
    c0r0: number, c0r1: number, c0r2: number,
    c1r0: number, c1r1: number, c1r2: number,
    c2r0: number, c2r1: number, c2r2: number,
  ) {
    return new Matrix3(
      new Vector3(c0r0, c0r1, c0r2),
      new Vector3(c1r0, c1r1, c1r2),
      new Vector3(c2r0, c2r1, c2r2),
    );
  }

  static identity(): Matrix3 {
    // deno-fmt-ignore
    return Matrix3.from(
      1, 0, 0,
      0, 1, 0,
      0, 0, 1,
    );
  }

  static lookToLh(dir: Vector3, up: Vector3): Matrix3 {
    dir = dir.normal();
    const side = up.cross(dir).normal();
    up = dir.cross(side).normal();

    return new Matrix3(side, up, dir).transpose();
  }

  static lookToRh(dir: Vector3, up: Vector3): Matrix3 {
    return Matrix3.lookToLh(dir.neg(), up);
  }

  static lookAtLh(eye: Vector2, center: Vector2, up: Vector2): Matrix3 {
    const dir = center.sub(eye);
    return Matrix2.lookAt(dir, up).toMatrix3();
  }

  static lookAtRh(eye: Vector2, center: Vector2, up: Vector2): Matrix3 {
    const dir = eye.sub(center);
    return Matrix2.lookAt(dir, up).toMatrix3();
  }

  static fromAngleX(theta: Angle): Matrix3 {
    const [s, c] = theta.sincos();

    // deno-fmt-ignore
    return Matrix3.from(
      1, 0, 0,
      0, c, s,
      0, -s, c,
    );
  }

  static fromAngleY(theta: Angle): Matrix3 {
    const [s, c] = theta.sincos();

    // deno-fmt-ignore
    return Matrix3.from(
      c, 0, -s,
      0, 1, 0,
      s, 0, c,
    );
  }

  static fromAngleZ(theta: Angle): Matrix3 {
    const [s, c] = theta.sincos();

    // deno-fmt-ignore
    return Matrix3.from(
      c, s, 0,
      -s, c, 0,
      0, 0, 1,
    );
  }

  static fromAxisAngle(axis: Vector3, angle: Angle): Matrix3 {
    const [s, c] = angle.sincos();
    const c1 = 1 - c;

    return Matrix3.from(
      c1 * axis.x * axis.x + c,
      c1 * axis.x * axis.y + s * axis.z,
      c1 * axis.x * axis.z - s * axis.y,
      c1 * axis.x * axis.y - s * axis.z,
      c1 * axis.y * axis.y + c,
      c1 * axis.y * axis.z + s * axis.x,
      c1 * axis.x * axis.z + s * axis.y,
      c1 * axis.y * axis.z - s * axis.x,
      c1 * axis.z * axis.z + c,
    );
  }

  static fromTranslation(translation: Vector2): Matrix3 {
    // deno-fmt-ignore
    return Matrix3.from(
      1, 0, 0,
      0, 1, 0,
      translation.x, translation.y, 1,
    );
  }

  static fromScale(scale: number): Matrix3 {
    return this.fromNonuniformScale(scale, scale);
  }

  static fromNonuniformScale(x: number, y: number): Matrix3 {
    // deno-fmt-ignore
    return Matrix3.from(
      x, 0, 0,
      0, y, 0,
      0, 0, 1,
    );
  }

  static fromQuaternion(quaternion: Quaternion): Matrix3 {
    const x2 = quaternion.vector.x * 2;
    const y2 = quaternion.vector.y * 2;
    const z2 = quaternion.vector.z * 2;

    const xx2 = x2 * quaternion.vector.x;
    const xy2 = x2 * quaternion.vector.y;
    const xz2 = x2 * quaternion.vector.z;

    const yy2 = y2 * quaternion.vector.y;
    const yz2 = y2 * quaternion.vector.z;
    const zz2 = z2 * quaternion.vector.z;

    const sy2 = y2 * quaternion.scalar;
    const sz2 = z2 * quaternion.scalar;
    const sx2 = x2 * quaternion.scalar;

    // deno-fmt-ignore
    return Matrix3.from(
      1 - yy2 - zz2, xy2 + sz2, xz2 - sy2,
      xy2 - sz2, 1 - xx2 - zz2, yz2 + sx2,
      xz2 + sy2, yz2 - sx2, 1 - xx2 - yy2,
    );
  }

  static fromDecomposed(decomposed: Decomposed2): Matrix3 {
    const m = Matrix2.fromAngle(decomposed.rot).mul(decomposed.scale)
      .toMatrix3();
    m.z = decomposed.disp.extend3(1);
    return m;
  }

  constructor();
  constructor(ptr: number);
  constructor(x: Vector3, y: Vector3, z: Vector3);
  constructor(x?: Vector3 | number, y?: Vector3, z?: Vector3) {
    this.ptr = typeof x === "number" ? x : alloc(36);
    this.#internal = new Float32Array(memory.buffer, this.ptr, 9);

    if (typeof x !== "number" && x !== undefined) {
      this.x = x ?? Vector3.zero();
      this.y = y ?? Vector3.zero();
      this.z = z ?? Vector3.zero();
    }
  }

  /** Creates a new Matrix3 with the same values */
  clone(): Matrix3 {
    return new Matrix3(this.x, this.y, this.z);
  }

  transpose(): Matrix3 {
    // deno-fmt-ignore
    return Matrix3.from(
      this[0][0], this[1][0], this[2][0],
      this[0][1], this[1][1], this[2][1],
      this[0][2], this[1][2], this[2][2],
    );
  }

  eq(other: Matrix3): boolean {
    return this[0][0] === other[0][0] &&
      this[0][1] === other[0][1] &&
      this[0][2] === other[0][2] &&
      this[1][0] === other[1][0] &&
      this[1][1] === other[1][1] &&
      this[1][2] === other[1][2] &&
      this[2][0] === other[2][0] &&
      this[2][1] === other[2][1] &&
      this[2][2] === other[2][2];
  }

  isFinite(): boolean {
    return this.x.isFinite() && this.y.isFinite() && this.z.isFinite();
  }

  row(n: 0 | 1 | 2): [number, number, number] {
    return [this[0][n], this[1][n], this[2][n]];
  }

  col(n: 0 | 1 | 2): [number, number, number] {
    return this[n];
  }

  diag(): [number, number, number] {
    return [this[0][0], this[1][1], this[2][2]];
  }

  trace(): number {
    return this[0][0] + this[1][1] + this[2][2];
  }

  determinant(): number {
    return matrix3determinant(this.ptr);
  }

  invert(): Matrix3 | undefined {
    const b01 = this.#internal[8] * this.#internal[4] -
      this.#internal[5] * this.#internal[7];
    const b11 = -this.#internal[8] * this.#internal[3] +
      this.#internal[5] * this.#internal[6];
    const b21 = this.#internal[7] * this.#internal[3] -
      this.#internal[4] * this.#internal[6];

    let det = this.#internal[0] * b01 + this.#internal[1] * b11 +
      this.#internal[2] * b21;

    if (det === 0) {
      return undefined;
    }

    const mat = new Matrix3();
    det = 1 / det;

    mat.#internal[0] = b01 * det;
    mat.#internal[1] =
      (-this.#internal[8] * this.#internal[1] +
        this.#internal[2] * this.#internal[7]) * det;
    mat.#internal[2] =
      (this.#internal[5] * this.#internal[1] -
        this.#internal[2] * this.#internal[4]) * det;
    mat.#internal[3] = b11 * det;
    mat.#internal[4] =
      (this.#internal[8] * this.#internal[0] -
        this.#internal[2] * this.#internal[6]) * det;
    mat.#internal[5] =
      (-this.#internal[5] * this.#internal[0] +
        this.#internal[2] * this.#internal[3]) * det;
    mat.#internal[6] = b21 * det;
    mat.#internal[7] =
      (-this.#internal[7] * this.#internal[0] +
        this.#internal[1] * this.#internal[6]) * det;
    mat.#internal[8] =
      (this.#internal[4] * this.#internal[0] -
        this.#internal[1] * this.#internal[3]) * det;

    return mat;
  }

  add(other: Matrix3 | number): Matrix3 {
    if (typeof other === "number") {
      return new Matrix3(
        this.x.add(other),
        this.y.add(other),
        this.z.add(other),
      );
    }

    return new Matrix3(matrix3add(this.ptr, other.ptr));
  }

  sub(other: Matrix3 | number): Matrix3 {
    if (typeof other === "number") {
      return new Matrix3(
        this.x.sub(other),
        this.y.sub(other),
        this.z.sub(other),
      );
    }

    return new Matrix3(matrix3sub(this.ptr, other.ptr));
  }

  mul(other: Matrix3 | number): Matrix3 {
    if (typeof other === "number") {
      return new Matrix3(
        this.x.mul(other),
        this.y.mul(other),
        this.z.mul(other),
      );
    }

    return new Matrix3(matrix3mul(this.ptr, other.ptr));
  }

  toMatrix4(): Matrix4 {
    // deno-fmt-ignore
    return Matrix4.from(
      this[0][0], this[0][1], this[0][2], 0,
      this[1][0], this[1][1], this[1][2], 0,
      this[2][0], this[2][1], this[2][2], 0,
      0, 0, 0, 1,
    );
  }

  toArray(): [
    [number, number, number],
    [number, number, number],
    [number, number, number],
  ] {
    return [this[0], this[1], this[2]];
  }

  toFloat32Array(): Float32Array {
    return this.#internal;
  }
}
