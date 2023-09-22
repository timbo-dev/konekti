import { DefaultTheme, defineConfig } from 'vitepress';
import { glob } from 'glob';
import { basename, join, resolve } from 'path';
import { execSync } from 'child_process';
import { cpSync, existsSync, lstat, lstatSync, readFileSync, readdirSync } from 'fs';

const monorepoName = 'Konekti'
const packages = glob.sync('packages/*')

packages.forEach(packagePath => {
    cpSync(join(packagePath, 'docs'), join('docs', packagePath), {
        recursive: true,
        force: true,
        filter: ((source, dest) => {
            if (source.endsWith('config.json')) return false;

            return true
        })
    })
})

const sidebarPackages: DefaultTheme.SidebarItem[] = packages.map(packagePath => {
    const text = `@${monorepoName.toLowerCase()}/${basename(packagePath)}`

    const docConfigPath = join(packagePath, 'docs', 'config.json')
    let items: DefaultTheme.SidebarItem[] = []

    if (existsSync(docConfigPath)) {
        const docConfig = readFileSync(docConfigPath, 'utf-8')
        items = JSON.parse(docConfig) as DefaultTheme.SidebarItem[]
    }

    return {
        link: `/${packagePath}/`,
        collapsed: true,
        text,
        items
    }
})

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: monorepoName,
    base: '/',
    description: "A tool box with several awesome tools.",
    head: [['link', { rel: 'icon', href: '/media/favicon.ico' }]],
    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        logo: `/media/${monorepoName.toLowerCase()}-logo.svg`,
        search: {
            provider: 'local',
        },
        nav: [
            { text: 'Home', link: '/' },
        ],
        sidebar: [
            {
                text: 'Introduction',
                items: [
                    { text: `What is ${monorepoName}?`, link: '/introduction/' },
                    {
                        text: 'Packages',
                        link: '/packages/',
                        items: sidebarPackages
                    }
                ]
            }
        ],
        socialLinks: [
            { icon: 'github', link: `https://github.com/timbo-dev/${monorepoName.toLowerCase()}` }
        ],
        footer: {
            message: `Released under the <a href="https://github.com/timbo-dev/${monorepoName.toLowerCase()}/blob/main/LICENSE">MIT License</a>.`,
            copyright: 'Copyright © 2023-present <a href="https://github.com/timbo-dev">Timbo</a>'
        }
    }
})
