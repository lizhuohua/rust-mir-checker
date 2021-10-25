# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [0.1.8] - 2021-03-03

### Bug Fixes

- do invert in js for now ([`4243bb0`])

## [0.1.7] - 2021-03-03

### Bug Fixes

- return correct ptr after inversion ([`c4dc398`])

## [0.1.6] - 2021-03-03

### Bug Fixes

- matrix ptr constructor ([`bc47516`])

## [0.1.5] - 2021-03-03

### Bug Fixes

- setters on matrices ([`a40d1c8`])

## [0.1.4] - 2021-03-02

### Bug Fixes

- matrix4determinant ([`eebe208`])

## [0.1.3] - 2021-03-02

### Features

- wasm accelerated matrices :D ([`61a730c`])

## [0.1.2] - 2021-02-23

### Features

- fromDecomposed and Decomposed2 and Decomposed3 interfaces ([`995e5b1`])
- scalar arithmetic on matrices ([`0c8cee8`])

## [0.1.1] - 2021-02-22

### Features

- Point2, Point3 and Point4 interfaces ([`2d489b9`])

## [0.1.0] - 2021-02-21

### Features

- Matrix determinant and inverse ([`1bde4f0`])

## [0.0.8] - 2021-02-17

## [0.0.7] - 2021-02-17

### Bug Fixes

- tests ([`f9ef686`])

## [0.0.6] - 2021-02-16

### Features

- quaternions ([`7c5f102`])
- vector midpoint method ([`171c11e`])
- add to and from homogenous ([`78d8b4d`])

## [0.0.5] - 2021-02-15

### Bug Fixes

- projection ([`d0dea04`])

## [0.0.4] - 2021-02-15

### Features

- fromAngle methods and other static from methods for matrices ([`90d536e`])
- add abstract Vector class and documentation ([`08198b3`])
- identity matrices ([`766bf5c`])
- Matrix4 tests ([`8d92ab9`])
- Matrix3 tests ([`1ae3345`])
- add and sub methods for Matrix3 and Matrix4 ([`838c822`])
- Matrix2 tests ([`f733ad5`])

### Bug Fixes

- Vector tests ([`f17079a`])

## [0.0.3] - 2021-02-15

### Features

- start working on Matrix2 tests ([`cfa415f`])
- Vector4 tests ([`bdd622a`])
- Vector3 tests ([`0fbf592`])
- Vector2 tests ([`8ae759c`])
- Angle.eq ([`ac62194`])
- refactor Angle, docs and tests ([`d65fb80`])

### Bug Fixes

- matrix multiplication ([`1705875`])

## [0.0.2] - 2021-02-13

### Features

- add projections ([`49180d3`])
- matrix3 to matrix4 conversion ([`d37d610`])
- matrix multiplication ([`524922c`])
- Matrix4 and isFinite methods ([`c4c4375`])

### Bug Fixes

- matrix conversions ([`a7c5f2c`])

## [0.0.1] - 2021-02-12

### Features

- export stuff ([`4550dd6`])
- initial commit ([`301501a`])

### Bug Fixes

- update license ([`da24c16`])
- update license ([`3c81ab4`])

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[0.1.8]: https://github.com/denosaurs/gmath/compare/0.1.7...0.1.8
[`4243bb0`]: https://github.com/denosaurs/gmath/commit/4243bb0c029b9db96c1d329a8298b1827a924387
[0.1.7]: https://github.com/denosaurs/gmath/compare/0.1.6...0.1.7
[`c4dc398`]: https://github.com/denosaurs/gmath/commit/c4dc398b237cabc5caf02457fa2af576c5b6003d
[0.1.6]: https://github.com/denosaurs/gmath/compare/0.1.5...0.1.6
[`bc47516`]: https://github.com/denosaurs/gmath/commit/bc47516706a7fc04c831cfbffe76d113a8359486
[0.1.5]: https://github.com/denosaurs/gmath/compare/0.1.4...0.1.5
[`a40d1c8`]: https://github.com/denosaurs/gmath/commit/a40d1c890b8f911837cf5390787d10fd3e233ff5
[0.1.4]: https://github.com/denosaurs/gmath/compare/0.1.3...0.1.4
[`eebe208`]: https://github.com/denosaurs/gmath/commit/eebe2087776541188035586dc0f428f8e9f67ca5
[0.1.3]: https://github.com/denosaurs/gmath/compare/0.1.2...0.1.3
[`61a730c`]: https://github.com/denosaurs/gmath/commit/61a730c065342371f37d1852853e9d9be64d7070
[0.1.2]: https://github.com/denosaurs/gmath/compare/0.1.1...0.1.2
[`995e5b1`]: https://github.com/denosaurs/gmath/commit/995e5b14c3b9b08f051c31b3da14cecacd9ed94b
[`0c8cee8`]: https://github.com/denosaurs/gmath/commit/0c8cee8d1e21f57beffdc9d280f26bfddcd0e9eb
[0.1.1]: https://github.com/denosaurs/gmath/compare/0.1.0...0.1.1
[`2d489b9`]: https://github.com/denosaurs/gmath/commit/2d489b9e22953706149df0aea4d279a5ca852bc2
[0.1.0]: https://github.com/denosaurs/gmath/compare/0.0.8...0.1.0
[`1bde4f0`]: https://github.com/denosaurs/gmath/commit/1bde4f044a800cf522b1778d02f2e5b1a8ac9890
[0.0.8]: https://github.com/denosaurs/gmath/compare/0.0.7...0.0.8
[0.0.7]: https://github.com/denosaurs/gmath/compare/0.0.6...0.0.7
[`f9ef686`]: https://github.com/denosaurs/gmath/commit/f9ef6867c5f69bad952f3659d2f6aa74f41a1185
[0.0.6]: https://github.com/denosaurs/gmath/compare/0.0.5...0.0.6
[`7c5f102`]: https://github.com/denosaurs/gmath/commit/7c5f102ae6bdf0f4b73b4cf0a87c60a26db63304
[`171c11e`]: https://github.com/denosaurs/gmath/commit/171c11efdff32acaa4522ae4496058c37eea4b4b
[`78d8b4d`]: https://github.com/denosaurs/gmath/commit/78d8b4dccafe2a45354cbfff79490e00c092e1d6
[0.0.5]: https://github.com/denosaurs/gmath/compare/0.0.4...0.0.5
[`d0dea04`]: https://github.com/denosaurs/gmath/commit/d0dea041bb44d818aeadf6c371cf2154308d8a43
[0.0.4]: https://github.com/denosaurs/gmath/compare/0.0.3...0.0.4
[`90d536e`]: https://github.com/denosaurs/gmath/commit/90d536e3f54855994cd97dbcd06b2d311f109475
[`08198b3`]: https://github.com/denosaurs/gmath/commit/08198b3181d3a6f4a1fe8b393f52a00a07bb5ea2
[`766bf5c`]: https://github.com/denosaurs/gmath/commit/766bf5cfa9f63e6251cbd3a20cffec134aef3107
[`8d92ab9`]: https://github.com/denosaurs/gmath/commit/8d92ab92465a516b5e7c585a50ab3ac35a020746
[`1ae3345`]: https://github.com/denosaurs/gmath/commit/1ae3345c8fd9f2347be169b25577b33a1c8743f1
[`838c822`]: https://github.com/denosaurs/gmath/commit/838c8220fa313bf633da833d1383738e7faf9530
[`f733ad5`]: https://github.com/denosaurs/gmath/commit/f733ad5cb4d9b0baebd770585b1738d582a7ddfc
[`f17079a`]: https://github.com/denosaurs/gmath/commit/f17079a755db89728c89de952fe06f722a2eaf0e
[0.0.3]: https://github.com/denosaurs/gmath/compare/0.0.2...0.0.3
[`cfa415f`]: https://github.com/denosaurs/gmath/commit/cfa415fd50b1a33fc194213506264d0b02a2e76b
[`bdd622a`]: https://github.com/denosaurs/gmath/commit/bdd622aacc873f1bd1234c6fe51627befaa2d307
[`0fbf592`]: https://github.com/denosaurs/gmath/commit/0fbf5929cf33f59a96f8ed7b25f64a5e8d6830b9
[`8ae759c`]: https://github.com/denosaurs/gmath/commit/8ae759ca0656ca586585fd691e7ef3525815d28e
[`ac62194`]: https://github.com/denosaurs/gmath/commit/ac62194ade7bbc644302b98f728cc3503304a96d
[`d65fb80`]: https://github.com/denosaurs/gmath/commit/d65fb8073a83546e0dccb77eb4cbf2b0578a6a03
[`1705875`]: https://github.com/denosaurs/gmath/commit/1705875feb0e3eff61ea590bd652520b7523f733
[0.0.2]: https://github.com/denosaurs/gmath/compare/0.0.1...0.0.2
[`49180d3`]: https://github.com/denosaurs/gmath/commit/49180d3d3a80fe04af17a02d87d9b37eaf9cc7ba
[`d37d610`]: https://github.com/denosaurs/gmath/commit/d37d6108fcfb67d7c7df66d252b47dd5b15b3055
[`524922c`]: https://github.com/denosaurs/gmath/commit/524922c3e73c5d936b54c61af1c6b7d3b6fd7c81
[`c4c4375`]: https://github.com/denosaurs/gmath/commit/c4c4375f8f3d230ae52544064bac7c9783d5f6b1
[`a7c5f2c`]: https://github.com/denosaurs/gmath/commit/a7c5f2c34f37a263efcfee6690172d9347da4680
[0.0.1]: https://github.com/denosaurs/gmath/compare/0.0.1
[`4550dd6`]: https://github.com/denosaurs/gmath/commit/4550dd6ec689029651aa2638ac7982d3b7a7bc16
[`301501a`]: https://github.com/denosaurs/gmath/commit/301501ac55cff5a092a37e602dc0d6ab5ea17d24
[`da24c16`]: https://github.com/denosaurs/gmath/commit/da24c16b2722076685daee4b27a5b379ba16b694
[`3c81ab4`]: https://github.com/denosaurs/gmath/commit/3c81ab4ba7e9505bda193e0877193f825984a8ab
