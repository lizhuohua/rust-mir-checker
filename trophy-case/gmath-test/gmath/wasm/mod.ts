import { source } from "./wasm.js";

const { instance } = await WebAssembly.instantiate(source);

export const memory = instance.exports.memory as WebAssembly.Memory;
export const alloc = instance.exports.alloc as (size: number) => number;
export const dealloc = instance.exports.dealloc as (
  ptr: number,
  size: number,
) => void;

export const matrix2determinant = instance.exports.matrix2determinant as (
  a: number,
) => number;
// export const matrix2invert = instance.exports.matrix2invert as (
//   a: number,
// ) => number;
export const matrix2mul = instance.exports.matrix2mul as (
  a: number,
  b: number,
) => number;
export const matrix2add = instance.exports.matrix2add as (
  a: number,
  b: number,
) => number;
export const matrix2sub = instance.exports.matrix2sub as (
  a: number,
  b: number,
) => number;

export const matrix3determinant = instance.exports.matrix3determinant as (
  a: number,
) => number;
// export const matrix3invert = instance.exports.matrix3invert as (
//   a: number,
// ) => number;
export const matrix3mul = instance.exports.matrix3mul as (
  a: number,
  b: number,
) => number;
export const matrix3add = instance.exports.matrix3add as (
  a: number,
  b: number,
) => number;
export const matrix3sub = instance.exports.matrix3sub as (
  a: number,
  b: number,
) => number;

export const matrix4determinant = instance.exports.matrix4determinant as (
  a: number,
) => number;
// export const matrix4invert = instance.exports.matrix4invert as (
//   a: number,
// ) => number;
export const matrix4mul = instance.exports.matrix4mul as (
  a: number,
  b: number,
) => number;
export const matrix4add = instance.exports.matrix4add as (
  a: number,
  b: number,
) => number;
export const matrix4sub = instance.exports.matrix4sub as (
  a: number,
  b: number,
) => number;
