import fileTree from '../src/app.config';
import fs from 'fs';
import path from 'path';

// Convert the file tree to JSON
const fileTreeJson = JSON.stringify(fileTree, null, 2);

// Write to file-tree.json
fs.writeFileSync(path.join(__dirname, '../src/app.config.json'), fileTreeJson);

console.log('app.config exported to app.config.json successfully!');
