import { Vector2 } from "./vector2.ts";
import { Vector4 } from "./vector4.ts";

export interface Point3 {
  x: number;
  y: number;
  z: number;
}

export class Vector3 implements Point3 {
  #internal = new Float32Array(3);

  get [0](): number {
    return this.#internal[0];
  }

  set [0](val: number) {
    this.#internal[0] = val;
  }

  get [1](): number {
    return this.#internal[1];
  }

  set [1](val: number) {
    this.#internal[1] = val;
  }

  get [2](): number {
    return this.#internal[2];
  }

  set [2](val: number) {
    this.#internal[2] = val;
  }

  get x(): number {
    return this.#internal[0];
  }

  set x(val: number) {
    this.#internal[0] = val;
  }

  get y(): number {
    return this.#internal[1];
  }

  set y(val: number) {
    this.#internal[1] = val;
  }

  get z(): number {
    return this.#internal[2];
  }

  set z(val: number) {
    this.#internal[2] = val;
  }

  /** A Vector3 with all values set to Number.NEGATIVE_INFINITY */
  static negativeInfinity(): Vector3 {
    return new Vector3(Number.NEGATIVE_INFINITY);
  }

  /** A Vector3 with all values set to Number.POSITIVE_INFINITY */
  static positiveInfinity(): Vector3 {
    return new Vector3(Number.POSITIVE_INFINITY);
  }

  /** A Vector2 with all values set to 0 */
  static zero(): Vector3 {
    return new Vector3(0);
  }

  /** A Vector2 with all values set to 1 */
  static one(): Vector3 {
    return new Vector3(1);
  }

  /** Shorthand for writing Vector3(0, 1, 0) */
  static up(): Vector3 {
    return new Vector3(0, 1, 0);
  }

  /** Shorthand for writing Vector3(0, -1, 0) */
  static down(): Vector3 {
    return new Vector3(0, -1, 0);
  }

  /** Shorthand for writing Vector3(-1, 0, 0) */
  static left(): Vector3 {
    return new Vector3(-1, 0, 0);
  }

  /** Shorthand for writing Vector3(1, 0, 0) */
  static right(): Vector3 {
    return new Vector3(1, 0, 0);
  }

  /** Shorthand for writing Vector3(0, 0, -1) */
  static back(): Vector3 {
    return new Vector3(0, 0, -1);
  }

  /** Shorthand for writing Vector3(0, 0, 1) */
  static forward(): Vector3 {
    return new Vector3(0, 0, 1);
  }

  static fromHomogeneous(vector: Vector4): Vector3 {
    return vector.trunc().mul(1 / vector.w);
  }

  constructor();
  constructor(x: number);
  constructor(x: number, y: number, z: number);
  constructor(x?: number, y?: number, z?: number) {
    if (x !== undefined) {
      this.x = x;

      if (y !== undefined && z !== undefined) {
        this.y = y;
        this.z = z;
      } else {
        this.y = x;
        this.z = x;
      }
    }
  }

  /** Creates a new Vector3 with the same values */
  clone(): Vector3 {
    return new Vector3(this.x, this.y, this.z);
  }

  /** The magnitude of this Vector3 */
  mag(): number {
    return Math.hypot(this.x, this.y, this.z);
  }

  /** The squared magnitude of this Vector2 */
  mag2(): number {
    return this.x ** 2 + this.y ** 2 + this.z ** 2;
  }

  /** Returns a new Vector2 with the same direction, but with a magnitude of 1 */
  normal(): Vector3 {
    return this.div(this.mag());
  }

  /** Truncates this Vector3 to a Vector2 dropping the nth value */
  truncN(n: 0 | 1 | 2): Vector2 {
    switch (n) {
      case 0:
        return new Vector2(this.y, this.z);
      case 1:
        return new Vector2(this.x, this.z);
      case 2:
        return new Vector2(this.x, this.y);
    }
  }

  /** Truncates this Vector3 to a Vector2 dropping the z value */
  trunc(): Vector2 {
    return new Vector2(this.x, this.y);
  }

  /** Returns a new Vector2 with the same direction, but clamped to the specified length */
  clamp(length: number): Vector3 {
    return this.normal().mul(length);
  }

  /** Calculates the dot product of this Vector3 */
  dot(other: Vector3): number {
    const { x, y, z } = this.mul(other);
    return x + y + z;
  }

  /** Calculates the cross product of this and specified Vector3 */
  cross(other: Vector3): Vector3 {
    return new Vector3(
      this.y * other.z - this.z * other.y,
      this.z * other.x - this.x * other.z,
      this.x * other.y - this.y * other.x,
    );
  }

  /** Linearly interpolates between this and the specified Vector3 */
  lerp(other: Vector3, alpha: number): Vector3 {
    return this.add(other.sub(this).mul(alpha));
  }

  /** Sets the x, y and z of this Vector3 to the specified Vector3 x, y and z values */
  set(other: Vector3): Vector3 {
    this.x = other.x;
    this.y = other.y;
    this.z = other.z;

    return this;
  }

  /** Adds this Vector3 to the specified Vector3 or scalar */
  add(other: number | Vector3): Vector3 {
    const { x, y, z } = typeof other === "number"
      ? { x: other, y: other, z: other }
      : other;

    return new Vector3(this.x + x, this.y + y, this.z + z);
  }

  /** Subtracts this Vector3 from the specified Vector3 or scalar */
  sub(other: number | Vector3): Vector3 {
    const { x, y, z } = typeof other === "number"
      ? { x: other, y: other, z: other }
      : other;

    return new Vector3(this.x - x, this.y - y, this.z - z);
  }

  /** Multiplies this Vector3 with the specified Vector3 or scalar */
  mul(other: number | Vector3): Vector3 {
    const { x, y, z } = typeof other === "number"
      ? { x: other, y: other, z: other }
      : other;

    return new Vector3(this.x * x, this.y * y, this.z * z);
  }

  /** Divides this Vector3 with the specified Vector3 or scalar */
  div(other: number | Vector3): Vector3 {
    const { x, y, z } = typeof other === "number"
      ? { x: other, y: other, z: other }
      : other;

    return new Vector3(this.x / x, this.y / y, this.z / z);
  }

  /** Negates the values of this Vector3 */
  neg(): Vector3 {
    return new Vector3(-this.x, -this.y, -this.z);
  }

  /** Calculates the midpoint between two Vector3 */
  midpoint(other: Vector3): Vector3 {
    return other.sub(this).div(2).add(this);
  }

  /** Checks equality between two Vector3 */
  eq(other: Vector3): boolean {
    return this.x === other.x && this.y === other.y && this.z === other.z;
  }

  /** Checks if the Vector3 is finite */
  isFinite(): boolean {
    return isFinite(this.x) && isFinite(this.y) && isFinite(this.z);
  }

  /** Create a Vector4 using this x, y and z and the provided w */
  extend(w: number): Vector4 {
    return new Vector4(this.x, this.y, this.z, w);
  }

  /** Creates a new Vector4 using this x, y and z and setting w to 1 */
  toHomogeneous(): Vector4 {
    return this.extend(1);
  }

  /** Converts the Vector3 to a string */
  toString(): string {
    return `Vector3 { x: ${this[0]}, y: ${this[1]}, z: ${this[2]} }`;
  }

  /** Converts the Vector3 to a tuple of numbers */
  toArray(): [number, number, number] {
    return [this[0], this[1], this[2]];
  }

  /** Converts the Vector to a Float32Array */
  toFloat32Array(): Float32Array {
    return this.#internal;
  }
}
