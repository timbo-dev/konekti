# @konekti/swc-plugin-module-exports

| Package                                                                                           | Last Version                            | Github                                                                                                                                                                                              | Npm                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [@konekti/swc-plugin-module-exports](https://konekti.timbo.me/packages/swc-plugin-module-exports/)| `1.0.0`                                 | <a href="https://github.com/timbo-dev/konekti/tree/main/packages/swc-plugin-module-exports"><img src="https://github.com/timbo-dev/konekti/tree/main/docs/assets/github.svg" alt="Github Icon"></a> | <a href="https://npmjs.com/package/@konekti/swc-plugin-module-exports"><img src="https://github.com/timbo-dev/konekti/tree/main/docs/assets/npm.svg" alt="Npm Icon"></a> |


## Installation

```sh [npm]
$ npm add @konekti/swc-plugin-module-exports --save-dev
```

```sh [pnpm]
$ pnpm add @konekti/swc-plugin-module-exports -D
```

```sh [yarn]
$ yarn add @konekti/swc-plugin-module-exports -D
```

```sh [bun]
$ bun add @konekti/swc-plugin-module-exports -D
```


## TL;DR

The package "swc-plugin-module-exports" is a plugin that resolves the erroneous transpilation of the default CommonJS export in swc.

## Usage

Place the following configuration in your `.swcrc` file:

```json
"experimental": {
    "plugins": [["@konekti/swc-plugin-module-exports", {}]]
}
```

For example:

```json
{
    "$schema": "https://json.schemastore.org/swcrc",
    "module": {
        "type": "commonjs"
    },
    "jsc": {
        "target": "es2015",
        "parser": {
            "syntax": "typescript",
            "dts": true
        },
        "experimental": {
            "plugins": [["@konekti/swc-plugin-module-exports", {}]]
        }
    }
}

```

## Params

None required.

## The problem

Okay, let's say we have a software written in TypeScript with the following structure:

```sh
.
├── src
│   ├── index.ts
│   └── source.ts
├── .swcrc
├── package.json
```

The index.ts file only contains an export-all declaration:

```ts
// index.ts
export * from './source';
```

Our source.ts file contains some exports:

```ts
// source.ts
export function sum(a: number, b: number): number {
    return a + b;
}

export class Foo {
    bar(): string {
        return 'foo bar';
    }
}
```

Now, we want to transpile it to CommonJS. Our .swcrc file is this JSON:

```json
{
    "$schema": "https://json.schemastore.org/swcrc",
    "module": {
        "type": "commonjs"
    },
    "jsc": {
        "target": "es2015",
        "parser": {
            "syntax": "typescript",
            "dts": true
        }
    }
}
```

Okay, the target is **"es2015,"** and we want to transpile TypeScript to JavaScript, so the parser has the syntax set to **"typescript."** So far, so good. Let's run it.

Running the command `npx swc -d dist src`, the output should resemble something like this:

```sh
$ npx swc -d dist src
Successfully compiled: 2 files with swc (27.98ms)
```

Now, we can create a new file called index.mjs in the root of the project and place the following code into the file:

```js
import { sum, Foo } from './dist/index.js';

console.log(Foo);
console.log(sum);
```

When we run this file with node index.mjs, we encounter this problem:

```sh
$ node index.mjs
import { sum, Foo } from './dist/index.js';
              ^^^
SyntaxError: Named export 'Foo' not found. The requested module './dist/index.js' is a CommonJS module, which may not support all module.exports as named exports.
CommonJS modules can always be imported via the default export, for example using:

import pkg from './dist/index.js';
const { sum, Foo } = pkg;

    at ModuleJob._instantiate (node:internal/modules/esm/module_job:124:21)
    at async ModuleJob.run (node:internal/modules/esm/module_job:190:5)
```

Okay, it may not be supported. Let's see what tsc does in this case.

Creating a `tsconfig.json` file with this configuration:

```json
{
    "compilerOptions": {
        "target": "ES2015",
        "module": "commonjs",
        "esModuleInterop": true,
        "forceConsistentCasingInFileNames": true,
        "strict": true,
        "outDir": "dist",
        "skipLibCheck": true
    }
}
```

Let's run `npx tsc`. The following command should not appear in the output, but the directory will be created.

Now, we can run `node index.mjs` again, and we will obtain this output:

```sh
$ node index.mjs
[class Foo]
[Function: sum]
```

Wait, what??? Why does `tsc` work, and `swc` does not? Let's take a look at the transpiled code to understand.

The `swc` generated code is:

```js
// index.js
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});

_export_star(require("./source"), exports);
function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}
```

This code may look intimidating, but stay calm. It simply populates the `exports` property with the result of `require`.

```js
// source.js
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    sum: function() {
        return sum;
    },
    Foo: function() {
        return Foo;
    }
});
function sum(a, b) {
    return a + b;
}
class Foo {
    bar() {
        return 'foo bar';
    }
}
```

The same principle applies here; this script populates the `exports` property with functions and classes. Now, let's examine the `tsc` version.

```js
// index.js
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
__exportStar(require("./source"), exports);

```

Okay, this code is more complex than the `swc` version, but essentially, it accomplishes the same task: populating the `exports` object with the result of `require`.

```js
// source.js
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Foo = exports.sum = void 0;
function sum(a, b) {
    return a + b;
}
exports.sum = sum;
class Foo {
    bar() {
        return 'foo bar';
    }
}
exports.Foo = Foo;
```

The `source.js` is simpler in terms of syntax, but it achieves the same goal of populating the `exports` object.

Okay, looking at all of these files, we can see that they are equal in terms of their logic. But why does `tsc` work while `swc` does not?

The `tsc` uses the function `__exportStar` to export all functions and classes, whereas `swc` uses the function name `_export_star` for exporting.

Upon examining the `source.js` files, I noticed that the `swc` version utilizes `Object.defineProperty`, while `tsc` uses the pattern `exports.{property} = {value}`.

So, I created a custom version that swaps between these differences, and it works!

I don't know why, but when `Node.js` runs, it searches for a function named `__exportStar`. This is the reason why the `swc` version doesn't work as expected.

Another reason why the `swc` version doesn't work is because the `source.js` file uses `Object.defineProperty`, and for some reason, `Node.js` does not recognize this.

# Solution

Based on the following issues, I've been thinking, and I've come to a conclusion: let's develop a `swc` plugin.

This plugin replies to the `tsc` transpile action for `ExportDefaultExpression`, `ExportAllDeclaration`, and `ExportDeclaration` to resolve this issue.
