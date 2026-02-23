import { copyFileSync, mkdirSync } from 'fs';

// Ensure dist directory exists
mkdirSync('dist', { recursive: true });

// Copy static files to dist
copyFileSync('src/index.html', 'dist/index.html');
copyFileSync('src/styles.css', 'dist/styles.css');

console.log('Static files copied to dist/');
