/// <reference types="vitest" />

import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        typecheck: {
            include: [
                '**/*.e2e.ts'
            ]
        },
        watchExclude: ['e2e'],
        globals: true,
        include: [
            '**/*.e2e.ts'
        ]
    }
});
