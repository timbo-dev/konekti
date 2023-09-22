import cp, { spawnSync } from 'child_process';
import dedent from 'dedent';
import fs from 'fs';
import path, { join } from 'path';

type SpawnOut = cp.SpawnSyncReturns<Buffer>
type SpawnOptions = cp.SpawnSyncOptionsWithBufferEncoding

const packageName = 'swc-plugin-module-exports';

let bun: (...args: string[]) => SpawnOut;
let commandSwcPluginModuleExports: (command: string, ...args: string[]) => SpawnOut;

const createCommand = (options: SpawnOptions = { stdio: 'ignore' }) =>
    (command: string, ...args: string[]): SpawnOut => {
        console.log(`\x1b[33m${command} ${args.join(' ')}`);
        return spawnSync(command, args, options);
    };
const command = createCommand();

const getTestPath = (): string => {
    return join(process.cwd(), 'e2e', packageName);
};

const createFile = (fileName: string) => {
    return (sutFunc: TemplateStringsArray) => {
        command('mkdir', '-p', `${path.dirname(fileName)}`);
        fs.writeFileSync(fileName, dedent(sutFunc[0]), {
            encoding: 'utf8'
        });
    };
};

const createSutFile = (fileName: string) => {
    return (sutFunc: TemplateStringsArray) => {
        createFile(join(getTestPath(), fileName))(sutFunc);
    };
};

const createSut = (file_path: string) => {
    return (sutFunc: TemplateStringsArray) => {
        createSutFile(file_path)(sutFunc);
        return () => {
            const response = commandSwcPluginModuleExports('node', file_path);

            return response.status;
        };
    };

};

beforeAll(() => {
    command('mkdir', '-p', `e2e/${packageName}`);
    command('bun', 'run', 'build', `${packageName}`);

    commandSwcPluginModuleExports = createCommand({
        stdio: 'inherit',
        cwd: join('e2e', packageName)
    });

    bun = (...args: string[]) => commandSwcPluginModuleExports('bun', ...args);

    bun('init', '-y');
    bun('add','@swc/core', '@swc/cli', '-D');
    bun('add', `../../packages/${packageName}`, '-D');

    createSutFile('.swcrc')/*ts*/`
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
    `;
});

afterAll(() => {
    command('rm', '-rf', `e2e/${packageName}`);
});

afterEach(() => {
    const run = commandSwcPluginModuleExports;

    run('rm', '-rf', 'dist');
    run('rm', '-rf', 'src');
    run('rm', '-rf', 'index.ts');
    run('rm', '-rf', 'sut.mjs');
});

test('should transpile named export and execute', () => {
    const run = commandSwcPluginModuleExports;

    run('npm', 'pkg', 'set', 'type="module"');

    createSutFile('src/source.ts')/*ts*/`
        export function sutFunction(message: string) : string {
            return message;
        };
        export class SutClass {
            constructor(
                public message: string
            ) {}

            getMessage() {
                return this.message;
            }
        };
        export const sut_var: string = "sut message";
    `;

    const executeNode = createSut('sut.mjs')/*ts*/`
        import { sutFunction, SutClass, sut_var } from './dist/source.js';

        if (typeof sutFunction !== 'function') process.exit(1);
        if (typeof SutClass !== 'function') process.exit(1);
        if (typeof sut_var !== 'string') process.exit(1);

        process.exit(0);
    `;

    run('bunx',' swc', '-d',' dist', 'src');

    expect(executeNode()).toBe(0);
});

test('should transpile an export all declaration and execute', () => {
    const run = commandSwcPluginModuleExports;

    run('npm', 'pkg', 'set', 'type="module"');

    createSutFile('src/source.ts')/*ts*/`
        export function sutFunction(message: string) : string {
            return message;
        };
        export class SutClass {
            constructor(
                public message: string
            ) {}

            getMessage() {
                return this.message;
            }
        };
        export const sut_var: string = "sut message";
    `;

    createSutFile('src/index.ts')/*ts*/`
        export * from './source';
    `;

    const executeNode = createSut('sut.mjs')/*ts*/`
        import { sutFunction, SutClass, sut_var } from './dist/index.js';

        if (typeof sutFunction !== 'function') process.exit(1);
        if (typeof SutClass !== 'function') process.exit(1);
        if (typeof sut_var !== 'string') process.exit(1);

        process.exit(0);
    `;

    run('bunx',' swc', '-d',' dist', 'src');
    expect(executeNode()).toBe(0);
});
