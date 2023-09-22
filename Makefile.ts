import swc from '@swc/core';
import { spawnSync } from 'child_process';
import fs, { readdirSync } from 'fs';
import { GlobOptions, glob } from 'glob';
import os from 'os';
import path, { isAbsolute, join } from 'path';
import ts from 'typescript';

ts.createLanguageServiceSourceFile;

const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const ENDCOLOR = '\x1b[0m';

function error(...args: any[]): void {
    console.error(`${RED}`, ...args, `${ENDCOLOR}`);
}

function log(...args: any[]): void {
    console.log(`${GREEN}`, ...args, `${ENDCOLOR}`);
}

const args = process.argv.splice(2);
const command = args[0];
const commandList: {
    [key: string]: () => void;
} = {
    build() {
        const target = args[1];
        const buildProcess = new BuildProcess();

        if (!target) {
            const build = buildProcess
                .setBuildTarget('packages')
                .create();

            build.execute();
            return;
        }

        const build = buildProcess
            .setBuildTarget(join('packages', target))
            .create();

        build.execute();
    }
};

const execute = commandList[command];

if (!execute) {
    error(`Command '${command}' not found, exiting with status code 127`);
    process.exit(127);
}

class BuildProcess {
    private target: Package;

    public setBuildTarget(target: string): BuildProcess {
        this.target = new Package(target);

        if (!this.target.exist()) {
            error(`Package '${this.target.getName()}' does not exist, exiting with status code 127.`);
            process.exit(127);
        }

        return this;
    }

    public create(): Build {
        return new Build(this.target);
    }
}

interface ITime {
    start: number;
    end: number | null;
}

class Time {
    private static timeList: Map<string, ITime> = new Map<string, ITime>();

    public static start(label: string): void {
        if (this.timeList.has(label)) {
            error(`The label '${label}' already exists in Time`);
            process.exit(1);
        }

        this.timeList.set(label, {
            start: performance.now(),
            end: null
        });
    }

    private static calc(label: string): number {
        const time = this.timeList.get(label);

        if (!time) {
            error(`The label '${label}' does not exist`);
            process.exit(1);
        }

        if (time.end == null) {
            error('Time not ended');
            process.exit(1);
        }

        return time.end - time.start;
    }

    public static toMillis(time: number): string {
        return `${time.toFixed(2)}ms`;
    }

    public static toSeconds(time: number): string {
        return `${(time/1000).toFixed(2)}s`;
    }

    public static toMins(time: number): string {
        return `${((time/1000)/60).toFixed(2)}m`;
    }

    public static showTime(time: number): string {
        if (time < 100) return this.toMillis(time);
        else if (time < 10000) return this.toSeconds(time);
        else return this.toMins(time);
    }

    public static end(label: string): void {
        if (!this.timeList.has(label)) {
            error(`The label '${label}' does not exist`);
            process.exit(1);
        }

        const time = this.timeList.get(label) as ITime;

        this.timeList.set(label, {
            start: time.start,
            end: performance.now()
        });

        log(`Task '${label}' finished: ${YELLOW}${this.showTime(this.calc(label))}`);
    }
}

class Task {
    public static runTask<T>(taskLabel: string, task: () => T): T {
        Time.start(taskLabel);
        const val = task();
        Time.end(taskLabel);

        return val;
    }
}

class Build {
    public constructor(
        private target: Package
    ) {}

    private searchTsFiles(): Array<string> {
        return this.target.search('**/*.ts', {
            ignore: [
                '**/*.d.ts',
                '**/*.spec.ts',
                '**/*.test.ts',
                '**/*.e2e.ts'
            ]
        });
    }

    private parseToLibJs(tsFiles: Array<string>) {
        return tsFiles.map(ts => ts
            .replace('.ts', '.js')
            .replace('/src/', '/lib/')
        );
    }

    private parseToDts(tsFiles: Array<string>) {
        return tsFiles.map(ts => ts
            .replace('.ts', '.d.ts')
        );
    }

    private makeDirs(jsFiles: Array<string>): void {
        jsFiles.forEach(jsFile => {
            const directory = path.dirname(jsFile);

            if (fs.existsSync(directory))
                return;

            log(`Create dir${ENDCOLOR}\t => ${YELLOW}'${Package.shortPath(directory)}'`);

            fs.mkdirSync(path.dirname(jsFile), {
                recursive: true
            });
        });
    }

    private loadSwcConfig(configPath: string = './.swcrc'): swc.Options {
        const configContent = fs.readFileSync(configPath, 'utf-8');

        type SwcOptionsWithSchema = {
            '$schema'?: string
        } & swc.Options

        const configObject = JSON.parse(configContent) as SwcOptionsWithSchema;
        delete configObject['$schema'];

        return configObject;
    }

    private transpile(tsFile: Array<string>, config: swc.Options): Array<swc.Output> {
        return tsFile.map((tsFile) => {
            const fileContent = fs.readFileSync(tsFile, 'utf-8');

            log(`Transpile source${ENDCOLOR}\t => ${YELLOW}${Package.shortPath(tsFile)}`);
            return swc.transformSync(fileContent, config);
        });
    }

    private writeFiles(transpiledSource: Array<swc.Output>, jsFiles: Array<string>): void {
        jsFiles.forEach((jsFile, index) => {
            if (transpiledSource[index].code === 'export{};')
                return;

            log(`Write file${ENDCOLOR}\t => ${YELLOW}${Package.shortPath(jsFile)}`);
            fs.writeFileSync(jsFile, transpiledSource[index].code, 'utf-8');
        });
    }

    private makeDTSFiles(tsFiles: Array<string>, options: ts.CompilerOptions): {
        [key: string]: string
    } {
        const createdFiles = {};
        const host = ts.createCompilerHost(options);
        host.writeFile = (fileName: string, contents: string) => {
            log(`Transpiled dts file${ENDCOLOR}\t => ${YELLOW}${Package.shortPath(fileName)}`);
            return createdFiles[fileName] = contents;
        };

        const program = ts.createProgram(tsFiles, options, host);
        program.emit();

        return createdFiles;
    }

    private writeDtsFiles(createdFiles: { [key: string]: string }, dtsFiles: Array<string>) {
        dtsFiles.forEach((dtsFile) => {
            log(`Write dts file${ENDCOLOR}\t => ${YELLOW}${Package.shortPath(dtsFile)}`);

            const content = createdFiles[dtsFile];
            const target = dtsFile.replace('/src/', '/lib/');

            fs.writeFileSync(target, content, 'utf8');
        });
    }

    private runPrepackScript() {
        if (this.target.getName() === 'packages') {
            const packages = readdirSync(this.target.getPath());

            packages.forEach(package_dir => {
                spawnSync('bun', ['run', 'prepack'], {
                    cwd: join(this.target.getPath(), package_dir),
                    stdio: 'inherit'
                });
            });
        } else {
            spawnSync('bun', ['run', 'prepack'], {
                cwd: this.target.getPath(),
                stdio: 'inherit'
            });
        }
    }

    public execute() {
        Task.runTask('build', () => {
            log(`Building ${this.target.getName()}...`);

            const tsFiles: Array<string> = Task.runTask('search_ts_files', this.searchTsFiles.bind(this));
            const jsFiles: Array<string> = Task.runTask('parse_path_to_lib_js', this.parseToLibJs.bind(this, tsFiles));

            const config: swc.Options = Task.runTask('load_swc_config', this.loadSwcConfig.bind(this));
            const transpiledSource: Array<swc.Output> = Task.runTask('trasnpile', this.transpile.bind(this, tsFiles, config));

            Task.runTask('make_dirs', this.makeDirs.bind(this, jsFiles));
            Task.runTask('write_files', this.writeFiles.bind(this, transpiledSource, jsFiles));
            Task.runTask('prepack', this.runPrepackScript.bind(this));
        });
    }
}

class Package {
    public constructor(
		private path: string
    ) {
        this.path = isAbsolute(path) ? path : join(process.cwd(), path);
    }

    public static shortPath(path: string): string {
        return path.replace(os.homedir(), '...');
    }

    public getPath(): string {
        return this.path;
    }

    public getName(): string {
        return path.basename(this.path);
    }

    public exist(): boolean {
        return fs.existsSync(this.path);
    }

    public getAllFiles(): Array<string> {
        return glob.sync(path.join(this.path, '**/**'));
    }

    public getDirectories(): Array<string> {
        if (!fs.lstatSync(this.path).isDirectory()) {
            error(`The path '${this.path}' is not a directory.`);
            process.exit(1);
        }

        return fs.readdirSync(this.path).map(folderName => join(this.path, folderName));
    }

    public getFiles(): Array<string> {
        return this.search('*', {
            nodir: true
        });
    }

    public search(globSearch: string, options: GlobOptions = {}): Array<string> {
        return glob.sync(path.join(this.path, globSearch), options) as Array<string>;
    }
}

execute();
