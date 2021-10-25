export abstract class Angle {
  static turn: number;
  value: number;

  constructor(value = 0) {
    this.value = value;
  }

  /** The sine of the Angle */
  abstract sin(): number;
  /** The cosine of the Angle */
  abstract cos(): number;
  /** The tangent of the Angle */
  abstract tan(): number;
  /** Both the sine and cosine of the Angle */
  abstract sincos(): [number, number];
  /** The cosecant of the Angle */
  abstract csc(): number;
  /** The cotangent of the Angle */
  abstract cot(): number;
  /** The secant of the Angle */
  abstract sec(): number;
  /** The arcsine of the Angle */
  abstract asin(): number;
  /** The arccosine of the Angle */
  abstract acos(): number;
  /** The arctangent of the Angle */
  abstract atan(): number;

  /** Adds this Angle to another */
  abstract add(other: Angle | number): Angle;
  /** Subtracts this Angle to another */
  abstract sub(other: Angle | number): Angle;
  /** Multiplies this Angle to another */
  abstract mul(other: Angle | number): Angle;
  /** Divides this Angle to another */
  abstract div(other: Angle | number): Angle;
  /** Negates this Angle */
  abstract neg(): Angle;
  /** Checks if the angles are equal */
  abstract eq(other: Angle | number): boolean;

  /** Returns a new Angle normalized to to a range of 0 to a full turn */
  abstract normal(): Angle;
  /** Normalizes this Angle to a range of 0 to a full turn */
  abstract normalize(): Angle;

  /** Converts this Angle to a Rad */
  abstract toRad(): Rad;
  /** Converts this Angle to a Deg */
  abstract toDeg(): Deg;
  /** Converts this Angle to its string representation */
  abstract toString(): string;
}

export class Rad extends Angle {
  static turn = 2 * Math.PI;

  sin(): number {
    return Math.sin(this.value);
  }

  cos(): number {
    return Math.cos(this.value);
  }

  tan(): number {
    return Math.tan(this.value);
  }

  sincos(): [number, number] {
    return [Math.sin(this.value), Math.cos(this.value)];
  }

  csc(): number {
    return 1 / this.sin();
  }

  cot(): number {
    return 1 / this.tan();
  }

  sec(): number {
    return 1 / this.cos();
  }

  asin(): number {
    return Math.asin(this.value);
  }

  acos(): number {
    return Math.acos(this.value);
  }

  atan(): number {
    return Math.atan(this.value);
  }

  add(other: Angle | number): Rad {
    const value = other instanceof Angle ? other.toRad().value : other;

    return new Rad(this.value + value);
  }

  sub(other: Angle | number): Rad {
    const value = other instanceof Angle ? other.toRad().value : other;

    return new Rad(this.value - value);
  }

  mul(other: Angle | number): Rad {
    const value = other instanceof Angle ? other.toRad().value : other;

    return new Rad(this.value * value);
  }

  div(other: Angle | number): Rad {
    const value = other instanceof Angle ? other.toRad().value : other;

    return new Rad(this.value / value);
  }

  neg(): Rad {
    return new Rad(-this.value);
  }

  eq(other: Angle | number): boolean {
    const value = other instanceof Angle ? other.toRad().value : other;

    return this.value === value;
  }

  normal(): Rad {
    const rem = this.value % Rad.turn;

    return new Rad(rem < 0 ? rem + Rad.turn : rem);
  }

  normalize(): Rad {
    const rem = this.value % Rad.turn;
    this.value = rem < 0 ? rem + Rad.turn : rem;

    return this;
  }

  toRad(): Rad {
    return this;
  }

  toDeg(): Deg {
    return new Deg(this.value * (180 / Math.PI));
  }

  toString(): string {
    return `${this.value} rad`;
  }
}

export class Deg extends Angle {
  static turn = 360;

  sin(): number {
    return this.toRad().sin();
  }

  cos(): number {
    return this.toRad().cos();
  }

  tan(): number {
    return this.toRad().tan();
  }

  sincos(): [number, number] {
    return this.toRad().sincos();
  }

  csc(): number {
    return this.toRad().csc();
  }

  cot(): number {
    return this.toRad().cot();
  }

  sec(): number {
    return this.toRad().sec();
  }

  asin(): number {
    return this.toRad().asin();
  }

  acos(): number {
    return this.toRad().acos();
  }

  atan(): number {
    return this.toRad().atan();
  }

  add(other: Angle | number): Deg {
    const value = other instanceof Angle ? other.toDeg().value : other;

    return new Deg(this.value + value);
  }

  sub(other: Angle | number): Deg {
    const value = other instanceof Angle ? other.toDeg().value : other;

    return new Deg(this.value - value);
  }

  mul(other: Angle | number): Deg {
    const value = other instanceof Angle ? other.toDeg().value : other;

    return new Deg(this.value * value);
  }

  div(other: Angle | number): Deg {
    const value = other instanceof Angle ? other.toDeg().value : other;

    return new Deg(this.value / value);
  }

  neg(): Deg {
    return new Deg(-this.value);
  }

  eq(other: Angle | number): boolean {
    const value = other instanceof Angle ? other.toDeg().value : other;

    return this.value === value;
  }

  normal(): Deg {
    const rem = this.value % Deg.turn;

    return new Deg(rem < 0 ? rem + Deg.turn : rem);
  }

  normalize(): Deg {
    const rem = this.value % Deg.turn;
    this.value = rem < 0 ? rem + Deg.turn : rem;

    return this;
  }

  toRad(): Rad {
    return new Rad(this.value * (Math.PI / 180));
  }

  toDeg(): Deg {
    return this;
  }

  toString(): string {
    return `${this.value} deg`;
  }
}
