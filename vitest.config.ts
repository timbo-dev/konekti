/// <reference types="vitest" />

import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        typecheck: {
            include: [
                '**/*.spec.ts',
                '**/*.test.ts'
            ]
        },
        globals: true,
        include: [
            '**/*.spec.ts',
            '**/*.test.ts'
        ]
    }
});
