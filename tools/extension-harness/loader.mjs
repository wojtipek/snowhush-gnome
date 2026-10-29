import {pathToFileURL} from 'node:url';
import path from 'node:path';
const dir = path.dirname(new URL(import.meta.url).pathname);
export async function resolve(specifier, context, next) {
    if (specifier.startsWith('gi://'))
        return {url: pathToFileURL(`${dir}/mocks/gi-${specifier.slice(5).split('?')[0]}.mjs`).href, shortCircuit: true};
    if (specifier.startsWith('resource:///org/gnome/shell/') || specifier.startsWith('resource:///org/gnome/Shell/Extensions/')) {
        const name = specifier.split('/').pop().replace('.js', '');
        return {url: pathToFileURL(`${dir}/mocks/shell-${name}.mjs`).href, shortCircuit: true};
    }
    return next(specifier, context);
}
